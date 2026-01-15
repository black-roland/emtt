// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand, ValueEnum};
use clap_i18n_richformatter::{clap_i18n, ClapI18nRichFormatter, init_clap_rich_formatter_localizer};
use env_logger::Env;
use minijinja::{context, Environment};
use std::future::Future;
use std::pin::Pin;
use std::sync::LazyLock;
use teloxide::types::ParseMode;

mod lang;
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

// --- Define Commands enum BEFORE Cli struct ---
#[derive(Subcommand)]
enum Commands {
    /// Run in syslog mode - Help text will be localized via fl! macro
    #[command(about = fl!("command-syslog"))]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Syslog {
        #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
        #[arg(help = fl!("arg-bot-token"))]
        bot_token: String,

        #[arg(long, env = "TELEGRAM_CHAT_ID")]
        #[arg(help = fl!("arg-chat-id"))]
        chat_id: i64,

        #[arg(long, env = "MESH_DM", default_value = "true")]
        #[arg(help = fl!("arg-dm"))]
        dm: bool,

        #[arg(long, env = "MESH_CHANNEL")]
        #[arg(help = fl!("arg-channel"))]
        channel: Option<u32>,

        #[arg(long, env = "TELEGRAM_TEMPLATE", default_value = "<b>{{ from | e }}</b> (via <i>{{ via | e }}</i>)\nSNR: {{ snr | default(\"N/A\") }} | RSSI: {{ rssi | default(\"N/A\") }} | Hops: {{ hops_away | default(\"N/A\") }}\n<blockquote>{{ text | e }}</blockquote>")]
        #[arg(help = fl!("arg-template"))]
        template: String,

        #[arg(long, env = "TELEGRAM_PARSE_MODE", default_value = "html")]
        #[arg(help = fl!("arg-parse-mode"))]
        parse_mode: ParseModeOpt,

        #[arg(long, env = "SYSLOG_HOST", default_value = "0.0.0.0")]
        #[arg(help = fl!("arg-syslog-host"))]
        syslog_host: String,

        #[arg(long, env = "SYSLOG_PORT", default_value = "50514")]
        #[arg(help = fl!("arg-syslog-port"))]
        syslog_port: u16,
    },
}
// --- End Commands definition ---

pub static HELP_HEADING: LazyLock<String> = LazyLock::new(|| fl!("command-syslog")); // Or another appropriate heading key
pub static ARG_HELP_HEADING: LazyLock<String> = LazyLock::new(|| fl!("arg-bot-token")); // Use a general heading or specific one if needed
pub static HELP_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    format!(
        "\
{{before-help}}{{about-with-newline}}

{}{}:{} {{usage}}

{{all-args}}{{after-help}}\
        ",
        clap::builder::Styles::default().get_usage().render(),
        fl!("usage"),
        clap::builder::Styles::default().get_usage().render_reset()
    )
});

fn localize_bool(value: bool) -> String {
    if value {
        fl!("true-value").to_string()
    } else {
        fl!("false-value").to_string()
    }
}

#[derive(Parser)]
#[clap_i18n] // Apply this to the main struct
#[command(name = "emtt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = fl!("app-description"))]
#[command(long_about = fl!("app-long-description"))]
#[command(next_help_heading = &**ARG_HELP_HEADING)]
#[command(help_template = &*HELP_TEMPLATE)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ParseModeOpt {
    #[value(name = "none", help = fl!("parse-mode-none"))]
    None,
    #[value(name = "html", help = fl!("parse-mode-html"))]
    Html,
    #[value(name = "markdown", help = fl!("parse-mode-markdown"))]
    Markdown,
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

fn print_sponsorship_message() {
    println!();

    #[cfg(feature = "boosty")]
    {
        log::info!("{}", fl!("boosty-sponsorship-message"));
        log::info!("{}: {}", fl!("documentation-link"), fl!("boosty-url"));
    }

    #[cfg(not(feature = "boosty"))]
    {
        log::info!("{}", fl!("oss-sponsorship-message"));
        log::info!("{}: {}", fl!("support-link"), fl!("support-url"));
    }

    println!();
}

#[tokio::main]
async fn main() {
    // Initialize i18n first
    init_clap_rich_formatter_localizer();
    lang::init_localizer();

    let env = Env::new()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "auto");
    env_logger::Builder::from_env(env)
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .format_module_path(false)
        .format_target(false)
        .format_source_path(false)
        .init();

    // Use the i18n-aware parsing method
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let e = e.apply::<ClapI18nRichFormatter>();
            e.exit();
        }
    };

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

            log::info!("{}", fl!("starting-syslog-mode"));
            log::info!("{}", fl!("telegram-chat-id", chat_id = config.chat_id));

            log::info!("{}", fl!("forward-dm", dm = localize_bool(config.dm)));

            if let Some(ch) = config.channel {
                log::info!("{}", fl!("forward-channel", channel = ch));
            } else {
                log::info!("{}", fl!("channel-disabled"));
            }

            log::info!("{}", fl!("parse-mode", parse_mode = format!("{:?}", config.parse_mode)));

            print_sponsorship_message();

            log::info!("{}", fl!("syslog-listening", host = config.syslog_host.clone(), port = config.syslog_port));

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
                                log::warn!("{}", fl!("failed-to-render", error = e.to_string()));
                                return;
                            }
                        };

                        let parse_mode = match parse_mode_opt {
                            ParseModeOpt::None => None,
                            ParseModeOpt::Html => Some(ParseMode::Html),
                            ParseModeOpt::Markdown => Some(ParseMode::MarkdownV2),
                        };

                        if let Err(err) = telegram::send_message(&bot, chat_id, &rendered, parse_mode).await {
                            log::warn!(
                                "{}\n{}",
                                fl!("failed-to-send", error = err.to_string()),
                                fl!("message-content", content = rendered)
                            );
                        } else {
                            log::debug!("{}", fl!("forwarded-to-telegram", from = data.from, message = rendered));
                        }
                    }) as Pin<Box<dyn Future<Output = ()> + Send>>
                }
            };

            log::info!("{}", fl!("syslog-server"));
            syslog::run_server(&config, sender).await;
        }
    }
}
