// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand, ValueEnum};
use minijinja::{context, Environment};
use std::future::Future;
use std::pin::Pin;
use teloxide::types::ParseMode;

mod syslog;
mod telegram;

#[derive(Clone, Debug)]
pub struct MessageData {
    from: String,
    via: String,
    text: String,
    snr: Option<f32>,
    rssi: Option<i32>,
    hops_away: Option<i32>,
}

#[derive(Parser)]
#[command(name = "emtt")]
#[command(about = "Easy Meshtastic to Telegram bridge")]
#[command(long_about = "Easy Meshtastic to Telegram bridge\n\nProject page: https://github.com/black-roland/emtt\nDocumentation: https://boosty.to/mansmarthome\nLicense: MPL 2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ParseModeOpt {
    #[value(name = "none")]
    None,
    #[value(name = "html")]
    Html,
    #[value(name = "markdown")]
    Markdown,
}

#[derive(Subcommand)]
enum Commands {
    Syslog {
        #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
        bot_token: String,

        #[arg(long, env = "TELEGRAM_CHAT_ID")]
        chat_id: i64,

        #[arg(long, env = "MESH_DM", default_value = "true")]
        dm: bool,

        #[arg(long, env = "MESH_CHANNEL")]
        channel: Option<u32>,

        #[arg(long, env = "TELEGRAM_TEMPLATE", default_value = "<b>{{ from | e }}</b> (via <i>{{ via | e }}</i>)\nSNR: {{ snr | default(\"N/A\") }} | RSSI: {{ rssi | default(\"N/A\") }} | Hops: {{ hops_away | default(\"N/A\") }}\n<blockquote>{{ text | e }}</blockquote>")]
        template: String,

        #[arg(long, env = "TELEGRAM_PARSE_MODE", default_value = "html")]
        parse_mode: ParseModeOpt,

        #[arg(long, env = "SYSLOG_HOST", default_value = "0.0.0.0")]
        syslog_host: String,

        #[arg(long, env = "SYSLOG_PORT", default_value = "50514")]
        syslog_port: u16,
    },
}

#[derive(Clone)]
struct Config {
    bot_token: String,
    chat_id: i64,
    dm: bool,
    channel: Option<u32>,
    template: String,
    parse_mode: ParseModeOpt,
    syslog_host: String,
    syslog_port: u16,
}

fn unescape_template(s: String) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.peek() {
                match next {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    _ => {
                        result.push('\\');
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Syslog {
            bot_token,
            chat_id,
            dm,
            channel,
            template,
            parse_mode,
            syslog_host,
            syslog_port,
        } => {
            let template = unescape_template(template);
            let config = Config {
                bot_token,
                chat_id,
                dm,
                channel,
                template,
                parse_mode,
                syslog_host,
                syslog_port,
            };

            let bot = telegram::init_bot(&config);

            let sender = {
                let bot = bot.clone();
                let chat_id = config.chat_id;
                let template = config.template.clone();
                let parse_mode_opt = config.parse_mode;
                move |data: MessageData| {
                    let bot = bot.clone();
                    let chat_id = chat_id;
                    let template = template.clone();
                    let parse_mode_opt = parse_mode_opt;
                    Box::pin(async move {
                        let env = Environment::new();
                        let ctx = context! {
                            from => data.from,
                            via => data.via,
                            text => data.text,
                            snr => data.snr,
                            rssi => data.rssi,
                            hops_away => data.hops_away,
                        };
                        let rendered = match env.render_str(&template, ctx) {
                            Ok(s) => s,
                            Err(e) => {
                                log::warn!("Failed to render template: {}", e);
                                return;
                            }
                        };
                        let parse_mode = match parse_mode_opt {
                            ParseModeOpt::None => None,
                            ParseModeOpt::Html => Some(ParseMode::Html),
                            ParseModeOpt::Markdown => Some(ParseMode::MarkdownV2),
                        };
                        if let Err(err) = telegram::send_message(&bot, chat_id, &rendered, parse_mode).await {
                            log::warn!("Failed to send message to Telegram: {}\nMessage content: {}", err, rendered);
                        } else {
                            log::info!("Forwarded message to Telegram: {}", rendered);
                        }
                    }) as Pin<Box<dyn Future<Output = ()> + Send>>
                }
            };

            log::info!("Launching syslog server...");
            syslog::run_server(&config, sender).await;
        }
    }
}
