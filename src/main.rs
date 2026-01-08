// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};

mod syslog;
mod telegram;
// mod mqtt;  // Uncomment when implementing MQTT

#[derive(Parser)]
#[command(name = "emtt")]
#[command(about = "Easy Meshtastic to Telegram bridge")]
struct Cli {
    #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
    bot_token: String,

    #[arg(long, env = "TELEGRAM_CHAT_ID")]
    chat_id: String,

    #[arg(long, env = "EMTT_CHANNEL", default_value = "0")]
    channel: u32,

    #[arg(long, env = "SYSLOG_HOST", default_value = "0.0.0.0")]
    syslog_host: String,

    #[arg(long, env = "SYSLOG_PORT", default_value = "50514")]
    syslog_port: u16,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the syslog server (MVP)
    Syslog,
    // Future: Mqtt { ... args ... },
}

#[derive(Clone)]
struct Config {
    bot_token: String,
    chat_id: String,
    channel: u32,
    syslog_host: String,
    syslog_port: u16,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let config = Config {
        bot_token: cli.bot_token,
        chat_id: cli.chat_id,
        channel: cli.channel,
        syslog_host: cli.syslog_host,
        syslog_port: cli.syslog_port,
    };

    let bot = telegram::init_bot(&config);

    match cli.command {
        Commands::Syslog => {
            log::info!("Launching syslog server...");
            syslog::run_server(&config).await;
        }
        // Future: Commands::Mqtt => mqtt::run_client(&config).await,
    }
}
