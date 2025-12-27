use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn normalize_scenario_name(name: &str) -> String {
    name.trim_end_matches(" - Challenge").to_string()
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ScoreSource {
    Local,
    Online,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioScore {
    pub scenario_name: String,
    pub highscore: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_played: Option<u64>,
    pub source: ScoreSource,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalScoresFile {
    pub version: u32,
    pub scores: HashMap<String, ScenarioScore>,
}
impl Default for LocalScoresFile {
    fn default() -> Self {
        Self {
            version: 1,
            scores: HashMap::new(),
        }
    }
}
pub struct LocalScoresManager {
    file_path: PathBuf,
}
#[allow(dead_code)]
impl LocalScoresManager {
    pub fn new() -> Result<Self> {
        let app_data_dir = crate::backend::get_app_data_dir();
        let _ = fs::create_dir_all(&app_data_dir);
        let file_path = app_data_dir.join("local_scores.json");
        Ok(Self { file_path })
    }
    pub fn load(&self) -> Result<LocalScoresFile> {
        if !self.file_path.exists() {

            return Ok(LocalScoresFile::default());
        }
        match fs::read_to_string(&self.file_path) {
            Ok(contents) => {
                if contents.trim().is_empty() {
                    return Ok(LocalScoresFile::default());
                }
                match serde_json::from_str::<LocalScoresFile>(&contents) {
                    Ok(data) => {
                        let mut normalized_scores = HashMap::new();
                        let mut migration_count = 0;
                        for (old_name, mut score) in data.scores.into_iter() {
                            let normalized_name = normalize_scenario_name(&old_name);
                            if normalized_name != old_name {
                                migration_count += 1;
                                score.scenario_name = normalized_name.clone();
                            }
                            if let Some(existing) = normalized_scores.get_mut(&normalized_name) {
                                let existing_score: &mut ScenarioScore = existing;
                                if score.highscore > existing_score.highscore {
                                    *existing_score = score;
                                }
                            } else {
                                normalized_scores.insert(normalized_name, score);
                            }
                        }
                        let migrated_data = LocalScoresFile {
                            version: data.version,
                            scores: normalized_scores,
                        };
                        if migration_count > 0 {
                            let _ = self.save(&migrated_data);
                        }
                        Ok(migrated_data)
                    }
                    Err(_) => {
                        let mut backup_path = self.file_path.clone();
                        backup_path.set_extension("bak");
                        let _ = fs::rename(&self.file_path, &backup_path);
                        Ok(LocalScoresFile::default())
                    }
                }
            }
            Err(_) => Ok(LocalScoresFile::default()),
        }
    }
    pub fn save(&self, data: &LocalScoresFile) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let mut tmp_path = self.file_path.clone();
        tmp_path.set_extension("tmp");
        fs::write(&tmp_path, &json)?;
        fs::rename(&tmp_path, &self.file_path)?;
        Ok(())
    }
    pub fn get_score(&self, scenario_name: &str) -> Result<Option<ScenarioScore>> {
        let data = self.load()?;
        Ok(data.scores.get(scenario_name).cloned())
    }
    pub fn update_score(
        &self,
        scenario_name: &str,
        new_score: f64,
        last_played: Option<SystemTime>,
        source: ScoreSource,
    ) -> Result<bool> {
        let normalized_name = normalize_scenario_name(scenario_name);
        let mut data = self.load()?;
        let last_played_timestamp = last_played.and_then(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs())
        });
        let mut is_new_highscore = false;
        if let Some(existing) = data.scores.get_mut(&normalized_name) {
            if new_score > existing.highscore {
                existing.highscore = new_score;
                existing.last_played = last_played_timestamp;
                existing.source = source;
                is_new_highscore = true;
            } else {
                if last_played_timestamp.is_some() {
                    existing.last_played = last_played_timestamp;
                }
            }
        } else {
            data.scores.insert(
                normalized_name.clone(),
                ScenarioScore {
                    scenario_name: normalized_name,
                    highscore: new_score,
                    last_played: last_played_timestamp,
                    source,
                },
            );
            is_new_highscore = true;
        }
        self.save(&data)?;
        Ok(is_new_highscore)
    }
    pub fn populate_from_stats_folder(&self, stats_scores: HashMap<String, (f64, Option<SystemTime>)>) -> Result<usize> {
        let mut data = self.load()?;
        let mut updated_count = 0;
        for (scenario_name, (highscore, last_played)) in stats_scores {
            let normalized_name = normalize_scenario_name(&scenario_name);
            let last_played_timestamp = last_played.and_then(|t| {
                t.duration_since(SystemTime::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            });
            if let Some(existing) = data.scores.get_mut(&normalized_name) {
                if highscore > existing.highscore {
                    existing.highscore = highscore;
                    existing.last_played = last_played_timestamp;
                    existing.source = ScoreSource::Local;
                    updated_count += 1;
                } else {
                    if last_played_timestamp.is_some() && existing.last_played != last_played_timestamp {
                        existing.last_played = last_played_timestamp;
                    }
                }
            } else {

                data.scores.insert(
                    normalized_name.clone(),
                    ScenarioScore {
                        scenario_name: normalized_name,
                        highscore,
                        last_played: last_played_timestamp,
                        source: ScoreSource::Local,
                    },
                );
                updated_count += 1;
            }
        }
        self.save(&data)?;

        Ok(updated_count)
    }
    pub fn merge_online_scores(&self, online_scores: HashMap<String, f64>) -> Result<usize> {
        let mut data = self.load()?;
        let mut updated_count = 0;
        for (scenario_name, online_score) in online_scores {
            if let Some(existing) = data.scores.get_mut(&scenario_name) {
                if online_score > existing.highscore {
                    existing.highscore = online_score;
                    existing.source = ScoreSource::Online;
                    updated_count += 1;
                }
            } else {

                data.scores.insert(
                    scenario_name.clone(),
                    ScenarioScore {
                        scenario_name,
                        highscore: online_score,
                        last_played: None,
                        source: ScoreSource::Online,
                    },
                );
                updated_count += 1;
            }
        }
        self.save(&data)?;

        Ok(updated_count)
    }
    pub fn get_all_scores(&self) -> Result<HashMap<String, ScenarioScore>> {
        let data = self.load()?;
        Ok(data.scores)
    }
    pub fn was_played_locally(&self, scenario_name: &str) -> Result<bool> {
        let data = self.load()?;
        Ok(data.scores.get(scenario_name).and_then(|s| s.last_played).is_some())
    }
    pub fn get_path(&self) -> &PathBuf {
        &self.file_path
    }
}