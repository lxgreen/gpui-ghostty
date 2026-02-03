use ghostty_vt::{CursorStyle, Rgb};

/// Cursor color configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CursorColor {
    /// Explicit RGB color.
    Color(Rgb),
    /// Use the cell's foreground color (inverts with text).
    #[default]
    CellForeground,
    /// Use the cell's background color.
    CellBackground,
}

#[derive(Clone, Debug)]
pub struct TerminalConfig {
    pub cols: u16,
    pub rows: u16,
    pub default_fg: Rgb,
    pub default_bg: Rgb,
    pub update_window_title: bool,
    /// Font family name (e.g., "JetBrains Mono"). If `None`, uses platform default.
    pub font_family: Option<String>,
    /// Font size in points. If `None`, uses the system default.
    pub font_size: Option<f32>,
    /// Shell command to run. If `None`, uses `$SHELL` or platform default.
    pub command: Option<String>,

    /// Default cursor style (block/bar/underline). Can be overridden by DECSCUSR.
    pub cursor_style: CursorStyle,
    /// Whether cursor should blink. If `None`, follows terminal escape sequences.
    pub cursor_style_blink: Option<bool>,
    /// Cursor color. Defaults to `CellForeground` for good contrast.
    pub cursor_color: CursorColor,
    /// Color for text under the cursor. Defaults to `CellBackground`.
    pub cursor_text: CursorColor,
    /// Adjust cursor height as percentage (0.0-1.0). Only affects bar/underline.
    /// Values > 1.0 are treated as percentages (e.g., 47 means 47%).
    pub adjust_cursor_height: Option<f32>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            default_fg: Rgb {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
            },
            default_bg: Rgb {
                r: 0x00,
                g: 0x00,
                b: 0x00,
            },
            update_window_title: true,
            font_family: None,
            font_size: None,
            command: None,
            cursor_style: CursorStyle::Block,
            cursor_style_blink: None,
            cursor_color: CursorColor::CellForeground,
            cursor_text: CursorColor::CellBackground,
            adjust_cursor_height: None,
        }
    }
}
