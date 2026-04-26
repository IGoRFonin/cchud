# Фаза 4 — Powerline и темизация

**Длительность:** 3–4 дня
**Входные условия:** Фазы 0–3
**Релиз:** 0.2.0

## Цель

Powerline-рендеринг визуально идентичен ccstatusline. Цветовые темы, hyperlinks (OSC 8), color sanitize/fallback, корректный расчёт ширины с учётом emoji/grapheme.

## Шаги

### 4.1. Renderer-абстракция

Расширить `src/render/mod.rs`:
```rust
pub enum Renderer {
    Plain(Plain),
    Powerline(Powerline),
}

impl Renderer {
    pub fn from_settings(s: &Settings) -> Self {
        match s.theme.kind {
            ThemeKind::Plain => Renderer::Plain(Plain::default()),
            ThemeKind::Powerline => Renderer::Powerline(Powerline::from(&s.theme)),
        }
    }
    pub fn render(&self, segments: &[Segment]) -> String { ... }
}

pub struct Segment {
    pub text: String,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
}
```

### 4.2. Powerline-рендерер

`src/render/powerline.rs` — порт `utils/powerline.ts`:
```rust
pub struct Powerline {
    pub separator_left: char,   // '\u{e0b0}'
    pub separator_right: char,  // '\u{e0b2}'
    pub theme: PowerlineTheme,
}

impl Powerline {
    pub fn render(&self, segments: &[Segment]) -> String {
        let mut out = String::new();
        let mut prev_bg = self.theme.terminal_bg;
        for seg in segments {
            let bg = seg.bg.unwrap_or(self.theme.default_bg);
            // separator с переходом prev_bg → bg
            out.push_str(&style(self.separator_left, prev_bg, bg));
            // содержимое
            out.push_str(&style(&seg.text, seg.fg.unwrap_or(self.theme.default_fg), bg));
            prev_bg = bg;
        }
        // финальный separator → terminal_bg
        out.push_str(&style(self.separator_left, prev_bg, self.theme.terminal_bg));
        out
    }
}
```

### 4.3. Темы

Порт `utils/powerline-theme-index.ts`. Решение: захардкодить ~5 тем в Rust (`themes::DRACULA`, `themes::SOLARIZED_DARK`, ...) + поддержка custom темы через TOML/JSON в settings.

```rust
pub static BUILTIN_THEMES: phf::Map<&'static str, PowerlineTheme> = phf::phf_map! {
    "default"        => themes::DEFAULT,
    "dracula"        => themes::DRACULA,
    "solarized-dark" => themes::SOLARIZED_DARK,
    "nord"           => themes::NORD,
    "gruvbox-dark"   => themes::GRUVBOX_DARK,
};
```

### 4.4. Color sanitize / fallback

Порт `utils/color-sanitize.ts`:
```rust
pub fn adapt_color(color: Color, level: ColorLevel) -> Color {
    match (color, level) {
        (Color::TrueColor(_), ColorLevel::TrueColor) => color,
        (Color::TrueColor(rgb), ColorLevel::Ansi256) => Color::Ansi256(rgb_to_ansi256(rgb)),
        (Color::TrueColor(rgb), ColorLevel::Basic) => Color::Basic(rgb_to_basic(rgb)),
        (_, ColorLevel::None) => Color::None,
        _ => color,
    }
}
```

Уровень определяется через `supports_color::on(supports_color::Stream::Stdout)`.

### 4.5. Hyperlinks (OSC 8)

`src/render/hyperlink.rs`:
```rust
pub fn link(text: &str, url: &str, supported: bool) -> String {
    if supported {
        format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text)
    } else {
        text.to_string()
    }
}

pub fn supports_hyperlinks() -> bool {
    // Детект через env: WT_SESSION (Windows Terminal), TERM_PROGRAM (iTerm/Hyper/...),
    // VTE_VERSION (GNOME Terminal), KITTY_WINDOW_ID, etc.
    std::env::var("TERM_PROGRAM").is_ok()
        || std::env::var("WT_SESSION").is_ok()
        || std::env::var("KITTY_WINDOW_ID").is_ok()
}
```

### 4.6. ANSI strip helper

Для `TerminalWidth`, Powerline-расчётов длины и тестов:
```rust
// src/utils/ansi.rs
pub fn strip(input: &str) -> String {
    // Использовать `anstyle-parse` Parser
}

pub fn visible_width(input: &str) -> usize {
    let stripped = strip(input);
    UnicodeWidthStr::width(stripped.as_str())
}
```

### 4.7. Visual snapshot-тесты

Сравнить вывод cchud Powerline с эталонными снапшотами от ccstatusline:
```bash
# в Фазе 0 при сборе семплов — параллельно сохранить выход ccstatusline:
for sample in benches/samples/*.json; do
  cat $sample | ccstatusline > "tests/golden/$(basename $sample .json).txt"
done
```

В тестах:
```rust
#[test]
fn powerline_matches_upstream() {
    insta::glob!("../benches/samples", "*.json", |path| {
        let payload = fs::read_to_string(path).unwrap();
        let cchud_out = run_cchud(&payload, &powerline_config());
        let golden = fs::read_to_string(format!("tests/golden/{}.txt", ...)).unwrap();
        assert_eq!(strip_ansi(&cchud_out), strip_ansi(&golden));
        // Точное побайтовое сравнение ANSI — too brittle. Сравниваем visible content.
    });
}
```

### 4.8. Релиз 0.2.0

CHANGELOG: Powerline + темы + hyperlinks + color fallback.

## Exit Criteria

- [ ] Powerline-конфиг рендерится визуально идентично ccstatusline (золотые тесты)
- [ ] 5+ встроенных тем работают
- [ ] OSC 8 hyperlinks работают в iTerm/Windows Terminal/Kitty, gracefully fallback в остальных
- [ ] `supports-color` корректно даунгрейдит цвета на TTY без truecolor
- [ ] `unicode-width` корректно считает ширину для emoji/CJK
- [ ] hyperfine: budget < 5 мс не нарушен (Powerline ~ Plain по скорости)
- [ ] Релиз 0.2.0

## Связи

- **Фаза 5/6/7** добавляют новые виджеты, которые автоматически рендерятся в Powerline
- **Фаза 8** TUI-конфигуратор отображает live preview Powerline
- **Фаза 9** README показывает скриншоты Powerline на 3 платформах

## Риски

- **Nerd Font glyphs не отображаются** на платформах без шрифта — `cchud doctor` (Фаза 9) детектит и говорит установить.
- **Windows Terminal с conhost legacy** — частая проблема, документировать `chcp 65001` и UTF-8 mode.
- **CJK-символы ломают Powerline-выравнивание** — обязательно `unicode_width::UnicodeWidthChar::width()`, не `chars().count()`.
- **Темы upstream меняются** — раз в полгода синхронизировать вручную, версионировать темы.
