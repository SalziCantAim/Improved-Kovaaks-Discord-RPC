pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[cfg(target_os = "windows")]
pub fn get_autostart_enabled() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
        run_key.get_value::<String, _>("KovaaksDiscordRPC").is_ok()
    } else {
        false
    }
}
#[cfg(target_os = "windows")]
pub fn set_autostart_enabled(enable: bool) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        KEY_WRITE,
    )?;
    if enable {
        let exe_path = std::env::current_exe()?;
        let exe_path_str = format!("\"{}\"", exe_path.display());
        run_key.set_value("KovaaksDiscordRPC", &exe_path_str)?;

    } else {
        let _ = run_key.delete_value("KovaaksDiscordRPC");

    }
    Ok(())
}
#[cfg(not(target_os = "windows"))]
pub fn get_autostart_enabled() -> bool {
    false
}
#[cfg(not(target_os = "windows"))]
pub fn set_autostart_enabled(_enable: bool) -> Result<()> {
    Ok(())
}