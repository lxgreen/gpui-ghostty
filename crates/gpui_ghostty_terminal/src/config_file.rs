//! Ghostty config file parser.
//!
//! Loads configuration from `~/.config/ghostty/config` (or `$XDG_CONFIG_HOME/ghostty/config`)
//! using the Ghostty key-value format. Also supports loading themes from theme files.

use std::fs;
use std::io;
use std::path::PathBuf;

use ghostty_vt::{CursorStyle, Rgb};

use crate::TerminalConfig;
use crate::config::{CursorColor, DEFAULT_PALETTE};

/// Errors that can occur when loading a config file.
#[derive(Debug)]
pub enum ConfigError {
    /// Config file not found at any standard location.
    NotFound,
    /// I/O error reading the config file.
    Io(io::Error),
    /// Parse error on a specific line.
    Parse { line: usize, message: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound => write!(f, "config file not found"),
            ConfigError::Io(e) => write!(f, "I/O error: {}", e),
            ConfigError::Parse { line, message } => {
                write!(f, "parse error on line {}: {}", line, message)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

/// Load configuration from the default Ghostty config file locations.
///
/// Searches in order:
/// 1. `$XDG_CONFIG_HOME/ghostty/config` (if `XDG_CONFIG_HOME` is set)
/// 2. `~/.config/ghostty/config`
///
/// Returns `Err(ConfigError::NotFound)` if no config file exists.
pub fn load_config() -> Result<TerminalConfig, ConfigError> {
    let path = find_config_file().ok_or(ConfigError::NotFound)?;
    load_config_from_path(&path)
}

/// Load configuration from a specific file path.
pub fn load_config_from_path(path: &std::path::Path) -> Result<TerminalConfig, ConfigError> {
    let contents = fs::read_to_string(path)?;
    parse_config(&contents)
}

/// Reload theme colors for a config based on explicit dark mode setting.
///
/// This is useful when the system appearance changes and you want to switch
/// between dark/light theme variants. Only affects configs with a `theme_spec`
/// that contains `dark:` or `light:` variants.
///
/// Returns `true` if the theme was reloaded, `false` if no theme spec exists
/// or the theme spec doesn't have dark/light variants.
pub fn reload_theme_for_appearance(config: &mut TerminalConfig, is_dark: bool) -> bool {
    let Some(theme_spec) = config.theme_spec.clone() else {
        return false;
    };

    // Only reload if the theme spec has dark/light variants
    if !theme_spec.contains(':') {
        return false;
    }

    // Resolve the theme name for the given appearance
    let theme_name = resolve_theme_name_for_appearance(&theme_spec, is_dark);
    let Some(theme_name) = theme_name else {
        return false;
    };
    // Clone to avoid borrow issues
    let theme_name = theme_name.to_string();

    // Reset theme-related fields before applying new theme
    config.default_fg = Rgb {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    };
    config.default_bg = Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    };
    config.palette = None;
    config.selection_background = None;
    config.selection_foreground = None;
    config.cursor_color = CursorColor::CellForeground;
    config.cursor_text = CursorColor::CellBackground;

    // Load the theme
    load_theme(config, &theme_name).is_ok()
}

/// Resolve theme name for a specific appearance (dark or light).
fn resolve_theme_name_for_appearance(theme_spec: &str, is_dark: bool) -> Option<&str> {
    let theme_spec = theme_spec.trim();
    if theme_spec.is_empty() {
        return None;
    }

    // Check for dark:/light: syntax
    if theme_spec.contains(':') {
        for part in theme_spec.split(',') {
            let part = part.trim();
            if let Some(name) = part.strip_prefix("dark:")
                && is_dark
            {
                return Some(name.trim());
            } else if let Some(name) = part.strip_prefix("light:")
                && !is_dark
            {
                return Some(name.trim());
            }
        }
        // If no matching variant found, try to use the first one
        for part in theme_spec.split(',') {
            let part = part.trim();
            if let Some(name) = part
                .strip_prefix("dark:")
                .or_else(|| part.strip_prefix("light:"))
            {
                return Some(name.trim());
            }
        }
        None
    } else {
        // Simple theme name - no appearance-specific variant
        Some(theme_spec)
    }
}

/// Find the config file path following Ghostty's search order.
fn find_config_file() -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME first
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("ghostty/config");
        if path.exists() {
            return Some(path);
        }
    }

    // Fall back to ~/.config/ghostty/config
    if let Some(home) = home_dir() {
        let path = home.join(".config/ghostty/config");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Get the user's home directory.
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Find a theme file by name.
///
/// Searches in order:
/// 1. `$XDG_CONFIG_HOME/ghostty/themes/{name}` (if `XDG_CONFIG_HOME` is set)
/// 2. `~/.config/ghostty/themes/{name}`
/// 3. `/Applications/Ghostty.app/Contents/Resources/ghostty/themes/{name}` (macOS)
/// 4. `/usr/share/ghostty/themes/{name}` (Linux system-wide)
fn find_theme_file(name: &str) -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME first
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("ghostty/themes").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Try ~/.config/ghostty/themes/
    if let Some(home) = home_dir() {
        let path = home.join(".config/ghostty/themes").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // macOS: Try Ghostty.app bundle
    #[cfg(target_os = "macos")]
    {
        let path =
            PathBuf::from("/Applications/Ghostty.app/Contents/Resources/ghostty/themes").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Linux: Try system-wide location
    #[cfg(target_os = "linux")]
    {
        let path = PathBuf::from("/usr/share/ghostty/themes").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Load and apply a theme by name.
///
/// Searches for themes in this order:
/// 1. Embedded themes (bundled in the binary)
/// 2. User config directory (`~/.config/ghostty/themes/`)
/// 3. System locations (Ghostty.app bundle, `/usr/share/ghostty/themes/`)
///
/// Returns `Ok(())` if the theme was loaded successfully, or `Err` if the theme
/// file was not found or could not be parsed.
fn load_theme(config: &mut TerminalConfig, name: &str) -> Result<(), ConfigError> {
    // First, try embedded themes (no filesystem access needed)
    if let Some(contents) = crate::themes::get_embedded_theme(name) {
        return apply_theme_contents(config, contents);
    }

    // Fall back to filesystem-based themes
    let path = find_theme_file(name).ok_or(ConfigError::NotFound)?;
    let contents = fs::read_to_string(&path)?;
    apply_theme_contents(config, &contents)
}

/// Apply theme file contents to a config.
fn apply_theme_contents(config: &mut TerminalConfig, contents: &str) -> Result<(), ConfigError> {
    for (line_num, line) in contents.lines().enumerate() {
        let line_num = line_num + 1;

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = parse_line(trimmed) {
            apply_theme_option(config, key, value, line_num)?;
        }
    }
    Ok(())
}

/// Apply a single theme option to the config.
/// Theme files support a subset of config options (colors only).
fn apply_theme_option(
    config: &mut TerminalConfig,
    key: &str,
    value: &str,
    line_num: usize,
) -> Result<(), ConfigError> {
    match key {
        "foreground" => {
            if !value.is_empty() {
                config.default_fg = parse_color(value).ok_or_else(|| ConfigError::Parse {
                    line: line_num,
                    message: format!("invalid color: {}", value),
                })?;
            }
        }
        "background" => {
            if !value.is_empty() {
                config.default_bg = parse_color(value).ok_or_else(|| ConfigError::Parse {
                    line: line_num,
                    message: format!("invalid color: {}", value),
                })?;
            }
        }
        "cursor-color" => {
            if !value.is_empty() {
                config.cursor_color =
                    parse_cursor_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid cursor color: {}", value),
                    })?;
            }
        }
        "cursor-text" => {
            if !value.is_empty() {
                config.cursor_text =
                    parse_cursor_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid cursor text color: {}", value),
                    })?;
            }
        }
        "selection-background" => {
            if value.is_empty() {
                config.selection_background = None;
            } else {
                config.selection_background =
                    Some(parse_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid selection background color: {}", value),
                    })?);
            }
        }
        "selection-foreground" => {
            if value.is_empty() {
                config.selection_foreground = None;
            } else {
                config.selection_foreground =
                    Some(parse_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid selection foreground color: {}", value),
                    })?);
            }
        }
        "palette" => {
            // Format: "palette = N=#RRGGBB" where N is 0-15
            if let Some((index, color)) = parse_palette_entry(value)
                && index < 16
            {
                let palette = config.palette.get_or_insert(DEFAULT_PALETTE);
                palette[index] = color;
            }
            // Invalid palette entries are silently ignored
        }
        // Unknown keys in theme files are silently ignored
        _ => {}
    }
    Ok(())
}

/// Parse a palette entry value.
/// Format: "N=#RRGGBB" where N is the palette index (0-15).
fn parse_palette_entry(value: &str) -> Option<(usize, Rgb)> {
    let (index_str, color_str) = value.split_once('=')?;
    let index: usize = index_str.trim().parse().ok()?;
    let color = parse_color(color_str.trim())?;
    Some((index, color))
}

/// Parse a theme specification.
///
/// Supports:
/// - Simple theme name: `"Catppuccin Mocha"`
/// - Dark/light variants: `"dark:Catppuccin Mocha,light:Catppuccin Latte"`
///
/// Returns the theme name to load based on system appearance.
fn resolve_theme_name(theme_spec: &str) -> Option<&str> {
    let theme_spec = theme_spec.trim();
    if theme_spec.is_empty() {
        return None;
    }

    // Check for dark:/light: syntax
    if theme_spec.contains(':') {
        let is_dark = is_system_dark_mode();
        for part in theme_spec.split(',') {
            let part = part.trim();
            if let Some(name) = part.strip_prefix("dark:")
                && is_dark
            {
                return Some(name.trim());
            } else if let Some(name) = part.strip_prefix("light:")
                && !is_dark
            {
                return Some(name.trim());
            }
        }
        // If no matching variant found, try to use the first one
        for part in theme_spec.split(',') {
            let part = part.trim();
            if let Some(name) = part
                .strip_prefix("dark:")
                .or_else(|| part.strip_prefix("light:"))
            {
                return Some(name.trim());
            }
        }
        None
    } else {
        // Simple theme name
        Some(theme_spec)
    }
}

/// Detect if the system is in dark mode.
#[cfg(target_os = "macos")]
fn is_system_dark_mode() -> bool {
    use std::process::Command;
    // Use defaults read to check macOS appearance.
    // Note: AppleInterfaceStyle key only exists when Dark mode is active.
    // If the key doesn't exist (exit code non-zero), the system is in Light mode.
    Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .map(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .eq_ignore_ascii_case("dark")
        })
        .unwrap_or(false) // Default to light if detection fails
}

#[cfg(not(target_os = "macos"))]
fn is_system_dark_mode() -> bool {
    // Default to dark mode on other platforms
    true
}

/// Parse config file contents into a `TerminalConfig`.
fn parse_config(contents: &str) -> Result<TerminalConfig, ConfigError> {
    let mut config = TerminalConfig::default();

    for (line_num, line) in contents.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed for error messages

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse key=value
        if let Some((key, value)) = parse_line(trimmed) {
            apply_config_option(&mut config, key, value, line_num)?;
        }
        // Lines without '=' are silently ignored (matching Ghostty behavior)
    }

    Ok(config)
}

/// Parse a single line into key and value.
/// Returns `None` if the line doesn't contain '='.
fn parse_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '=');
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    // Strip surrounding quotes if present (Ghostty allows optional quotes)
    let value = strip_quotes(value);
    Some((key, value))
}

/// Strip surrounding double quotes from a string value.
fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Apply a single config option to the config struct.
fn apply_config_option(
    config: &mut TerminalConfig,
    key: &str,
    value: &str,
    line_num: usize,
) -> Result<(), ConfigError> {
    match key {
        "foreground" => {
            if value.is_empty() {
                // Reset to default
                config.default_fg = Rgb {
                    r: 0xFF,
                    g: 0xFF,
                    b: 0xFF,
                };
            } else {
                config.default_fg = parse_color(value).ok_or_else(|| ConfigError::Parse {
                    line: line_num,
                    message: format!("invalid color: {}", value),
                })?;
            }
        }
        "background" => {
            if value.is_empty() {
                // Reset to default
                config.default_bg = Rgb {
                    r: 0x00,
                    g: 0x00,
                    b: 0x00,
                };
            } else {
                config.default_bg = parse_color(value).ok_or_else(|| ConfigError::Parse {
                    line: line_num,
                    message: format!("invalid color: {}", value),
                })?;
            }
        }
        "font-family" => {
            if value.is_empty() {
                config.font_family = None;
            } else {
                config.font_family = Some(value.to_string());
            }
        }
        "font-size" => {
            if value.is_empty() {
                config.font_size = None;
            } else {
                let size = value.parse::<f32>().map_err(|_| ConfigError::Parse {
                    line: line_num,
                    message: format!("invalid font size: {}", value),
                })?;
                if size <= 0.0 {
                    return Err(ConfigError::Parse {
                        line: line_num,
                        message: format!("font size must be positive: {}", value),
                    });
                }
                config.font_size = Some(size);
            }
        }
        "command" => {
            if value.is_empty() {
                config.command = None;
            } else {
                config.command = Some(value.to_string());
            }
        }
        "cursor-style" => {
            config.cursor_style = parse_cursor_style(value).ok_or_else(|| ConfigError::Parse {
                line: line_num,
                message: format!(
                    "invalid cursor style: {} (expected block, bar, or underline)",
                    value
                ),
            })?;
        }
        "cursor-style-blink" => {
            if value.is_empty() {
                config.cursor_style_blink = None;
            } else {
                config.cursor_style_blink =
                    Some(parse_bool(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid boolean: {} (expected true or false)", value),
                    })?);
            }
        }
        "cursor-color" => {
            if value.is_empty() {
                config.cursor_color = CursorColor::CellForeground;
            } else {
                config.cursor_color =
                    parse_cursor_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid cursor color: {}", value),
                    })?;
            }
        }
        "cursor-text" => {
            if value.is_empty() {
                config.cursor_text = CursorColor::CellBackground;
            } else {
                config.cursor_text =
                    parse_cursor_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid cursor text color: {}", value),
                    })?;
            }
        }
        "adjust-cursor-height" => {
            if value.is_empty() {
                config.adjust_cursor_height = None;
            } else {
                config.adjust_cursor_height = Some(parse_percentage(value).ok_or_else(|| ConfigError::Parse {
                    line: line_num,
                    message: format!("invalid cursor height: {} (expected percentage like 47% or decimal like 0.47)", value),
                })?);
            }
        }
        // cursor-invert-fg-bg is deprecated but we support it for compatibility
        "cursor-invert-fg-bg" => {
            if parse_bool(value).unwrap_or(false) {
                config.cursor_color = CursorColor::CellForeground;
                config.cursor_text = CursorColor::CellBackground;
            }
        }
        "theme" => {
            if !value.is_empty() {
                // Store the raw theme spec for dynamic appearance switching
                config.theme_spec = Some(value.to_string());
                eprintln!("[theme] Parsing theme spec: {:?}", value);
                if let Some(theme_name) = resolve_theme_name(value) {
                    eprintln!("[theme] Resolved theme name: {:?}", theme_name);
                    match load_theme(config, theme_name) {
                        Ok(()) => {
                            eprintln!(
                                "[theme] Theme loaded successfully: bg={:?}, fg={:?}",
                                config.default_bg, config.default_fg
                            );
                        }
                        Err(e) => {
                            eprintln!("[theme] Failed to load theme {:?}: {}", theme_name, e);
                        }
                    }
                } else {
                    eprintln!("[theme] Failed to resolve theme name from spec: {:?}", value);
                }
            }
        }
        "palette" => {
            // Format: "palette = N=#RRGGBB" where N is 0-15
            if let Some((index, color)) = parse_palette_entry(value)
                && index < 16
            {
                let palette = config.palette.get_or_insert(DEFAULT_PALETTE);
                palette[index] = color;
            }
            // Invalid palette entries are silently ignored
        }
        "selection-background" => {
            if value.is_empty() {
                config.selection_background = None;
            } else {
                config.selection_background =
                    Some(parse_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid selection background color: {}", value),
                    })?);
            }
        }
        "selection-foreground" => {
            if value.is_empty() {
                config.selection_foreground = None;
            } else {
                config.selection_foreground =
                    Some(parse_color(value).ok_or_else(|| ConfigError::Parse {
                        line: line_num,
                        message: format!("invalid selection foreground color: {}", value),
                    })?);
            }
        }
        // Unknown keys are silently ignored (matching Ghostty behavior for forward compatibility)
        _ => {}
    }

    Ok(())
}

/// Parse a hex color value.
///
/// Supports formats:
/// - `#RRGGBB` (with hash)
/// - `RRGGBB` (without hash)
pub fn parse_color(value: &str) -> Option<Rgb> {
    let hex = value.strip_prefix('#').unwrap_or(value);

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Rgb { r, g, b })
}

/// Parse a cursor style value.
fn parse_cursor_style(value: &str) -> Option<CursorStyle> {
    match value.to_lowercase().as_str() {
        "block" => Some(CursorStyle::Block),
        "bar" => Some(CursorStyle::Bar),
        "underline" => Some(CursorStyle::Underline),
        _ => None,
    }
}

/// Parse a boolean value.
fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

/// Parse a cursor color value.
///
/// Supports:
/// - `cell-foreground` or `CellForeground`
/// - `cell-background` or `CellBackground`
/// - Hex color (`#RRGGBB` or `RRGGBB`)
fn parse_cursor_color(value: &str) -> Option<CursorColor> {
    // Normalize: lowercase, replace underscore with dash, remove all dashes
    let normalized = value.to_lowercase().replace('_', "-");
    match normalized.as_str() {
        "cell-foreground" | "cellforeground" => Some(CursorColor::CellForeground),
        "cell-background" | "cellbackground" => Some(CursorColor::CellBackground),
        _ => parse_color(value).map(CursorColor::Color),
    }
}

/// Parse a percentage value.
///
/// Supports:
/// - `47%` (percentage with suffix)
/// - `0.47` (decimal)
/// - `47` (treated as percentage)
fn parse_percentage(value: &str) -> Option<f32> {
    if let Some(stripped) = value.strip_suffix('%') {
        // "47%" -> 0.47
        stripped.parse::<f32>().ok().map(|v| v / 100.0)
    } else if let Ok(v) = value.parse::<f32>() {
        // "0.47" or "47"
        if v > 1.0 {
            // "47" -> 0.47
            Some(v / 100.0)
        } else {
            // "0.47" -> 0.47
            Some(v)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_with_hash() {
        let color = parse_color("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_parse_color_without_hash() {
        let color = parse_color("00ff00").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_parse_color_mixed_case() {
        let color = parse_color("#AaBbCc").unwrap();
        assert_eq!(color.r, 0xAA);
        assert_eq!(color.g, 0xBB);
        assert_eq!(color.b, 0xCC);
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("invalid").is_none());
        assert!(parse_color("#fff").is_none()); // Too short
        assert!(parse_color("#gggggg").is_none()); // Invalid hex
        assert!(parse_color("").is_none());
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(parse_line("key=value"), Some(("key", "value")));
        assert_eq!(parse_line("key = value"), Some(("key", "value")));
        assert_eq!(parse_line("key= value"), Some(("key", "value")));
        assert_eq!(parse_line("key =value"), Some(("key", "value")));
        assert_eq!(parse_line("key ="), Some(("key", "")));
        assert_eq!(parse_line("no-equals"), None);
        // Quoted values should have quotes stripped
        assert_eq!(
            parse_line("key = \"quoted value\""),
            Some(("key", "quoted value"))
        );
        assert_eq!(
            parse_line("font-family = \"JetBrains Mono\""),
            Some(("font-family", "JetBrains Mono"))
        );
    }

    #[test]
    fn test_parse_config_empty() {
        let config = parse_config("").unwrap();
        assert_eq!(config.default_fg.r, 0xFF);
        assert_eq!(config.default_bg.r, 0x00);
        assert!(config.font_family.is_none());
        assert!(config.font_size.is_none());
    }

    #[test]
    fn test_parse_config_comments() {
        let input = r#"
# This is a comment
foreground = #ffffff

# Another comment
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.default_fg.r, 0xFF);
    }

    #[test]
    fn test_parse_config_colors() {
        let input = r#"
foreground = #eaeaea
background = #1a1a2e
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.default_fg.r, 0xEA);
        assert_eq!(config.default_fg.g, 0xEA);
        assert_eq!(config.default_fg.b, 0xEA);
        assert_eq!(config.default_bg.r, 0x1A);
        assert_eq!(config.default_bg.g, 0x1A);
        assert_eq!(config.default_bg.b, 0x2E);
    }

    #[test]
    fn test_parse_config_font() {
        let input = r#"
font-family = JetBrains Mono
font-size = 14.5
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.font_family.as_deref(), Some("JetBrains Mono"));
        assert_eq!(config.font_size, Some(14.5));
    }

    #[test]
    fn test_parse_config_reset_values() {
        let input = r#"
font-family = Some Font
font-family =
font-size = 12
font-size =
"#;
        let config = parse_config(input).unwrap();
        assert!(config.font_family.is_none());
        assert!(config.font_size.is_none());
    }

    #[test]
    fn test_parse_config_unknown_keys() {
        // Unknown keys should be silently ignored
        let input = r#"
unknown-key = some value
foreground = #ffffff
another-unknown = test
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.default_fg.r, 0xFF);
    }

    #[test]
    fn test_parse_config_invalid_color() {
        let input = "foreground = notacolor";
        let result = parse_config(input);
        assert!(matches!(result, Err(ConfigError::Parse { line: 1, .. })));
    }

    #[test]
    fn test_parse_config_invalid_font_size() {
        let input = "font-size = abc";
        let result = parse_config(input);
        assert!(matches!(result, Err(ConfigError::Parse { line: 1, .. })));
    }

    #[test]
    fn test_parse_config_negative_font_size() {
        let input = "font-size = -12";
        let result = parse_config(input);
        assert!(matches!(result, Err(ConfigError::Parse { line: 1, .. })));
    }

    #[test]
    fn test_parse_cursor_style() {
        assert_eq!(parse_cursor_style("block"), Some(CursorStyle::Block));
        assert_eq!(parse_cursor_style("bar"), Some(CursorStyle::Bar));
        assert_eq!(
            parse_cursor_style("underline"),
            Some(CursorStyle::Underline)
        );
        assert_eq!(parse_cursor_style("Block"), Some(CursorStyle::Block));
        assert_eq!(parse_cursor_style("BAR"), Some(CursorStyle::Bar));
        assert_eq!(parse_cursor_style("invalid"), None);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("no"), Some(false));
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool("invalid"), None);
    }

    #[test]
    fn test_parse_cursor_color() {
        assert_eq!(
            parse_cursor_color("cell-foreground"),
            Some(CursorColor::CellForeground)
        );
        assert_eq!(
            parse_cursor_color("cell-background"),
            Some(CursorColor::CellBackground)
        );
        assert_eq!(
            parse_cursor_color("CellForeground"),
            Some(CursorColor::CellForeground)
        );
        assert_eq!(
            parse_cursor_color("cell_foreground"),
            Some(CursorColor::CellForeground)
        );
        assert_eq!(
            parse_cursor_color("#ff0000"),
            Some(CursorColor::Color(Rgb { r: 255, g: 0, b: 0 }))
        );
        assert_eq!(
            parse_cursor_color("00ff00"),
            Some(CursorColor::Color(Rgb { r: 0, g: 255, b: 0 }))
        );
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("47%"), Some(0.47));
        assert_eq!(parse_percentage("100%"), Some(1.0));
        assert_eq!(parse_percentage("0.47"), Some(0.47));
        assert_eq!(parse_percentage("47"), Some(0.47));
        assert_eq!(parse_percentage("0.5"), Some(0.5));
        assert!(parse_percentage("invalid").is_none());
    }

    #[test]
    fn test_parse_config_cursor_settings() {
        let input = r#"
cursor-style = bar
cursor-style-blink = false
cursor-color = #ff0000
cursor-text = cell-background
adjust-cursor-height = 47%
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(config.cursor_style, CursorStyle::Bar);
        assert_eq!(config.cursor_style_blink, Some(false));
        assert_eq!(
            config.cursor_color,
            CursorColor::Color(Rgb { r: 255, g: 0, b: 0 })
        );
        assert_eq!(config.cursor_text, CursorColor::CellBackground);
        assert!((config.adjust_cursor_height.unwrap() - 0.47).abs() < 0.001);
    }

    #[test]
    fn test_parse_config_cursor_invert_fg_bg() {
        let input = "cursor-invert-fg-bg = true";
        let config = parse_config(input).unwrap();
        assert_eq!(config.cursor_color, CursorColor::CellForeground);
        assert_eq!(config.cursor_text, CursorColor::CellBackground);
    }

    #[test]
    fn test_parse_palette_entry() {
        // Valid entries
        assert_eq!(
            parse_palette_entry("0=#ff0000"),
            Some((0, Rgb { r: 255, g: 0, b: 0 }))
        );
        assert_eq!(
            parse_palette_entry("15=#00ff00"),
            Some((15, Rgb { r: 0, g: 255, b: 0 }))
        );
        assert_eq!(
            parse_palette_entry("7 = #aabbcc"),
            Some((
                7,
                Rgb {
                    r: 0xAA,
                    g: 0xBB,
                    b: 0xCC
                }
            ))
        );

        // Invalid entries
        assert!(parse_palette_entry("invalid").is_none());
        assert!(parse_palette_entry("0=invalid").is_none());
        assert!(parse_palette_entry("abc=#ff0000").is_none());
    }

    #[test]
    fn test_parse_config_palette() {
        let input = r#"
palette = 0=#45475a
palette = 1=#f38ba8
palette = 15=#bac2de
"#;
        let config = parse_config(input).unwrap();
        let palette = config.palette.unwrap();
        assert_eq!(
            palette[0],
            Rgb {
                r: 0x45,
                g: 0x47,
                b: 0x5a
            }
        );
        assert_eq!(
            palette[1],
            Rgb {
                r: 0xf3,
                g: 0x8b,
                b: 0xa8
            }
        );
        assert_eq!(
            palette[15],
            Rgb {
                r: 0xba,
                g: 0xc2,
                b: 0xde
            }
        );
    }

    #[test]
    fn test_parse_config_selection_colors() {
        let input = r#"
selection-background = #585b70
selection-foreground = #cdd6f4
"#;
        let config = parse_config(input).unwrap();
        assert_eq!(
            config.selection_background,
            Some(Rgb {
                r: 0x58,
                g: 0x5b,
                b: 0x70
            })
        );
        assert_eq!(
            config.selection_foreground,
            Some(Rgb {
                r: 0xcd,
                g: 0xd6,
                b: 0xf4
            })
        );
    }

    #[test]
    fn test_parse_config_selection_colors_reset() {
        let input = r#"
selection-background = #ff0000
selection-background =
selection-foreground = #00ff00
selection-foreground =
"#;
        let config = parse_config(input).unwrap();
        assert!(config.selection_background.is_none());
        assert!(config.selection_foreground.is_none());
    }

    #[test]
    fn test_resolve_theme_name_simple() {
        assert_eq!(
            resolve_theme_name("Catppuccin Mocha"),
            Some("Catppuccin Mocha")
        );
        assert_eq!(resolve_theme_name("  Dracula  "), Some("Dracula"));
        assert_eq!(resolve_theme_name(""), None);
    }

    #[test]
    fn test_resolve_theme_name_dark_light() {
        // Test dark/light syntax - result depends on system appearance
        let spec = "dark:Catppuccin Mocha,light:Catppuccin Latte";
        let result = resolve_theme_name(spec);
        // Should return one of the theme names
        assert!(result == Some("Catppuccin Mocha") || result == Some("Catppuccin Latte"));
    }

    #[test]
    fn test_resolve_theme_name_only_dark() {
        let spec = "dark:Dracula";
        let result = resolve_theme_name(spec);
        // If system is dark, returns Dracula; otherwise falls back to Dracula anyway
        assert_eq!(result, Some("Dracula"));
    }

    #[test]
    fn test_resolve_theme_name_only_light() {
        let spec = "light:Solarized Light";
        let result = resolve_theme_name(spec);
        // Returns Solarized Light (either as match or fallback)
        assert_eq!(result, Some("Solarized Light"));
    }

    #[test]
    fn test_apply_theme_contents() {
        let theme_contents = r#"
palette = 0=#45475a
palette = 1=#f38ba8
background = #1e1e2e
foreground = #cdd6f4
cursor-color = #f5e0dc
cursor-text = #1e1e2e
selection-background = #585b70
selection-foreground = #cdd6f4
"#;
        let mut config = TerminalConfig::default();
        apply_theme_contents(&mut config, theme_contents).unwrap();

        assert_eq!(
            config.default_bg,
            Rgb {
                r: 0x1e,
                g: 0x1e,
                b: 0x2e
            }
        );
        assert_eq!(
            config.default_fg,
            Rgb {
                r: 0xcd,
                g: 0xd6,
                b: 0xf4
            }
        );
        assert_eq!(
            config.cursor_color,
            CursorColor::Color(Rgb {
                r: 0xf5,
                g: 0xe0,
                b: 0xdc
            })
        );
        assert_eq!(
            config.cursor_text,
            CursorColor::Color(Rgb {
                r: 0x1e,
                g: 0x1e,
                b: 0x2e
            })
        );
        assert_eq!(
            config.selection_background,
            Some(Rgb {
                r: 0x58,
                g: 0x5b,
                b: 0x70
            })
        );
        assert_eq!(
            config.selection_foreground,
            Some(Rgb {
                r: 0xcd,
                g: 0xd6,
                b: 0xf4
            })
        );

        let palette = config.palette.unwrap();
        assert_eq!(
            palette[0],
            Rgb {
                r: 0x45,
                g: 0x47,
                b: 0x5a
            }
        );
        assert_eq!(
            palette[1],
            Rgb {
                r: 0xf3,
                g: 0x8b,
                b: 0xa8
            }
        );
    }

    #[test]
    fn test_apply_theme_contents_partial() {
        // Theme files can have partial content
        let theme_contents = r#"
background = #282a36
foreground = #f8f8f2
"#;
        let mut config = TerminalConfig::default();
        apply_theme_contents(&mut config, theme_contents).unwrap();

        assert_eq!(
            config.default_bg,
            Rgb {
                r: 0x28,
                g: 0x2a,
                b: 0x36
            }
        );
        assert_eq!(
            config.default_fg,
            Rgb {
                r: 0xf8,
                g: 0xf8,
                b: 0xf2
            }
        );
        // Other fields should remain at defaults
        assert!(config.palette.is_none());
        assert!(config.selection_background.is_none());
    }
}
