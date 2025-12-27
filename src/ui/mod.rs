pub mod main_tab;
pub mod settings_tab;
pub mod theme;
pub use main_tab::render_main_tab;
pub use settings_tab::{render_settings_tab, SettingsForm};
pub use theme::*;