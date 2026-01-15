# Метаданные приложения
app-name = Easy Meshtastic to Telegram
app-description = Мост Easy Meshtastic to Telegram
app-long-description =
  Easy Meshtastic to Telegram

  Страница проекта: https://github.com/black-roland/emtt
  Лицензия: MPL 2.0

# Добавлено для шаблона справки
usage = Использование

# Команды
command-syslog = Запуск в режиме syslog

# Аргументы
arg-bot-token = Токен бота Telegram
arg-chat-id = ID чата Telegram
arg-dm = Пересылать личные сообщения
arg-channel = Пересылать сообщения из каналов
arg-template = Шаблон сообщения
arg-parse-mode = Режим парсинга сообщений
arg-syslog-host = Хост сервера syslog
arg-syslog-port = Порт сервера syslog

# Булевы значения
true-value = да
false-value = нет

# Режимы парсинга
parse-mode-none = Без форматирования
parse-mode-html = HTML
parse-mode-markdown = Markdown

# Сообщения о спонсорстве
oss-sponsorship-message =
    Если EMtT оказался полезным, вы можете угостить автора чашечкой кофе. Ваша благодарность ценится!

boosty-sponsorship-message =
    Спасибо за поддержку проекта! Полная документация доступна на Boosty.

support-link = Поддержать проект
documentation-link = Документация

# UTM-enhanced URLs
support-url = https://mansmarthome.info/donate/?utm_source=emtt&utm_medium=app&utm_campaign=oss
boosty-url = https://boosty.to/mansmarthome?utm_source=emtt&utm_medium=app&utm_campaign=boosty

# Логи
starting-syslog-mode = Запуск EMtT в режиме syslog...
telegram-chat-id = ID чата Telegram: { $chat_id }
forward-dm = Пересылка личных сообщений: { $dm }
forward-channel = Пересылка сообщений из канала: { $channel }
channel-disabled = Пересылка из канала отключена
parse-mode = Режим парсинга по умолчанию: { $parse_mode }
syslog-listening = Syslog слушает на { $host }:{ $port }
syslog-server = Запуск сервера syslog...
ignoring-range-test = Игнорируем range test от { $from }, id { $id }
no-handle-info = Нет информации об отправителе для сообщения с id: { $id }
no-via-info = Нет информации о шлюзе для сообщения с id: { $id }, через: { $via }
stale-handle-info = Устаревшая информация об отправителе для сообщения с id: { $id }
skipping-mqtt = Пропуск текста, пересланного через MQTT, для сообщения с id: { $id }
ignoring-text-msg = Пропуск сообщения с id: { $id }, канал: { $ch }, получатель: { $to }
forwarded-to-telegram = Сообщение переслано в Telegram (от { $from }): { $message }
failed-to-render = Ошибка рендеринга шаблона: { $error }
failed-to-send = Ошибка отправки сообщения в Telegram: { $error }
message-content = Содержимое сообщения: { $content }
processed-nodeinfo = Обработана информация об узле: { $longname } ({ $shortname }) - { $id }
syslog-binding = Сервер syslog ожидает подключений на { $addr }
received-text-msg = Получено текстовое сообщение от { $from }, id { $id }: { $text }
recv-error = Ошибка приёма: { $error }
invalid-utf8 = Некорректный UTF-8 от { $peer }
failed-to-parse-syslog = Ошибка разбора syslog: { $error }, сырые данные: { $raw }
unhandled-syslog = Необработанное syslog-сообщение: { $message }
