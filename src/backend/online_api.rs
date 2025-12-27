use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::time::Instant;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const CACHE_TTL_DAYS: i64 = 7;
#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    #[serde(with = "chrono::serde::ts_seconds")]
    fetched_at: DateTime<Utc>,
    scores: HashMap<String, f64>,
}
#[derive(Debug, Serialize, Deserialize)]
struct LocalScoresData {
    username: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    last_updated: DateTime<Utc>,
    scores: HashMap<String, f64>,
}
#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<ScenarioEntry>,
}
#[derive(Debug, Deserialize)]
struct ScenarioEntry {
    #[serde(rename = "scenarioName")]
    scenario_name: String,
    score: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_attributes")]
    attributes: Option<ScenarioAttributes>,
}
#[derive(Debug, Deserialize)]
struct ScenarioAttributes {
    score: Option<f64>,
}
fn deserialize_attributes<'de, D>(deserializer: D) -> std::result::Result<Option<ScenarioAttributes>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde::Deserialize;
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Bool(false) | serde_json::Value::Null => Ok(None),
        serde_json::Value::Object(_) => {
            ScenarioAttributes::deserialize(value)
                .map(Some)
                .map_err(D::Error::custom)
        }
        _ => Ok(None),
    }
}
#[derive(Debug, Deserialize)]
struct PopularSearchResponse {
    data: Vec<PopularScenarioEntry>,
}
#[derive(Debug, Deserialize)]
struct PopularScenarioEntry {
    #[serde(rename = "scenarioName")]
    scenario_name: String,
}
pub struct OnlineScoreAPI {
    base_url: String,
    cache_dir: PathBuf,
    local_scores_file: PathBuf,
    raw_scores_dir: PathBuf,
}
#[allow(dead_code)]
impl OnlineScoreAPI {
    pub fn new() -> Self {
        let app_data_dir = crate::backend::get_app_data_dir();
        let cache_dir = app_data_dir.join("cache");
        let _ = fs::create_dir_all(&cache_dir);
        let raw_scores_dir = app_data_dir.join("raw_scores");
        let _ = fs::create_dir_all(&raw_scores_dir);
        let local_scores_file = app_data_dir.join("online_highscores.json");
        Self {
            base_url: "https://kovaaks.com/webapp-backend".to_string(),
            cache_dir,
            local_scores_file,
            raw_scores_dir,
        }
    }
    fn cache_path(&self, username: &str) -> PathBuf {
        let safe_username = username.replace(['/', '\\'], "_");
        self.cache_dir.join(format!("{}_scores.json", safe_username))
    }
    fn load_cache(&self, username: &str) -> Option<HashMap<String, f64>> {
        let path = self.cache_path(username);
        if !path.exists() {
            return None;
        }
        match fs::read_to_string(&path) {
            Ok(contents) => {
                if contents.trim().is_empty() {
                    return None;
                }
                match serde_json::from_str::<CacheData>(&contents) {
                    Ok(cache_data) => {
                        let now = Utc::now();
                        let age = now.signed_duration_since(cache_data.fetched_at);
                        if age < Duration::days(CACHE_TTL_DAYS) {
                            return Some(cache_data.scores);
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        None
    }
    fn save_cache(&self, username: &str, scores: HashMap<String, f64>) {
        let cache_data = CacheData {
            fetched_at: Utc::now(),
            scores,
        };
        let path = self.cache_path(username);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&cache_data) {
            Ok(json) => {
                let mut tmp = path.clone();
                tmp.set_extension("tmp");
                if let Err(_) = fs::write(&tmp, &json) {
                    return;
                }
                if let Err(_) = fs::rename(&tmp, &path) {
                    let _ = fs::remove_file(&tmp);
                    return;
                }
            }
            Err(_) => {}
        }
    }
    pub fn load_local_scores(&self) -> HashMap<String, f64> {
        if let Ok(contents) = fs::read_to_string(&self.local_scores_file) {
            match serde_json::from_str::<LocalScoresData>(&contents) {
                Ok(data) => {
                    return data.scores;
                }
                Err(_) => {
                    let mut backup = self.local_scores_file.clone();
                    backup.set_extension("bak");
                    let _ = fs::rename(&self.local_scores_file, &backup);
                }
            }
        }
        HashMap::new()
    }
    pub fn save_local_scores(&self, scores: HashMap<String, f64>, username: &str) -> Result<()> {
        let data = LocalScoresData {
            username: username.to_string(),
            last_updated: Utc::now(),
            scores,
        };
        let json = serde_json::to_string_pretty(&data)?;
        if let Some(parent) = self.local_scores_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut tmp = self.local_scores_file.clone();
        tmp.set_extension("tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, &self.local_scores_file)?;
        Ok(())
    }
    pub fn update_local_score(&self, scenario_name: &str, new_score: f64, username: &str) -> bool {
        let mut scores = self.load_local_scores();
        let current_score = scores.get(scenario_name).copied().unwrap_or(0.0);
        if new_score > current_score {
            scores.insert(scenario_name.to_string(), new_score);
            let _ = self.save_local_scores(scores, username);
            return true;
        }
        false
    }
    pub fn fetch_user_scenario_scores(&self, username: &str) -> Result<HashMap<String, f64>> {
        if username.is_empty() {
            return Ok(HashMap::new());
        }
        if let Some(cached) = self.load_cache(username) {
            return Ok(cached);
        }
        let lock_path = self.cache_path(username).with_extension("lock");
        let wait_total_ms = 5000_usize;
        let wait_sleep_ms = 200_usize;
        let start = Instant::now();
        let try_create_lock = || -> std::io::Result<std::fs::File> {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
        };
        let lock_file_opt = match try_create_lock() {
            Ok(f) => Some(f),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                let mut got_cache = None;
                while start.elapsed().as_millis() < wait_total_ms as u128 {
                    if let Some(cached) = self.load_cache(username) {
                        got_cache = Some(cached);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(wait_sleep_ms as u64));
                }
                if let Some(cached) = got_cache {
                    return Ok(cached);
                } else {
                    match try_create_lock() {
                        Ok(f2) => Some(f2),
                        Err(_) => return Ok(HashMap::new()),
                    }
                }
            }
            Err(_) => None,
        };
        let result = (|| -> Result<HashMap<String, f64>> {
            let scores = self.sync_online_scores_once(username)?;
            self.save_cache(username, scores.clone());
            let _ = self.save_local_scores(scores.clone(), username);
            Ok(scores)
        })();
        if lock_file_opt.is_some() {
            let _ = fs::remove_file(&lock_path);
        }
        result
    }
    pub fn get_online_score(&self, username: &str, scenario_name: &str) -> Option<f64> {
        if username.is_empty() || scenario_name.is_empty() {
            return None;
        }
        let local_scores = self.load_local_scores();
        if let Some(score) = local_scores.get(scenario_name) {
            return Some(*score);
        }
        if let Ok(all_scores) = self.fetch_user_scenario_scores(username) {
            return all_scores.get(scenario_name).copied();
        }
        None
    }
    pub fn search_scenario_popular(&self, scenario_name: &str) -> Result<bool> {
        if scenario_name.is_empty() {
            return Ok(false);
        }
        let encoded_name = scenario_name.to_lowercase().replace(' ', "+");
        let url = format!(
            "{}/scenario/popular?page=0&max=5&scenarioNameSearch={}",
            self.base_url, encoded_name
        );
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send();
        let response = match response {
            Ok(r) => r,
            Err(_) => return Ok(false),
        };
        if !response.status().is_success() {
            return Ok(false);
        }
        let search_response: PopularSearchResponse = match response.json() {
            Ok(r) => r,
            Err(_) => return Ok(false),
        };
        let scenario_lower = scenario_name.to_lowercase();
        for entry in search_response.data {
            if entry.scenario_name.to_lowercase() == scenario_lower {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn is_scenario_available_online(&self, username: &str, scenario_name: &str) -> bool {
        if username.is_empty() || scenario_name.is_empty() {
            return false;
        }
        let local_scores = self.load_local_scores();
        if local_scores.contains_key(scenario_name) {
            return true;
        }
        if let Ok(scores) = self.fetch_user_scenario_scores(username) {
            return scores.contains_key(scenario_name);
        }
        false
    }
    pub fn sync_online_scores_once(&self, username: &str) -> Result<HashMap<String, f64>> {
        if username.is_empty() {
            return Err("Username is empty".into());
        }
        let _ = fs::create_dir_all(&self.raw_scores_dir);
        let client = reqwest::blocking::Client::new();
        let mut all_scores = HashMap::new();
        let mut page = 0;
        let max_per_page = 100;
        let mut _total_entries_fetched = 0;
        loop {
            let url = format!("{}/user/scenario/total-play", self.base_url);
            let response = match client
                .get(&url)
                .query(&[
                    ("username", username),
                    ("page", &page.to_string()),
                    ("max", &max_per_page.to_string()),
                    ("sort_param[]", "count"),
                ])
                .timeout(std::time::Duration::from_secs(30))
                .send() {
                    Ok(r) => r,
                    Err(e) => return Err(e.into()),
                };
            let status = response.status();
            if !status.is_success() {
                break;
            }
            let raw_json = match response.text() {
                Ok(json) => json,
                Err(e) => return Err(e.into()),
            };
            let raw_file_path = self.raw_scores_dir.join(format!("{}_page_{}.json", username, page));
            let _ = fs::write(&raw_file_path, &raw_json);
            let api_response: ApiResponse = match serde_json::from_str(&raw_json) {
                Ok(resp) => resp,
                Err(_) => break,
            };
            let data_len = api_response.data.len();
            _total_entries_fetched += data_len;
            if data_len == 0 {
                break;
            }
            let mut _page_scores_added = 0;
            for entry in api_response.data {
                let scenario = entry.scenario_name.trim().to_string();
                let score = entry.score.or_else(|| entry.attributes.as_ref().and_then(|a| a.score));
                if let Some(score) = score {
                    if !scenario.is_empty() {
                        let current_max: f64 = all_scores.get(&scenario).copied().unwrap_or(0.0);
                        all_scores.insert(scenario.clone(), current_max.max(score));
                        _page_scores_added += 1;
                    }
                }
            }
            page += 1;
            if page > 20 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(all_scores)
    }
}
