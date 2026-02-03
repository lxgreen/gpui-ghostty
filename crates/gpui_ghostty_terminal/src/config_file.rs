//! Ghostty config file parser.
//!
//! Loads configuration from `~/.config/ghostty/config` (or `$XDG_CONFIG_HOME/ghostty/config`)
//! using the Ghostty key-value format.

use std::fs;
use std::io;
use std::path::PathBuf;

use ghostty_vt::{CursorStyle, Rgb};

use crate::TerminalConfig;
use crate::config::CursorColor;

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
}
