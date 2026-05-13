# Метаданные приложения
app-name = Easy Mesh to Telegram
app-description = Мост из Meshtastic в Telegram, который легко настроить
app-long-description =
  Easy Mesh to Telegram

  Пересылает сообщения из сети Meshtastic в Telegram-чаты.

  Информация о лицензиях и товарных знаках: https://github.com/black-roland/emtt

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
arg-parse-mode = Режим обработки Telegram-сообщений
arg-syslog-host = Хост сервера syslog
arg-syslog-port = Порт сервера syslog
arg-webhook-url = URL вебхука для пересылки сообщений (опционально, в дополнение или вместо Telegram)
arg-proxy = URL прокси для исходящих соединений (поддерживает http://, https://, socks5:// и socks5h:// прокси)
arg-api-server = Telegram Bot API (опционально; по умолчанию официальный api.telegram.org; для self-hosted: http://127.0.0.1:8081)
arg-log-level = Уровень логирования

# Булевы значения
true-value = да
false-value = нет

# Режимы обработки
parse-mode-none = Обычный текст
parse-mode-html = HTML
parse-mode-markdown = Markdown

# Уровни логирования (используются в --help)
log-level-error = Ошибка
log-level-warn = Предупреждение
log-level-info = Информация
log-level-debug = Отладка (по умолчанию)
log-level-trace = Трассировка

# Сообщения о поддержке проекта
oss-sponsorship-message =
  Если EMtT оказался полезен, вы можете угостить автора чашечкой кофе! Ваша поддержка очень важна!

boosty-sponsorship-message =
  Спасибо за поддержку проекта! Полная документация доступна на Boosty.

support-link = Поддержать проект
documentation-link = Документация

support-url = https://mansmarthome.info/donate/?utm_source=emtt&utm_medium=app&utm_campaign=oss
boosty-url = https://boosty.to/mansmarthome/posts/ca2ddb88-d808-419b-8faf-5d5619f66b95?utm_source=emtt&utm_medium=app&utm_campaign=boosty

# Логи
starting-syslog-mode = Запуск EMtT в режиме syslog...
telegram-chat-id = ID чата Telegram: { $chat_id }
forward-dm = Пересылка личных сообщений: { $dm }
forward-channel = Пересылка сообщений из канала: { $channel }
channel-disabled = Пересылка из канала отключена
parse-mode = Режим обработки по умолчанию: { $parse_mode }
syslog-server = Запуск сервера syslog...
ignoring-range-test = Игнорирую range test от { $from } (ID: { $id })
no-handle-info = Нет информации об отправителе для сообщения с ID: { $id }
no-via-info = Нет информации о шлюзе для сообщения с ID: { $id }, через: { $via }
stale-handle-info = Устаревшая информация об отправителе для сообщения с ID: { $id }
skipping-mqtt = Пропускаю сообщение, пересланное через MQTT, для сообщения с ID: { $id }
stored-handle-info = Сохранены метаданные для сообщения с ID: { $id }, через: { $via }, канал: { $ch }, адресат: { $to }, MQTT: { $is_mqtt }
ignoring-text-msg = Пропускаю текстовое сообщение с ID: { $id }, канал: { $ch }, получатель: { $to }
forwarded-to-telegram = Сообщение отправлено в Telegram (от { $from }): { $message }
failed-to-render = Ошибка рендеринга шаблона: { $error }
failed-to-send = Ошибка отправки в Telegram: { $error }
failed-to-send-webhook = Ошибка отправки в вебхук: { $error }
message-content = Содержимое сообщения: { $content }
processed-nodeinfo = Обработана информация об узле: { $longname } ({ $shortname }) - { $id }
syslog-binding = Сервер syslog ожидает подключений на { $addr }
received-text-msg = Получено текстовое сообщение от { $from } (ID: { $id }): { $text }
recv-error = Ошибка получения: { $error }
invalid-utf8 = Некорректные данные UTF-8 от { $peer }
failed-to-parse-syslog = Ошибка разбора syslog-сообщения: { $error }, исходные данные: { $raw }
unhandled-syslog = Необработанное syslog-сообщение: { $message }
webhook-enabled = Пересылка в вебхук включена для: { $url }
webhook-disabled = Вебхук отключён
forwarded-to-webhook = Сообщение отправлено в вебхук (от { $from }): { $message }
no-output-configured = Настройте пересылку в Telegram или вебхук
proxy-enabled = Прокси включён: { $url }
bot-api-server-official = Используется официальный Telegram Bot API (api.telegram.org)
bot-api-server-custom   = Используется собственный сервер Telegram Bot API: { $url }
