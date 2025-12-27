use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioValidationCache {
    #[serde(skip)]
    file_path: PathBuf,
    cache: HashMap<String, bool>,
}
#[allow(dead_code)]
impl ScenarioValidationCache {
    pub fn new() -> Result<Self> {
        let app_data_dir = crate::backend::get_app_data_dir();
        let _ = fs::create_dir_all(&app_data_dir);
        let file_path = app_data_dir.join("scenario_validation_cache.json");
        let mut cache = Self {
            file_path,
            cache: HashMap::new(),
        };
        let _ = cache.load();
        Ok(cache)
    }
    fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {

            return Ok(());
        }
        let contents = fs::read_to_string(&self.file_path)?;
        if contents.trim().is_empty() {

            return Ok(());
        }
        let cache: HashMap<String, bool> = serde_json::from_str(&contents)?;
        self.cache = cache;
        Ok(())
    }
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.cache)?;
        let mut tmp = self.file_path.clone();
        tmp.set_extension("tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, &self.file_path)?;
        Ok(())
    }
    pub fn is_cached(&self, scenario_name: &str) -> Option<bool> {
        self.cache.get(scenario_name).copied()
    }
    pub fn insert(&mut self, scenario_name: &str, is_valid: bool) -> Result<()> {
        self.cache.insert(scenario_name.to_string(), is_valid);
        self.save()?;

        Ok(())
    }
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}