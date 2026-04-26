# Фаза 10 — Длинный хвост (1.x)

**Длительность:** continuous
**Входные условия:** 1.0.0 в проде
**Релиз:** 1.1+

## Цель

Сегментированный backlog после релиза 1.0. Каждый пункт — отдельный мини-проект с собственным PR. Делается по приоритету пользовательских запросов и собственного интереса.

## Backlog

### 10.1. TOML-конфиг (REQ-200)

Альтернатива JSON для тех, кто не любит. Парсер через `toml`. Settings можно хранить в `~/.config/cchud/config.toml`, при отсутствии — fallback на `~/.claude/settings.json`.

**Когда:** если 5+ юзеров просят в issues.

### 10.2. Schemars / JSON-schema (REQ-201)

Авто-генерация JSON-schema из Rust-типов (как у starship). Schema-URL добавляется в `~/.claude/settings.json` (`$schema` поле) → IDE-валидация и автокомплит в VS Code.

**Когда:** до 1.5.

### 10.3. Auto-update (REQ-202)

Команда `cchud update`:
- Проверяет последний release на GitHub
- Скачивает бинарь, верифицирует SHA256
- Делает atomic-replace (на Unix через `rename(2)`, на Windows через `MoveFileEx` с DELAY_UNTIL_REBOOT fallback)

**Riski:** если бинарь сломан — пользователь без statusline. Решение: backup `cchud.bak`, при падении revert.

**Когда:** 1.1.

### 10.4. Прометей-метрики (REQ-203)

Опциональный режим `cchud --metrics localhost:9100` (или сокет). Экспозиция:
- `cchud_render_duration_ms`
- `cchud_widget_errors_total`
- `cchud_cache_hit_total`

**Когда:** если кто-то реально просит. Иначе никогда — это premature.

### 10.5. Reverse-port улучшений в ccstatusline upstream (REQ-204)

Хорошая практика: некоторые наши находки (например, bug в JSONL incremental parser) PR'ить обратно в TS upstream. Это:
- Карма
- Помогает пользователям, которые остаются на Node-версии
- Не даёт упустить, если upstream начнёт догонять

**Когда:** ad-hoc на каждый найденный шаринговый bug.

### 10.6. Виджеты для других CLI-агентов

Cursor, Cline, aider, opencode — все имеют statusline-механизмы или не имеют, но хотят. Не делать в основном проекте — отдельные форки/ветки. cchud-core как библиотека?

**Когда:** не раньше 2.0. Сначала зрелый core.

### 10.7. GUI-конфигуратор

Tauri / egui / iced. Mac-only — SwiftUI. Нет, скорее всего никогда — TUI в терминале достаточен для целевой аудитории.

**Когда:** never (но в backlog для прозрачности).

### 10.8. Windows package managers

- `winget install cchud`
- `scoop install cchud`

**Когда:** 1.2+.

### 10.9. AUR (Arch Linux)

PKGBUILD в AUR. Нужен maintainer-аккаунт.

**Когда:** community-PR welcomed.

### 10.10. Nix flake

`flake.nix` для NixOS-юзеров. Не сложно, но требует поддержки.

**Когда:** community-PR welcomed.

### 10.11. Дополнительные виджеты

Запросы, которые могут прийти:
- `Battery` — `starship-battery` крейт
- `Time` — простой clock
- `Hostname`, `Username`
- `Kubectl context`
- `AWS profile`
- `Docker container count`

Добавлять только при явном запросе (иначе scope creep).

### 10.12. Альтернативные сборки

- `cchud-mini` — feature-flag, без `tui`, `gix`, `sonic-rs`. Бинарь < 1 МБ.
- `cchud-portable` — single-file zip с конфигом для USB-юзеров.

**Когда:** низкий приоритет.

### 10.13. Migration tooling

- `cchud import --validate` — dry-run
- `cchud export --to ccstatusline` — обратная миграция
- `cchud diff` — сравнение текущего конфига с дефолтом

**Когда:** по запросу.

### 10.14. Языки и локализация

Все strings в виджетах — `&'static str`. Если будет запрос — i18n через `fluent-rs` или `rust-i18n`.

**Когда:** never (низкий приоритет, статусбар коротко).

### 10.15. Persistent daemon (если Anthropic откроет)

Issue #10162 закрыт NOT_PLANNED, но если Anthropic передумает:
- Архитектура `cchud-daemon` уже подготовлена через `RenderContext` lazy state
- Daemon mode: `cchud --daemon` слушает unix socket, рендерит на push
- Hot-reload settings.json через `notify` крейт

**Когда:** если откроют (мониторить issue).

## Принципы для backlog

1. **Не делать впрок.** Каждый пункт ждёт реального запроса или явной user-pain.
2. **Один пункт = один PR + версия.** Не бандлить 5 фич в один релиз.
3. **Performance budget неприкосновенен.** Никакая фича не должна ронять p95 > 5 мс.
4. **Backwards compat в 1.x.** Settings 1.0 должен работать в 1.5 без миграций (только additive поля).

## Метрики успеха продукта

После 1.0 отслеживать:
- GitHub stars, forks
- npm weekly downloads
- Brew installs (через `brew analytics`)
- Issues open/close ratio
- PR из community

Целевые числа на год после 1.0 (ambitious):
- 1k+ stars
- 100+ weekly npm installs
- 10+ внешних contributor'ов
- Упоминания в r/ClaudeAI, HN

## Когда закрывать проект

Если за 6 месяцев после 1.0:
- < 50 stars
- < 5 issues от внешних
- сам не пользуешься

→ это сигнал, что ниша мала / решение не востребовано. Архивировать с дисклеймером.

Если, наоборот, идёт рост — рассматривать 2.0 с reorganization (например, ядро как библиотека `cchud-core`, плагин-система для виджетов).
