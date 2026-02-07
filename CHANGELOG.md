# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0]

### Added

- Webhook output support (in addition to or instead of Telegram): `--webhook-url` / `WEBHOOK_URL`.

### Updated

- Simplify the default Telegram template by removing SNR, RSSI and hops away.

## [1.0.2] - 2026-01-19

### Fixed

- Eliminate the unwanted symbols in Windows console logs (disable the addition of Unicode bidirectional isolate characters).

## [1.0.1] - 2026-01-18

### Updated

- Periodic cleanup of orphaned handle infos to avoid unbounded memory usage growth during runtime.
- Syslog messages parsing performance optimizations.

## [1.0.0] - 2026-01-17

### Added

- Syslog server to accept node's logs.
- Syslog messages parsing.
- Telegram forwarding.
