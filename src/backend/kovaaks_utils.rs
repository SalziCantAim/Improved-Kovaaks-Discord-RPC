use sysinfo::System;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn log_debug(message: &str) {

    if let Ok(temp_dir) = std::env::var("TEMP") {
        let log_path = PathBuf::from(temp_dir).join("kovaaks_rpc_debug.log");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] {}", timestamp, message);
        }
    }
}
pub fn is_kovaaks_running() -> bool {
    let mut system = System::new_all();
    system.refresh_processes();
    for (_pid, process) in system.processes() {

        let name = process.name();
        let process_name = name.to_lowercase();
        if process_name.contains("fpsaimtrainer") {
            if process_name.contains("discord") || process_name.contains("rpc") {
                log_debug(&format!("[DEBUG] Skipping RPC app itself: {}", name));
                continue;
            }
            log_debug(&format!("[DEBUG] Kovaaks process detected: {}", name));
            return true;
        }
    }
    log_debug("[DEBUG] Kovaaks not detected");
    false
}
pub fn extract_scenario_name(file_path: &Path) -> Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let keys: &[&[u8]] = &[b"FullScenarioPath", b"LastEditProfile"];
    let mut key_pos: Option<usize> = None;
    for key in keys {
        if let Some(pos) = find_subsequence(&data, key) {
            key_pos = Some(pos);
            break;
        }
    }
    if let Some(mut end) = key_pos {
        while end > 0 && (data[end - 1] < 32 || data[end - 1] > 126) {
            end -= 1;
        }
        let mut start = end.saturating_sub(1);
        while start > 0 && data[start] >= 32 && data[start] <= 126 {
            start -= 1;
        }
        start += 1;
        if start < end {
            let scenario_bytes = &data[start..end];
            let scenario_name = String::from_utf8_lossy(scenario_bytes).to_string();
            return Ok(scenario_name);
        }
    }
    Ok("Unknown Scenario".to_string())
}
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
pub fn get_current_scenario() -> Result<String> {
    let temp_dir = std::env::var("LOCALAPPDATA")?;
    let source_path = PathBuf::from(&temp_dir)
        .join("FPSAimTrainer")
        .join("Saved")
        .join("SaveGames")
        .join("session.sav");
    if source_path.exists() {
        let temp_file = PathBuf::from(&temp_dir)
            .join("Temp")
            .join("session_copy.sav");
        fs::copy(&source_path, &temp_file)?;
        let scenario_name = extract_scenario_name(&temp_file)?;
        let _ = fs::remove_file(&temp_file);
        Ok(scenario_name)
    } else {
        Ok("Unknown Scenario".to_string())
    }
}
pub fn find_initial_scores(scenario_name: &str, stats_directory: &Path) -> Result<(f64, Vec<String>)> {
    let mut highscore: f64 = 0.0;
    let mut checked_files = Vec::new();
    if let Ok(entries) = fs::read_dir(stats_directory) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with(&format!("{} - ", scenario_name)) && file_name_str.ends_with(".csv") {
                let file_path = entry.path();
                if let Ok(content) = fs::read_to_string(&file_path) {
                    for line in content.lines() {
                        if line.contains("Score:,") {
                            if let Some(score_str) = line.split(',').nth(1) {
                                if let Ok(score) = score_str.parse::<f64>() {
                                    highscore = highscore.max(score);
                                }
                            }
                        }
                    }
                }
                checked_files.push(file_name_str.to_string());
            }
        }
    }
    Ok(((highscore * 10.0).round() / 10.0, checked_files))
}
pub fn find_fight_time_and_score(
    scenario_name: &str,
    stats_directory: &Path,
    checked_files: &[String],
) -> Result<(f64, bool, Option<std::time::SystemTime>)> {
    let mut max_score: f64 = 0.0;
    let mut found_new_score = false;
    let mut newest_file_time: Option<std::time::SystemTime> = None;
    if let Ok(entries) = fs::read_dir(stats_directory) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with(&format!("{} - ", scenario_name))
                && !checked_files.contains(&file_name_str.to_string()) {
                let file_path = entry.path();
                let file_time = if let Ok(metadata) = fs::metadata(&file_path) {
                    metadata.created().or_else(|_| metadata.modified()).ok()
                } else {
                    None
                };
                if let Ok(content) = fs::read_to_string(&file_path) {
                    for line in content.lines() {
                        if line.contains("Score:,") {
                            if let Some(score_str) = line.split(',').nth(1) {
                                if let Ok(score) = score_str.parse::<f64>() {
                                    if score > max_score {
                                        max_score = score;
                                        newest_file_time = file_time;
                                    }
                                    found_new_score = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(((max_score * 10.0).round() / 10.0, found_new_score, newest_file_time))
}
#[allow(dead_code)]
pub fn get_last_played_time(scenario_name: &str, stats_directory: &Path) -> Option<std::time::SystemTime> {
    let mut newest_time: Option<std::time::SystemTime> = None;
    if let Ok(entries) = fs::read_dir(stats_directory) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with(&format!("{} - ", scenario_name)) && file_name_str.ends_with(".csv") {
                let file_path = entry.path();
                if let Ok(metadata) = fs::metadata(&file_path) {
                    let file_time = metadata.created().or_else(|_| metadata.modified()).ok();
                    if let Some(time) = file_time {
                        newest_time = Some(newest_time.map_or(time, |current| current.max(time)));
                    }
                }
            }
        }
    }
    newest_time
}
pub fn scan_all_stats_folder(stats_dir: &Path) -> Result<std::collections::HashMap<String, (f64, Option<std::time::SystemTime>)>> {
    scan_stats_folder_since(stats_dir, None)
}
pub fn scan_stats_folder_since(stats_dir: &Path, since_timestamp: Option<u64>) -> Result<std::collections::HashMap<String, (f64, Option<std::time::SystemTime>)>> {
    use std::collections::HashMap;
    use std::time::UNIX_EPOCH;
    let mut scenario_scores: HashMap<String, (f64, Option<std::time::SystemTime>)> = HashMap::new();
    if !stats_dir.exists() {

        return Ok(scenario_scores);
    }
    if let Some(_ts) = since_timestamp {
    }
    let entries = match fs::read_dir(stats_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(scenario_scores),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        if let Some(since) = since_timestamp {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                        let file_timestamp = duration.as_secs();
                        if file_timestamp <= since {
                            continue;
                        }
                    }
                }
            }
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        let name_without_ext = file_name.trim_end_matches(".csv");
        let scenario_name = if let Some(pos) = name_without_ext.rfind(" - ") {
            name_without_ext[..pos].to_string()
        } else {
            name_without_ext.to_string()
        };
        if scenario_name.is_empty() {
            continue;
        }
        let score = match fs::read_to_string(&path) {
            Ok(content) => {
                let mut found_score = 0.0;
                for line in content.lines() {
                    if line.contains("Score:,") {
                        if let Some(score_str) = line.split(',').nth(1) {
                            if let Ok(score) = score_str.parse::<f64>() {
                                found_score = score;
                                break;
                            }
                        }
                    }
                }
                found_score
            }
            Err(_) => continue,
        };
        let last_played = fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        scenario_scores
            .entry(scenario_name.clone())
            .and_modify(|(existing_score, existing_time)| {
                if score > *existing_score {
                    *existing_score = score;
                    *existing_time = last_played;
                }
            })
            .or_insert((score, last_played));
    }
    Ok(scenario_scores)
}
pub fn get_playlist_share_code(installation_path: &str) -> Option<String> {
    let playlist_file = PathBuf::from(installation_path)
        .join("Saved")
        .join("SaveGames")
        .join("PlaylistInProgress.json");
    if let Ok(content) = fs::read_to_string(playlist_file) {
        if let Some(start) = content.find(r#""shareCode": ""#) {
            let start = start + r#""shareCode": ""#.len();
            if let Some(end) = content[start..].find('"') {
                return Some(content[start..start + end].to_string());
            }
        }
    }
    None
}