# Application metadata
app-name = Easy Meshtastic to Telegram
app-description = Bridge between Meshtastic and Telegram
app-long-description =
  Easy Meshtastic to Telegram

  Forwards messages from Meshtastic network to Telegram chats.

  Project page: https://github.com/black-roland/emtt
  License: MPL 2.0

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

# Boolean values
true-value = yes
false-value = no

# Parse modes
parse-mode-none = Plain text
parse-mode-html = HTML
parse-mode-markdown = Markdown

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
syslog-listening = Listening for syslog messages on { $host }:{ $port }
syslog-server = Starting syslog server...
ignoring-range-test = Ignoring range test message from { $from } (ID: { $id })
no-handle-info = No sender information for message ID: { $id }
no-via-info = No gateway information for message ID: { $id }, via: { $via }
stale-handle-info = Stale sender information for message ID: { $id }
skipping-mqtt = Skipping MQTT-forwarded text for message ID: { $id }
ignoring-text-msg = Ignoring text message ID: { $id }, channel: { $ch }, to: { $to }
forwarded-to-telegram = Message forwarded to Telegram (from { $from }): { $message }
failed-to-render = Failed to render template: { $error }
failed-to-send = Failed to send message to Telegram: { $error }
message-content = Message content: { $content }
processed-nodeinfo = Processed node info: { $longname } ({ $shortname }) - { $id }
syslog-binding = Syslog server listening on { $addr }
received-text-msg = Received text message from { $from } (ID: { $id }): { $text }
recv-error = Receive error: { $error }
invalid-utf8 = Invalid UTF-8 data from { $peer }
failed-to-parse-syslog = Failed to parse syslog message: { $error }, raw data: { $raw }
unhandled-syslog = Unhandled syslog message: { $message }
