mod config;
mod config_file;
mod font;
mod session;
mod themes;

pub mod view;

pub use config::{CursorColor, DEFAULT_PALETTE, TerminalConfig};
pub use config_file::{
    ConfigError, load_config, load_config_from_path, reload_theme_for_appearance,
};
pub use font::{default_terminal_font, default_terminal_font_features, terminal_font};
pub use ghostty_vt::CursorStyle;
pub use session::TerminalSession;
pub use themes::{get_embedded_theme, list_embedded_themes};

#[cfg(test)]
mod tests;
