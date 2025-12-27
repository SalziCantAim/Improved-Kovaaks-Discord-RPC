use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub installation_path: String,
    pub steam_path: String,
    pub open_manually: bool,
    pub start_with_windows: bool,
    pub webapp_username: String,
    pub show_online_scores: bool,
    pub start_in_tray: bool,
    pub online_only_scenarios: bool,
    pub online_scores_synced: bool,
    #[serde(default)]
    pub last_sync_time: u64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            installation_path: String::new(),
            steam_path: r"C:\Program Files (x86)\Steam\steam.exe".to_string(),
            open_manually: false,
            start_with_windows: false,
            webapp_username: String::new(),
            show_online_scores: false,
            start_in_tray: false,
            online_only_scenarios: false,
            online_scores_synced: false,
            last_sync_time: 0,
        }
    }
}
fn get_settings_path() -> PathBuf {
    let app_data_dir = crate::backend::get_app_data_dir();
    let _ = fs::create_dir_all(&app_data_dir);
    app_data_dir.join("settings.json")
}
pub fn load_settings() -> Result<Settings> {
    let settings_path = get_settings_path();
    if let Ok(contents) = fs::read_to_string(&settings_path) {
        let settings: Settings = serde_json::from_str(&contents)?;

        Ok(settings)
    } else {

        Ok(Settings::default())
    }
}
pub fn save_settings(settings: &Settings) -> Result<()> {
    let settings_path = get_settings_path();
    let json = serde_json::to_string_pretty(settings)?;
    let mut tmp = settings_path.clone();
    tmp.set_extension("tmp");
    fs::write(&tmp, &json)?;
    fs::rename(&tmp, &settings_path)?;
    Ok(())
}
#[cfg(target_os = "windows")]
pub fn get_steam_path_from_registry() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(steam_key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam") {
        if let Ok(install_path) = steam_key.get_value::<String, _>("InstallPath") {
            let candidate = PathBuf::from(&install_path)
                .join("steamapps")
                .join("common")
                .join("FPSAimTrainer")
                .join("FPSAimTrainer");
            if candidate.join("stats").exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}
#[cfg(not(target_os = "windows"))]
pub fn get_steam_path_from_registry() -> Option<String> {
    None
}
pub fn initialize_installation_path(settings: &mut Settings) -> Result<()> {
    if settings.installation_path.is_empty() {
        if let Some(detected) = get_steam_path_from_registry() {

            settings.installation_path = detected;
            save_settings(settings)?;
        }
    }
    Ok(())
}
pub fn get_stats_directory(settings: &Settings) -> PathBuf {
    PathBuf::from(&settings.installation_path).join("stats")
}