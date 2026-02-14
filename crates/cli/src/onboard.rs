//! Interactive onboarding wizard for Open Clanker
//!
//! Prompts user for API keys and configuration, then writes config.toml and .env.

use anyhow::Result;
use clanker_config::{AgentConfig, ChannelsConfig, Config, DiscordConfig, LoggingConfig, ServerConfig, TelegramConfig};
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use std::io::IsTerminal;
use std::path::Path;

/// Provider configuration for onboarding
struct ProviderInfo {
    name: &'static str,
    api_key_env: &'static str,
    default_model: &'static str,
}

const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        name: "Anthropic (Claude)",
        api_key_env: "OPENCLAW_ANTHROPIC_API_KEY",
        default_model: "claude-sonnet-4-20250514",
    },
    ProviderInfo {
        name: "OpenAI (GPT)",
        api_key_env: "OPENCLAW_OPENAI_API_KEY",
        default_model: "gpt-4",
    },
    ProviderInfo {
        name: "xAI (Grok)",
        api_key_env: "OPENCLAW_GROK_API_KEY",
        default_model: "grok-2",
    },
    ProviderInfo {
        name: "Groq (llama-3)",
        api_key_env: "OPENCLAW_GROQ_API_KEY",
        default_model: "llama-3.3-70b-versatile",
    },
    ProviderInfo {
        name: "Z.ai (GLM-4.7)",
        api_key_env: "OPENCLAW_ZAI_API_KEY",
        default_model: "glm-4.7",
    },
];

const PROVIDER_IDS: &[&str] = &["anthropic", "openai", "grok", "groq", "zai"];

/// Run the onboarding wizard
pub fn run_onboard(config_path: &Path, env_path: &Path) -> Result<()> {
    if !std::io::stdin().is_terminal() {
        anyhow::bail!("Onboarding requires an interactive terminal. Run: open-clanker onboard");
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           Open Clanker — Setup Wizard                    ║");
    println!("║                                                          ║");
    println!("║  Configure your API keys and channels.                   ║");
    println!("║  Secrets are stored in .env (not committed to git).      ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // 1. AI Provider
    let provider_idx = Select::new()
        .with_prompt("Select AI Provider")
        .items(
            &PROVIDERS
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>(),
        )
        .default(0)
        .interact()?;

    let provider = PROVIDER_IDS[provider_idx];
    let provider_info = &PROVIDERS[provider_idx];

    // 2. API Key (primary provider)
    let api_key: String = Password::new()
        .with_prompt(format!("{} API key", provider_info.name))
        .allow_empty_password(false)
        .interact()?;

    // 2b. Add API keys for other providers?
    let mut extra_keys: Vec<(&str, &str, String)> = Vec::new(); // (provider_id, env_var, key)
    let other_providers: Vec<_> = PROVIDER_IDS
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != provider_idx)
        .map(|(i, &id)| (i, id, PROVIDERS[i].name))
        .collect();
    if !other_providers.is_empty()
        && Confirm::new()
            .with_prompt("Add API keys for other providers? (for fallback, workers, or switching)")
            .default(false)
            .interact()?
    {
        let choices: Vec<String> = other_providers
            .iter()
            .map(|(_, _, name)| format!("{}", name))
            .collect();
        let selected = MultiSelect::new()
            .with_prompt("Select providers to add keys for")
            .items(&choices)
            .interact()?;
        for &idx in &selected {
            let (_, id, name) = &other_providers[idx];
            let key = Password::new()
                .with_prompt(format!("{} API key", name))
                .allow_empty_password(true)
                .interact()?;
            let key = key.trim().to_string();
            if !key.is_empty() {
                let (orig_idx, _, _) = other_providers[idx];
                let env_var = PROVIDERS[orig_idx].api_key_env;
                extra_keys.push((id, env_var, key));
            }
        }
    }

    // 3. Enable Telegram?
    let enable_telegram = Confirm::new()
        .with_prompt("Enable Telegram channel?")
        .default(true)
        .interact()?;

    let telegram_token = if enable_telegram {
        Some(
            Password::new()
                .with_prompt("Telegram bot token (from @BotFather)")
                .allow_empty_password(false)
                .interact()?,
        )
    } else {
        None
    };

    // 4. Enable Discord?
    let enable_discord = Confirm::new()
        .with_prompt("Enable Discord channel?")
        .default(false)
        .interact()?;

    let discord_token = if enable_discord {
        Some(
            Password::new()
                .with_prompt("Discord bot token (from Developer Portal)")
                .allow_empty_password(false)
                .interact()?,
        )
    } else {
        None
    };

    // Require at least one channel
    if telegram_token.is_none() && discord_token.is_none() {
        anyhow::bail!("At least one channel (Telegram or Discord) is required. Run onboard again.");
    }

    let has_extra_key = |env_var: &str| extra_keys.iter().any(|(_, ev, _)| *ev == env_var);

    // 5. Z.ai fallback (when primary is Claude/OpenAI and user wants fallback on failure)
    let (zai_fallback_key, add_fallback) = if provider != "zai" && (provider == "anthropic" || provider == "openai") {
        let add = Confirm::new()
            .with_prompt("Add Z.ai (GLM-4.7) as fallback when primary fails? (Master_Clanker resilience)")
            .default(true)
            .interact()?;
        if add {
            let key = if has_extra_key("OPENCLAW_ZAI_API_KEY") {
                extra_keys.iter().find(|(_, ev, _)| *ev == "OPENCLAW_ZAI_API_KEY").map(|(_, _, k)| k.clone()).unwrap_or_default()
            } else {
                Password::new()
                    .with_prompt("Z.ai API key (fallback)")
                    .allow_empty_password(true)
                    .interact()?
                    .trim()
                    .to_string()
            };
            (if key.is_empty() { None } else { Some(key) }, add)
        } else {
            (None, false)
        }
    } else {
        (None, false)
    };

    // 6. Groq API key (for Worker_Clankers when orchestration enabled and master is not Groq)
    let groq_key: Option<String> = if provider != "groq" {
        let need_groq = Confirm::new()
            .with_prompt("Orchestration uses Groq for Worker_Clankers. Add Groq API key now?")
            .default(true)
            .interact()?;
        if need_groq {
            if has_extra_key("OPENCLAW_GROQ_API_KEY") {
                extra_keys.iter().find(|(_, ev, _)| *ev == "OPENCLAW_GROQ_API_KEY").map(|(_, _, k)| k.clone())
            } else {
                let key = Password::new()
                    .with_prompt("Groq API key (for Worker_Clankers)")
                    .allow_empty_password(true)
                    .interact()?;
                let key = key.trim().to_string();
                if key.is_empty() { None } else { Some(key) }
            }
        } else {
            None
        }
    } else {
        None
    };

    // 7. Server port
    let port_str: String = Input::new()
        .with_prompt("Server port")
        .default("18789".to_string())
        .validate_with(|s: &String| {
            s.parse::<u16>()
                .map(|_| ())
                .map_err(|_| "Enter a valid port (1-65535)".to_string())
        })
        .interact()?;
    let port: u16 = port_str.parse().expect("validated above");

    // 8. pCloud access (OAuth token for file storage, skills, shared links)
    let pcloud_token: Option<String> = if Confirm::new()
        .with_prompt("Add pCloud access? (OAuth token for file storage, skills libraries)")
        .default(false)
        .interact()?
    {
        let token = Password::new()
            .with_prompt("pCloud OAuth access token (from https://my.pcloud.com/oauth2/authorize)")
            .allow_empty_password(true)
            .interact()?;
        let token = token.trim().to_string();
        if token.is_empty() { None } else { Some(token) }
    } else {
        None
    };

    // 9. ProtonMail (email provider for Open Clanker)
    let (protonmail_username, protonmail_token): (Option<String>, Option<String>) = if Confirm::new()
        .with_prompt("Add ProtonMail as email provider? (SMTP for sending)")
        .default(false)
        .interact()?
    {
        let username: String = Input::new()
            .with_prompt("ProtonMail email address")
            .allow_empty(true)
            .interact()
            .unwrap_or_default();
        let username = username.trim().to_string();
        let token = if !username.is_empty() {
            let t = Password::new()
                .with_prompt("ProtonMail SMTP token (Settings → IMAP/SMTP → SMTP tokens)")
                .allow_empty_password(true)
                .interact()?;
            let t = t.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        } else {
            None
        };
        (if username.is_empty() { None } else { Some(username) }, token)
    } else {
        (None, None)
    };

    // Build config
    let mut channels = ChannelsConfig {
        telegram: None,
        discord: None,
    };

    if enable_telegram {
        channels.telegram = Some(TelegramConfig {
            bot_token: "from-env".to_string(), // Placeholder; real value from .env
            allowed_chats: None,
        });
    }

    if enable_discord {
        channels.discord = Some(DiscordConfig {
            bot_token: "from-env".to_string(),
            guild_id: None,
        });
    }

    let agent_fallback = if add_fallback && zai_fallback_key.is_some() {
        Some(clanker_config::FallbackAgentConfig {
            provider: "zai".to_string(),
            model: "glm-4.7".to_string(),
            api_key_env: "OPENCLAW_ZAI_API_KEY".to_string(),
            api_key: None, // From env
        })
    } else {
        None
    };

    let config = Config {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port,
            tls: None,
        },
        channels,
        agent: AgentConfig {
            provider: provider.to_string(),
            model: provider_info.default_model.to_string(),
            api_key_env: provider_info.api_key_env.to_string(),
            api_key: None, // Always from env
            max_tokens: 4096,
            api_base_url: None,
            worker: None,
            fallback: agent_fallback,
        },
        orchestration: clanker_config::OrchestrationConfig::default(),
        logging: LoggingConfig::default(),
    };

    // Write config.toml (no secrets)
    let config_toml = toml::to_string_pretty(&config)?;
    std::fs::write(config_path, config_toml)?;
    println!();
    println!("✓ Wrote {}", config_path.display());

    // Write .env (secrets)
    let mut env_lines: Vec<String> = vec![
        "# Open Clanker — Generated by onboard. Do not commit.".to_string(),
        "".to_string(),
        format!("{}={}", provider_info.api_key_env, api_key),
    ];

    for (_, env_var, key) in &extra_keys {
        env_lines.push(format!("{}={}", env_var, key));
    }

    if let Some(t) = &telegram_token {
        env_lines.push(format!("OPENCLAW_TELEGRAM_BOT_TOKEN={}", t));
    }
    if let Some(t) = &discord_token {
        env_lines.push(format!("OPENCLAW_DISCORD_BOT_TOKEN={}", t));
    }
    if let Some(k) = &groq_key {
        if !has_extra_key("OPENCLAW_GROQ_API_KEY") {
            env_lines.push(format!("OPENCLAW_GROQ_API_KEY={}", k));
        }
    }
    if let Some(k) = &zai_fallback_key {
        if !has_extra_key("OPENCLAW_ZAI_API_KEY") {
            env_lines.push(format!("OPENCLAW_ZAI_API_KEY={}", k));
        }
    }
    if let Some(t) = &pcloud_token {
        env_lines.push(format!("OPENCLAW_PCLOUD_ACCESS_TOKEN={}", t));
    }
    if let Some(u) = &protonmail_username {
        env_lines.push(format!("OPENCLAW_PROTONMAIL_USERNAME={}", u));
    }
    if let Some(t) = &protonmail_token {
        env_lines.push(format!("OPENCLAW_PROTONMAIL_SMTP_TOKEN={}", t));
    }

    env_lines.push("".to_string());
    env_lines.push("# Optional overrides:".to_string());
    env_lines.push("# OPENCLAW_HOST=0.0.0.0".to_string());
    env_lines.push(format!("# OPENCLAW_PORT={}", port));

    let env_content = env_lines.join("\n");
    std::fs::write(env_path, env_content)?;
    println!("✓ Wrote {}", env_path.display());

    // Load .env and validate config
    println!();
    println!("Validating configuration...");
    if let Err(e) = dotenvy::from_path(env_path) {
        println!("⚠ Could not load .env: {}", e);
    } else if let Ok(mut config) = Config::load_from_path(config_path) {
        if let Err(e) = config.load_env() {
            println!("⚠ Could not load env vars: {}", e);
        } else if let Err(e) = config.validate() {
            println!("✗ Config validation failed: {}", e);
        } else {
            println!("✓ Configuration is valid!");
        }
    } else {
        println!("⚠ Could not load config for validation");
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  Setup complete!                                         ║");
    println!("║                                                          ║");
    println!("║  Next steps:                                             ║");
    println!("║  1. Run: source .env     (load secrets)                  ║");
    println!("║  2. Run: open-clanker config-validate                    ║");
    println!("║  3. Run: open-clanker gateway                            ║");
    println!("║                                                          ║");
    println!("║  Or in one line:                                         ║");
    println!("║    source .env && open-clanker gateway                   ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}
