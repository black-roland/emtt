# Application metadata
app-name = Easy Meshtastic to Telegram
app-description = Easy Meshtastic to Telegram bridge
app-long-description =
  Easy Meshtastic to Telegram

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
arg-parse-mode = Parse mode for messages
arg-syslog-host = Syslog server host
arg-syslog-port = Syslog server port

# Boolean values
true-value = yes
false-value = no

# Parse modes
parse-mode-none = None
parse-mode-html = HTML
parse-mode-markdown = Markdown

# Log messages
starting-syslog-mode = Starting EMtT in syslog mode
telegram-chat-id = Telegram chat ID: { $chat_id }
forward-dm = Forward direct messages: { $dm }
forward-channel = Forward channel messages: { $channel }
channel-disabled = Channel forwarding disabled
parse-mode = Default parse mode: { $parse_mode }
syslog-listening = Syslog listening on { $host }:{ $port }
syslog-server = Launching syslog server...
ignoring-range-test = Ignoring range test message from { $from } id { $id }
no-handle-info = No handle info for text msg id: { $id }
no-via-info = No via info for text msg id: { $id }, via: { $via }
stale-handle-info = Stale handle info for text msg id: { $id }
skipping-mqtt = Skipping MQTT-forwarded text for msg id: { $id }
ignoring-text-msg = Ignoring text msg id: { $id }, ch: { $ch }, to: { $to }
forwarded-to-telegram = Forwarded message to Telegram (from { $from }): { $message }
failed-to-render = Failed to render template: { $error }
failed-to-send = Failed to send message to Telegram: { $error }
message-content = Message content: { $content }
processed-nodeinfo = Processed nodeinfo: { $longname } ({ $shortname }) - { $id }
syslog-binding = Syslog server listening on { $addr }
recv-error = Recv error: { $error }
invalid-utf8 = Invalid UTF-8 from { $peer }
failed-to-parse-syslog = Failed to parse syslog: { $error }, raw: { $raw }
unhandled-syslog = Unhandled syslog message: { $message }
