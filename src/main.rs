// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use std::future::Future;
use std::pin::Pin;

mod syslog;
mod telegram;

#[derive(Parser)]
#[command(name = "emtt")]
#[command(about = "Easy Meshtastic to Telegram bridge")]
#[command(long_about = "Easy Meshtastic to Telegram bridge\n\nProject page: https://github.com/black-roland/emtt\nDocumentation: https://boosty.to/mansmarthome\nLicense: MPL 2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the syslog server (MVP)
    Syslog {
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
    },
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

    match cli.command {
        Commands::Syslog {
            bot_token,
            chat_id,
            channel,
            syslog_host,
            syslog_port,
        } => {
            let config = Config {
                bot_token,
                chat_id: chat_id.clone(),
                channel,
                syslog_host,
                syslog_port,
            };

            let bot = telegram::init_bot(&config);

            let sender = move |msg: String| {
                let bot = bot.clone();
                let chat_id = chat_id.clone();
                Box::pin(async move {
                    if let Err(err) = telegram::send_message(&bot, &chat_id, &msg).await {
                        log::warn!("Failed to send message to Telegram: {}", err);
                    } else {
                        log::info!("Forwarded message to Telegram: {}", msg);
                    }
                }) as Pin<Box<dyn Future<Output = ()> + Send>>
            };

            log::info!("Launching syslog server...");
            syslog::run_server(&config, sender).await;
        }
    }
}
