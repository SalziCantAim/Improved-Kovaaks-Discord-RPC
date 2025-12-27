use crate::backend::{
    config::Settings, DiscordRPC, LocalScoresManager, OnlineScoreAPI, ScenarioValidationCache,
    ScenarioScore,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::SystemTime;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UiUpdate {
    RpcStateChanged { running: bool },
    ScenarioChanged { name: String, highscore: f64, session_best: f64 },
    ScoresUpdated,
    SyncProgress { message: String },
    SyncComplete { success: bool, message: String },
    Toast { message: String },
}

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Show,
    StartRpc,
    StopRpc,
    Quit,
}

pub struct AppState {

    pub settings: Mutex<Settings>,

    pub rpc: Mutex<Option<DiscordRPC>>,
    pub rpc_running: AtomicBool,
    pub current_scenario: Mutex<String>,
    pub local_highscore: Mutex<f64>,
    pub session_highscore: Mutex<f64>,
    pub start_time: Mutex<Option<i64>>,
    pub checked_files: Mutex<Vec<String>>,

    pub online_api: OnlineScoreAPI,
    pub online_scores: Mutex<HashMap<String, f64>>,

    pub local_scores_manager: LocalScoresManager,
    pub score_cache: Mutex<HashMap<String, ScenarioScore>>,

    pub session_start_time: Mutex<SystemTime>,
    pub session_best_scores: Mutex<HashMap<String, f64>>,
    pub kovaaks_was_running: AtomicBool,

    pub scenario_validation_cache: Mutex<ScenarioValidationCache>,

    pub sync_in_progress: AtomicBool,

    pub ui_update_tx: Sender<UiUpdate>,
}
impl AppState {
    pub fn new(settings: Settings, ui_update_tx: Sender<UiUpdate>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let local_scores_manager = LocalScoresManager::new()?;
        let scenario_validation_cache = ScenarioValidationCache::new()?;

        let score_cache = local_scores_manager.get_all_scores().unwrap_or_default();
        Ok(Self {
            settings: Mutex::new(settings),
            rpc: Mutex::new(None),
            rpc_running: AtomicBool::new(false),
            current_scenario: Mutex::new(String::new()),
            local_highscore: Mutex::new(0.0),
            session_highscore: Mutex::new(0.0),
            start_time: Mutex::new(None),
            checked_files: Mutex::new(Vec::new()),
            online_api: OnlineScoreAPI::new(),
            online_scores: Mutex::new(HashMap::new()),
            local_scores_manager,
            score_cache: Mutex::new(score_cache),
            session_start_time: Mutex::new(SystemTime::now()),
            session_best_scores: Mutex::new(HashMap::new()),
            kovaaks_was_running: AtomicBool::new(false),
            scenario_validation_cache: Mutex::new(scenario_validation_cache),
            sync_in_progress: AtomicBool::new(false),
            ui_update_tx,
        })
    }
    pub fn is_rpc_running(&self) -> bool {
        self.rpc_running.load(Ordering::Relaxed)
    }
    pub fn get_current_scenario(&self) -> String {
        self.current_scenario.lock().clone()
    }
    pub fn get_local_highscore(&self) -> f64 {
        *self.local_highscore.lock()
    }
    pub fn get_session_highscore(&self) -> f64 {
        *self.session_highscore.lock()
    }
    pub fn send_ui_update(&self, update: UiUpdate) {
        let _ = self.ui_update_tx.send(update);
    }
    pub fn get_score_for_scenario(&self, scenario_name: &str) -> f64 {
        let cache = self.score_cache.lock();
        cache.get(scenario_name).map(|s| s.highscore).unwrap_or(0.0)
    }
    pub fn is_scenario_allowed(&self, scenario_name: &str) -> bool {
        let settings = self.settings.lock();
        if !settings.online_only_scenarios {
            return true;
        }

        let mut cache = self.scenario_validation_cache.lock();
        if let Some(is_valid) = cache.is_cached(scenario_name) {
            return is_valid;
        }

        let online_scores = self.online_scores.lock();
        if online_scores.contains_key(scenario_name) {
            let _ = cache.insert(scenario_name, true);
            return true;
        }

        if !settings.online_scores_synced {
            return true;
        }

        let _username = settings.webapp_username.clone();
        drop(settings);
        drop(online_scores);
        drop(cache);
        if let Ok(is_available) = self.online_api.search_scenario_popular(scenario_name) {
            let mut cache = self.scenario_validation_cache.lock();
            let _ = cache.insert(scenario_name, is_available);
            return is_available;
        }
        true
    }
}

pub fn create_ui_channel() -> (Sender<UiUpdate>, Receiver<UiUpdate>) {
    channel()
}

pub fn create_tray_channel() -> (Sender<TrayMessage>, Receiver<TrayMessage>) {
    channel()
}