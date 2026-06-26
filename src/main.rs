// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand, ValueEnum};
use clap_i18n_richformatter::{
    ClapI18nRichFormatter, clap_i18n, init_clap_rich_formatter_localizer,
};
use env_logger::Env;
use log::LevelFilter;
use minijinja::{AutoEscape, Environment, Output, State, Value, context, value::ValueKind};
use reqwest::{ClientBuilder, Proxy};
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;
use teloxide::types::ParseMode;
use teloxide::utils::{html, markdown};
use tokio_graceful_shutdown::{SubsystemBuilder, Toplevel};

mod lang;
mod syslog;
mod telegram;
mod webhook;

#[derive(Clone, Debug, Serialize)]
pub struct MessageData {
    from: String,
    via: String,
    text: String,
    snr: Option<f32>,
    rssi: Option<i32>,
    hops_away: Option<i32>,
}

// Log level (global CLI arg + localized --help)
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum LogLevel {
    #[value(name = "error", help = fl!("log-level-error"))]
    Error,
    #[value(name = "warn", help = fl!("log-level-warn"))]
    Warn,
    #[value(name = "info", help = fl!("log-level-info"))]
    Info,
    #[value(name = "debug", help = fl!("log-level-debug"))]
    Debug,
    #[value(name = "trace", help = fl!("log-level-trace"))]
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
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
        bot_token: Option<String>,

        #[arg(
            long = "chat-id",
            env = "TELEGRAM_CHAT_ID",
            help = fl!("arg-chat-id"),
            allow_hyphen_values = true,
            value_delimiter = ',',
        )]
        chat_ids: Vec<i64>,

        #[arg(long, env = "WEBHOOK_URL")]
        #[arg(help = fl!("arg-webhook-url"))]
        webhook_url: Option<String>,

        #[arg(
            long,
            env = "MESH_DM",
            value_parser = clap::value_parser!(bool),
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
        )]
        #[arg(help = fl!("arg-dm"))]
        dm: bool,

        #[arg(long, env = "MESH_CHANNEL")]
        #[arg(help = fl!("arg-channel"))]
        channel: Option<u32>,

        #[arg(
            long,
            env = "TELEGRAM_TEMPLATE",
            default_value = "<b>{{ from }}</b> (via <i>{{ via }}</i>)\n<blockquote>{{ text }}</blockquote>"
        )]
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

        #[arg(long, env = "PROXY_URL")]
        #[arg(help = fl!("arg-proxy"))]
        proxy_url: Option<String>,

        #[arg(long, env = "TELEGRAM_API_SERVER")]
        #[arg(help = fl!("arg-api-server"))]
        api_server: Option<String>,
    },
}
// --- End Commands definition ---

pub static HELP_HEADING: LazyLock<String> = LazyLock::new(|| fl!("command-syslog"));
pub static ARG_HELP_HEADING: LazyLock<String> = LazyLock::new(|| fl!("arg-bot-token"));
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

#[derive(Parser)]
#[clap_i18n]
#[command(name = "emtt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = fl!("app-description"))]
#[command(long_about = fl!("app-long-description"))]
#[command(next_help_heading = &**ARG_HELP_HEADING)]
#[command(help_template = &*HELP_TEMPLATE)]
struct Cli {
    #[arg(
        long,
        short = 'l',
        env = "LOG_LEVEL",
        default_value_t = LogLevel::Debug,
        value_enum,
        help = fl!("arg-log-level"),
    )]
    log_level: LogLevel,

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
    bot_token: Option<String>,
    chat_ids: Vec<i64>,
    webhook_url: Option<String>,
    dm: bool,
    channel: Option<u32>,
    template: String,
    parse_mode: ParseModeOpt,
    syslog_host: String,
    syslog_port: u16,
    proxy_url: Option<String>,
    api_server: Option<String>,
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

fn telegram_escape_formatter(
    out: &mut Output,
    state: &State,
    value: &Value,
) -> Result<(), minijinja::Error> {
    if value.kind() == ValueKind::String {
        let s = value.as_str().unwrap();
        match state.auto_escape() {
            AutoEscape::Custom(tag) => match tag {
                "telegram_html" => {
                    write!(out, "{}", html::escape(s))?;
                }
                "telegram_markdown" => {
                    write!(out, "{}", markdown::escape(s))?;
                }
                _ => {
                    write!(out, "{}", s)?;
                }
            },
            AutoEscape::Html => {
                write!(out, "{}", html::escape(s))?;
            }
            _ => {
                write!(out, "{}", s)?;
            }
        }
    } else {
        write!(out, "{}", value)?;
    }
    Ok(())
}

fn create_template_env(parse_mode: ParseModeOpt) -> Environment<'static> {
    let mut env = Environment::new();
    let auto_escape = match parse_mode {
        ParseModeOpt::None => AutoEscape::None,
        ParseModeOpt::Html => AutoEscape::Custom("telegram_html"),
        ParseModeOpt::Markdown => AutoEscape::Custom("telegram_markdown"),
    };
    env.set_auto_escape_callback(move |_name| auto_escape);
    env.set_formatter(telegram_escape_formatter);
    env
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

#[cfg(windows)]
fn shutdown(exit_code: i32) -> ! {
    use std::io::{self, Write};

    // Only interact if the user can see the console.
    if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
        log::logger().flush();
        print!("\n{}", fl!("press-enter-close"));
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
    }

    std::process::exit(exit_code);
}

#[cfg(not(windows))]
fn shutdown(exit_code: i32) -> ! {
    std::process::exit(exit_code);
}

#[tokio::main]
async fn main() {
    // Initialize i18n first
    init_clap_rich_formatter_localizer();
    lang::init_localizer();

    // Parse CLI FIRST so we can use --log-level (CLI > LOG_LEVEL env > default debug)
    let cli = Cli::try_parse().unwrap_or_else(|e| {
        let e = e.apply::<ClapI18nRichFormatter>();
        let _ = e.print();
        shutdown(e.exit_code());
    });

    // Map LogLevel to LevelFilter
    let level_filter = match cli.log_level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };

    // Initialize logger with direct filter_level (respects CLI > env > debug) and LOG_STYLE env
    let env = Env::new().write_style_or("LOG_STYLE", "auto");
    env_logger::Builder::from_env(env)
        .filter_level(level_filter)
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .format_module_path(false)
        .format_target(false)
        .format_source_path(false)
        .init();

    match cli.command {
        Commands::Syslog {
            bot_token,
            chat_ids,
            webhook_url,
            dm,
            channel,
            template,
            parse_mode,
            syslog_host,
            syslog_port,
            proxy_url,
            api_server,
        } => {
            let template = unescape_template(template);
            let config = Config {
                bot_token,
                chat_ids,
                webhook_url,
                dm,
                channel,
                template,
                parse_mode,
                syslog_host,
                syslog_port,
                proxy_url,
                api_server,
            };

            let use_telegram = config.bot_token.is_some() && !config.chat_ids.is_empty();
            let use_webhook = config.webhook_url.is_some();

            if !use_telegram && !use_webhook {
                log::error!("{}", fl!("no-output-configured"));
                shutdown(2);
            }

            log::info!("{}", fl!("starting-syslog-mode"));

            if use_telegram {
                let ids = config
                    .chat_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                log::info!("{}", fl!("telegram-chat-id", chat_id = ids));
                log::info!(
                    "{}",
                    fl!(
                        "parse-mode",
                        parse_mode = format!("{:?}", config.parse_mode)
                    )
                );

                if let Some(ref server) = config.api_server {
                    log::info!("{}", fl!("bot-api-server-custom", url = server));
                } else {
                    log::info!("{}", fl!("bot-api-server-official"));
                }
            }

            log::info!("{}", fl!("forward-dm", dm = lang::localize_bool(config.dm)));

            if let Some(ch) = config.channel {
                log::info!("{}", fl!("forward-channel", channel = ch));
            } else {
                log::info!("{}", fl!("channel-disabled"));
            }

            if use_webhook {
                log::info!(
                    "{}",
                    fl!(
                        "webhook-enabled",
                        url = config.webhook_url.as_ref().unwrap()
                    )
                );
            } else {
                log::info!("{}", fl!("webhook-disabled"));
            }

            let mut client_builder = ClientBuilder::new();

            if let Some(proxy_url) = &config.proxy_url {
                log::info!("{}", fl!("proxy-enabled", url = proxy_url));

                match Proxy::all(proxy_url) {
                    Ok(proxy) => {
                        client_builder = client_builder.proxy(proxy);
                    }
                    Err(e) => {
                        log::error!(
                            "{}",
                            fl!("invalid-proxy-url", url = proxy_url, error = e.to_string())
                        );
                        shutdown(2);
                    }
                }
            }

            print_sponsorship_message();

            let http_client = match client_builder.build() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("{}", fl!("http-client-build-error", error = e.to_string()));
                    shutdown(1);
                }
            };

            let bot = if use_telegram {
                let token = config.bot_token.clone().unwrap();
                let bot_base = telegram::init_bot(token, http_client.clone());

                let bot = if let Some(server_url) = &config.api_server {
                    match reqwest::Url::parse(server_url) {
                        Ok(url) => bot_base.set_api_url(url),
                        Err(e) => {
                            log::error!(
                                "{}",
                                fl!(
                                    "invalid-api-server-url",
                                    url = server_url,
                                    error = e.to_string()
                                )
                            );
                            shutdown(2);
                        }
                    }
                } else {
                    bot_base
                };

                Some(bot)
            } else {
                None
            };

            let sender = {
                let bot = bot.clone();
                let chat_ids = config.chat_ids.clone();
                let template = config.template.clone();
                let parse_mode_opt = config.parse_mode;
                let webhook_url = config.webhook_url.clone();
                let use_telegram = use_telegram;
                let use_webhook = use_webhook;
                let http_client = http_client.clone();

                move |data: MessageData| {
                    let bot = bot.clone();
                    let chat_ids = chat_ids.clone();
                    let template = template.clone();
                    let parse_mode_opt = parse_mode_opt;
                    let webhook_url = webhook_url.clone();
                    let http_client = http_client.clone();

                    Box::pin(async move {
                        if use_telegram {
                            let env = create_template_env(parse_mode_opt);

                            let ctx = context! {
                                from => data.from,
                                via => data.via,
                                text => data.text,
                                snr => data.snr,
                                rssi => data.rssi,
                                hops_away => data.hops_away,
                            };

                            let rendered_result = env.render_str(&template, ctx);

                            if let Ok(rendered) = rendered_result {
                                let parse_mode = match parse_mode_opt {
                                    ParseModeOpt::None => None,
                                    ParseModeOpt::Html => Some(ParseMode::Html),
                                    ParseModeOpt::Markdown => Some(ParseMode::MarkdownV2),
                                };

                                for chat_id in &chat_ids {
                                    match telegram::send_message(
                                        bot.as_ref().unwrap(),
                                        *chat_id,
                                        &rendered,
                                        parse_mode,
                                    )
                                    .await
                                    {
                                        Err(err) => {
                                            log::warn!(
                                                "{}\n{}",
                                                fl!("failed-to-send", error = err.to_string()),
                                                fl!("message-content", content = rendered.clone())
                                            );
                                        }
                                        Ok(_) => {
                                            log::debug!(
                                                "{}",
                                                fl!(
                                                    "forwarded-to-telegram",
                                                    from = data.from.clone(),
                                                    message = rendered.clone()
                                                )
                                            );
                                        }
                                    }
                                }
                            } else if let Err(e) = rendered_result {
                                log::warn!("{}", fl!("failed-to-render", error = e.to_string()));
                            }
                        }

                        if use_webhook {
                            webhook::send_message(
                                &http_client,
                                webhook_url.as_ref().unwrap(),
                                &data,
                            )
                            .await;
                        }
                    }) as Pin<Box<dyn Future<Output = ()> + Send>>
                }
            };

            log::info!("{}", fl!("syslog-server"));

            let result = Toplevel::new(move |s| async move {
                s.start(SubsystemBuilder::new("syslog-server", move |subsys| {
                    syslog::run_server(subsys, config, sender)
                }));
            })
            .catch_signals()
            .handle_shutdown_requests(Duration::from_millis(2000))
            .await;

            match result {
                Ok(()) => shutdown(0),
                Err(err) => {
                    log::error!("{}", fl!("app-exit-error", error = err.to_string()));
                    shutdown(1);
                }
            }
        }
    }
}
