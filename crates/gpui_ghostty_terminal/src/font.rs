use crate::TerminalConfig;

/// Returns the platform-specific default font family for terminals.
fn default_font_family() -> &'static str {
    if cfg!(target_os = "macos") {
        "Menlo"
    } else if cfg!(target_os = "windows") {
        "Consolas"
    } else {
        "DejaVu Sans Mono"
    }
}

/// Returns font fallbacks for terminal rendering.
fn terminal_font_fallbacks() -> gpui::FontFallbacks {
    gpui::FontFallbacks::from_fonts(vec![
        "SF Mono".to_string(),
        "Menlo".to_string(),
        "Monaco".to_string(),
        "Consolas".to_string(),
        "Cascadia Mono".to_string(),
        "DejaVu Sans Mono".to_string(),
        "Noto Sans Mono".to_string(),
        "JetBrains Mono".to_string(),
        "Fira Mono".to_string(),
        "Sarasa Mono SC".to_string(),
        "Sarasa Term SC".to_string(),
        "Sarasa Mono J".to_string(),
        "Noto Sans Mono CJK SC".to_string(),
        "Noto Sans Mono CJK JP".to_string(),
        "Source Han Mono SC".to_string(),
        "WenQuanYi Zen Hei Mono".to_string(),
        "Apple Color Emoji".to_string(),
        "Noto Color Emoji".to_string(),
        "Segoe UI Emoji".to_string(),
    ])
}

/// Returns the default terminal font (platform-specific).
pub fn default_terminal_font() -> gpui::Font {
    let mut font = gpui::font(default_font_family());
    font.fallbacks = Some(terminal_font_fallbacks());
    font
}

/// Returns a terminal font based on the provided configuration.
///
/// If `config.font_family` is set, uses that font family; otherwise uses the platform default.
pub fn terminal_font(config: &TerminalConfig) -> gpui::Font {
    let family = match &config.font_family {
        Some(f) => f.clone(),
        None => default_font_family().to_string(),
    };
    let mut font = gpui::font(family);
    font.fallbacks = Some(terminal_font_fallbacks());
    font
}

pub fn default_terminal_font_features() -> gpui::FontFeatures {
    use std::sync::Arc;
    gpui::FontFeatures(Arc::new(vec![
        ("calt".to_string(), 0),
        ("liga".to_string(), 0),
        ("kern".to_string(), 0),
    ]))
}
