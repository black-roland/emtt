# Application metadata
app-name = Easy Mesh to Telegram
app-description = Easy to set up bridge from Meshtastic to Telegram
app-long-description =
  Easy Mesh to Telegram

  Forwards messages from Meshtastic network to Telegram chats.

  License and trademark details: https://github.com/black-roland/emtt

# Added for help template
usage = Usage

# Commands
command-syslog = Run in syslog mode

# Arguments
arg-bot-token = Telegram bot token
arg-chat-id = Telegram chat ID
arg-dm = Forward direct messages
arg-channel = Forward channel messages
arg-template = Message template
arg-parse-mode = Telegram message parse mode
arg-syslog-host = Syslog server host
arg-syslog-port = Syslog server port
arg-webhook-url = Webhook URL for forwarding messages (optional, in addition to or instead of Telegram)
arg-proxy = Proxy URL for outgoing connections (supports http://, https://, socks5:// and socks5h:// proxies)
arg-api-server = Telegram Bot API (optional; default = official api.telegram.org; for self-hosted use http://127.0.0.1:8081)
arg-log-level = Log level

# Boolean values
true-value = yes
false-value = no

# Parse modes
parse-mode-none = Plain text
parse-mode-html = HTML
parse-mode-markdown = Markdown

# Log levels (used in --help)
log-level-error = Error
log-level-warn = Warning
log-level-info = Information
log-level-debug = Debug (default)
log-level-trace = Trace

# Sponsorship messages
oss-sponsorship-message =
  If EMtT has been useful to you, consider buying the author a coffee! Your support is appreciated!

boosty-sponsorship-message =
  Thank you for supporting the project! Complete documentation is available on Boosty.

support-link = Support the project
documentation-link = Documentation

support-url = https://dalink.to/mansmarthome?utm_source=emtt&utm_medium=app&utm_campaign=oss
boosty-url = https://boosty.to/mansmarthome/posts/ca2ddb88-d808-419b-8faf-5d5619f66b95?utm_source=emtt&utm_medium=app&utm_campaign=boosty

# Log messages
starting-syslog-mode = Starting EMtT in syslog mode...
telegram-chat-id = Telegram chat ID: { $chat_id }
forward-dm = Forwarding direct messages: { $dm }
forward-channel = Forwarding channel messages: { $channel }
channel-disabled = Channel forwarding is disabled
parse-mode = Default parse mode: { $parse_mode }
syslog-server = Starting syslog server...
ignoring-range-test = Ignoring range test message from { $from } (ID: { $id })
no-handle-info = No sender information for message ID: { $id }
no-via-info = No gateway information for message ID: { $id }, via: { $via }
stale-handle-info = Stale sender information for message ID: { $id }
skipping-mqtt = Skipping MQTT-forwarded text for message ID: { $id }
stored-handle-info = Stored handle info for text msg ID: { $id }, via: { $via }, ch: { $ch }, to: { $to }, is_mqtt: { $is_mqtt }
ignoring-text-msg = Ignoring text message ID: { $id }, channel: { $ch }, to: { $to }
forwarded-to-telegram = Message forwarded to Telegram (from { $from }): { $message }
failed-to-render = Failed to render template: { $error }
failed-to-send = Failed to send message to Telegram: { $error }
failed-to-send-webhook = Failed to send message to webhook: { $error }
message-content = Message content: { $content }
processed-nodeinfo = Processed node info: { $longname } ({ $shortname }) - { $id }
syslog-binding = Syslog server listening on { $addr }
received-text-msg = Received text message from { $from } (ID: { $id }): { $text }
recv-error = Receive error: { $error }
invalid-utf8 = Invalid UTF-8 data from { $peer }
failed-to-parse-syslog = Failed to parse syslog message: { $error }, raw data: { $raw }
unhandled-syslog = Unhandled syslog message: { $message }
webhook-enabled = Webhook forwarding enabled to: { $url }
webhook-disabled = Webhook forwarding disabled
forwarded-to-webhook = Message forwarded to webhook (from { $from }): { $message }
no-output-configured = At least one output (Telegram or webhook) must be configured
proxy-enabled = Proxy enabled: { $url }
bot-api-server-official = Using official Telegram Bot API (api.telegram.org)
bot-api-server-custom   = Using custom Telegram Bot API server: { $url }
