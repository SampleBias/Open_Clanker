//! Interactive onboarding wizard for Open Clanker
//!
//! Prompts user for API keys and configuration, then writes config.toml and .env.

use anyhow::Result;
use clanker_config::{AgentConfig, ChannelsConfig, Config, DiscordConfig, LoggingConfig, ServerConfig, TelegramConfig};
use dialoguer::{Confirm, Input, Password, Select};
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
        name: "Grok (xAI)",
        api_key_env: "OPENCLAW_GROK_API_KEY",
        default_model: "grok-2",
    },
    ProviderInfo {
        name: "Groq",
        api_key_env: "OPENCLAW_GROQ_API_KEY",
        default_model: "llama-3.3-70b-versatile",
    },
];

const PROVIDER_IDS: &[&str] = &["anthropic", "openai", "grok", "groq"];

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

    // 2. API Key
    let api_key: String = Password::new()
        .with_prompt(format!("{} API key", provider_info.name))
        .allow_empty_password(false)
        .interact()?;

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

    // 5. Server port
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
        },
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

    if let Some(t) = &telegram_token {
        env_lines.push(format!("OPENCLAW_TELEGRAM_BOT_TOKEN={}", t));
    }
    if let Some(t) = &discord_token {
        env_lines.push(format!("OPENCLAW_DISCORD_BOT_TOKEN={}", t));
    }

    env_lines.push("".to_string());
    env_lines.push("# Optional overrides:".to_string());
    env_lines.push("# OPENCLAW_HOST=0.0.0.0".to_string());
    env_lines.push(format!("# OPENCLAW_PORT={}", port));

    let env_content = env_lines.join("\n");
    std::fs::write(env_path, env_content)?;
    println!("✓ Wrote {}", env_path.display());

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
