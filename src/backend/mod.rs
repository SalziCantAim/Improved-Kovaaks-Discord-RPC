pub mod autostart;
pub mod config;
pub mod discord_rpc;
pub mod kovaaks_utils;
pub mod local_scores;
pub mod online_api;
pub mod scenario_cache;
pub use autostart::*;
pub use config::*;
pub use discord_rpc::DiscordRPC;
pub use kovaaks_utils::*;
pub use local_scores::*;
pub use online_api::OnlineScoreAPI;
pub use scenario_cache::ScenarioValidationCache;
use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("KovaaksDiscordRPC")
}