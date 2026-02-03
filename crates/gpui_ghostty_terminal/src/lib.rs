mod config;
mod config_file;
mod font;
mod session;

pub mod view;

pub use config::{CursorColor, TerminalConfig};
pub use config_file::{ConfigError, load_config, load_config_from_path};
pub use font::{default_terminal_font, default_terminal_font_features, terminal_font};
pub use ghostty_vt::CursorStyle;
pub use session::TerminalSession;

#[cfg(test)]
mod tests;
