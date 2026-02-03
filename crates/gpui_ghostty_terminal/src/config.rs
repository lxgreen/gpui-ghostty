use ghostty_vt::Rgb;

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
        }
    }
}
