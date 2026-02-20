pub mod config;
pub mod config_file;
mod font;
mod session;
pub mod themes;

pub mod view;

pub use config::{CursorColor, DEFAULT_PALETTE, TerminalConfig};
pub use config_file::{
    ConfigError, load_config, load_config_from_path, reload_theme_for_appearance,
    save_theme_to_config,
};
pub use font::{default_terminal_font, default_terminal_font_features, terminal_font};
pub use ghostty_vt::{CursorStyle, Rgb};
pub use session::TerminalSession;
pub use themes::{get_embedded_theme, list_embedded_themes};

use gpui::{WindowBackgroundAppearance, WindowOptions};

/// Build `WindowOptions` with the appropriate background appearance for the given config.
///
/// When `background_opacity < 1.0`, uses `WindowBackgroundAppearance::Blurred` to enable
/// a frosted-glass effect on macOS. Otherwise uses `Opaque`.
pub fn window_options_for_config(config: &TerminalConfig) -> WindowOptions {
    let background = if config.background_opacity < 1.0 {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    };
    WindowOptions {
        window_background: background,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests;
