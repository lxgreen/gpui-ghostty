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

/// Default 16-color ANSI palette (colors 0-15).
/// Standard terminal colors: 0-7 normal, 8-15 bright variants.
pub const DEFAULT_PALETTE: [Rgb; 16] = [
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // 0: Black
    Rgb {
        r: 0xCD,
        g: 0x00,
        b: 0x00,
    }, // 1: Red
    Rgb {
        r: 0x00,
        g: 0xCD,
        b: 0x00,
    }, // 2: Green
    Rgb {
        r: 0xCD,
        g: 0xCD,
        b: 0x00,
    }, // 3: Yellow
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0xEE,
    }, // 4: Blue
    Rgb {
        r: 0xCD,
        g: 0x00,
        b: 0xCD,
    }, // 5: Magenta
    Rgb {
        r: 0x00,
        g: 0xCD,
        b: 0xCD,
    }, // 6: Cyan
    Rgb {
        r: 0xE5,
        g: 0xE5,
        b: 0xE5,
    }, // 7: White
    Rgb {
        r: 0x7F,
        g: 0x7F,
        b: 0x7F,
    }, // 8: Bright Black (Gray)
    Rgb {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    }, // 9: Bright Red
    Rgb {
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    }, // 10: Bright Green
    Rgb {
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    }, // 11: Bright Yellow
    Rgb {
        r: 0x5C,
        g: 0x5C,
        b: 0xFF,
    }, // 12: Bright Blue
    Rgb {
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    }, // 13: Bright Magenta
    Rgb {
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    }, // 14: Bright Cyan
    Rgb {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // 15: Bright White
];

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

    /// 16-color ANSI palette (colors 0-15). If `None`, uses default palette.
    pub palette: Option<[Rgb; 16]>,
    /// Selection background color. If `None`, uses a default highlight color.
    pub selection_background: Option<Rgb>,
    /// Selection foreground color. If `None`, keeps original text color.
    pub selection_foreground: Option<Rgb>,

    /// Raw theme specification for dynamic dark/light mode switching.
    /// E.g., `"dark:Catppuccin Mocha,light:Catppuccin Latte"`.
    /// Used internally to reload theme when system appearance changes.
    pub theme_spec: Option<String>,

    /// Background opacity (0.0 = fully transparent, 1.0 = fully opaque).
    /// Values below 1.0 enable a frosted-glass blur effect behind the window on macOS.
    pub background_opacity: f32,
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
            palette: None,
            selection_background: None,
            selection_foreground: None,
            theme_spec: None,
            background_opacity: 1.0,
        }
    }
}
