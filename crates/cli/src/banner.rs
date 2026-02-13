//! ASCII art banners for Open Clanker CLI and gateway.

/// Robot ASCII art (user-provided).
pub const ROBOT: &str = r#"                 _______
               _/       \_
              / |       | \
             /  |__   __|  \
            |__/((o| |o))\__|
            |      | |      |
            |\     |_|     /|
            | \           / |
             \| /  ___  \ |/
              \ | / _ \ | /
               \_________/
                _|_____|_
           ____|_________|____
          /                   \  "#;

/// Open_Clanker in ASCII block font (user-provided).
pub const OPEN_CLANKER_LOGO: &str = r#"
        ______   _______   ________  __    __         ______   __         ______   __    __  __    __  ________  _______  
       /      \ /       \ /        |/  \  /  |       /      \ /  |       /      \ /  \  /  |/  |  /  |/        |/       \ 
      /$$$$$$  |$$$$$$$  |$$$$$$$$/ $$  \ $$ |      /$$$$$$  |$$ |      /$$$$$$  |$$  \ $$ |$$ | /$$/ $$$$$$$$/ $$$$$$$  |
      $$ |  $$ |$$ |__$$ |$$ |__    $$$  \$$ |      $$ |  $$/ $$ |      $$ |__$$ |$$$  \$$ |$$ |/$$/  $$ |__    $$ |__$$ |
      $$ |  $$ |$$    $$/ $$    |   $$$$  $$ |      $$ |      $$ |      $$    $$ |$$$$  $$ |$$  $$<   $$    |   $$    $$< 
      $$ |  $$ |$$$$$$$/  $$$$$/    $$ $$ $$ |      $$ |   __ $$ |      $$$$$$$$ |$$ $$ $$ |$$$$$  \  $$$$$/    $$$$$$$  |
      $$ \__$$ |$$ |      $$ |_____ $$ |$$$$ |      $$ \__/  |$$ |_____ $$ |  $$ |$$ |$$$$ |$$ |$$  \ $$ |_____ $$ |  $$ |
      $$    $$/ $$ |      $$       |$$ | $$$ |      $$    $$/ $$       |$$ |  $$ |$$ | $$$ |$$ | $$  |$$       |$$ |  $$ |
       $$$$$$/  $$/       $$$$$$$$/ $$/   $$/        $$$$$$/  $$$$$$$$/ $$/   $$/ $$/   $$/ $$/   $$/ $$$$$$$$/ $$/   $$/ "#;

/// Full welcome banner: robot + Open_Clanker logo.
pub fn welcome_banner() -> String {
    format!(
        "{}\n\n{}\n\n  🤖 High-Performance AI Assistant Framework 🤖\n  Built with Rust ❤️ | Spawned from S4MPL3BI4S 🌟",
        ROBOT,
        OPEN_CLANKER_LOGO
    )
}

/// Compact banner for gateway startup.
pub fn gateway_banner() -> String {
    format!(
        "{}\n{}\n  🤖 Open Clanker Gateway — Built with Rust",
        ROBOT,
        OPEN_CLANKER_LOGO
    )
}
