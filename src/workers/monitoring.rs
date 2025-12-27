use crate::backend::{
    find_fight_time_and_score, find_initial_scores, get_current_scenario, get_playlist_share_code,
    get_stats_directory, is_kovaaks_running, normalize_scenario_name, ScoreSource,
};
use crate::state::{AppState, UiUpdate};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
pub fn start_monitoring_thread(state: Arc<AppState>) {

    loop {

        if !state.rpc_running.load(Ordering::Relaxed) {

            break;
        }

        if state.sync_in_progress.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        let kovaaks_running = is_kovaaks_running();
        let was_running = state.kovaaks_was_running.swap(kovaaks_running, Ordering::Relaxed);
        if kovaaks_running {

            if !was_running {

                *state.session_start_time.lock() = std::time::SystemTime::now();
                state.session_best_scores.lock().clear();
                state.checked_files.lock().clear();
            }

            {
                let mut rpc_guard = state.rpc.lock();
                if let Some(rpc) = rpc_guard.as_mut() {
                    if !rpc.is_connected() {

                        if rpc.connect().is_err() {
                            thread::sleep(Duration::from_secs(5));
                            continue;
                        }
                    }
                }
            }

            let scenario = match get_current_scenario() {
                Ok(s) => normalize_scenario_name(&s),
                Err(_) => {
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };
            if scenario.is_empty() || scenario == "Unknown Scenario" {
                thread::sleep(Duration::from_secs(5));
                continue;
            }

            if !state.is_scenario_allowed(&scenario) {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
            let current = state.current_scenario.lock().clone();
            let scenario_changed = current != scenario;
            if scenario_changed {

                *state.current_scenario.lock() = scenario.clone();

                let settings = state.settings.lock().clone();
                let stats_dir = get_stats_directory(&settings);

                let cached_highscore = state.get_score_for_scenario(&scenario);
                *state.local_highscore.lock() = cached_highscore;

                let session_best = state.session_best_scores.lock()
                    .get(&scenario)
                    .copied()
                    .unwrap_or(0.0);
                *state.session_highscore.lock() = session_best;

                if let Ok((initial_score, files)) = find_initial_scores(&scenario, &stats_dir) {
                    if initial_score > cached_highscore {
                        *state.local_highscore.lock() = initial_score;

                        let _ = state.local_scores_manager.update_score(
                            &scenario,
                            initial_score,
                            None,
                            ScoreSource::Local,
                        );
                    }
                    *state.checked_files.lock() = files;
                }
                state.send_ui_update(UiUpdate::ScenarioChanged {
                    name: scenario.clone(),
                    highscore: *state.local_highscore.lock(),
                    session_best: *state.session_highscore.lock(),
                });
            }

            let settings = state.settings.lock().clone();
            let stats_dir = get_stats_directory(&settings);
            let checked = state.checked_files.lock().clone();
            if let Ok((new_score, found_new, last_played)) =
                find_fight_time_and_score(&scenario, &stats_dir, &checked)
            {
                if found_new && new_score > 0.0 {

                    if let Ok((_, new_files)) = find_initial_scores(&scenario, &stats_dir) {
                        *state.checked_files.lock() = new_files;
                    }

                    {
                        let mut session_bests = state.session_best_scores.lock();
                        let current_session_best = session_bests.get(&scenario).copied().unwrap_or(0.0);
                        if new_score > current_session_best {
                            session_bests.insert(scenario.clone(), new_score);
                            *state.session_highscore.lock() = new_score;
                        }
                    }

                    let current_high = *state.local_highscore.lock();
                    if new_score > current_high {
                        *state.local_highscore.lock() = new_score;

                        let _ = state.local_scores_manager.update_score(
                            &scenario,
                            new_score,
                            last_played,
                            ScoreSource::Local,
                        );

                        if let Ok(all_scores) = state.local_scores_manager.get_all_scores() {
                            *state.score_cache.lock() = all_scores;
                        }
                    }
                }
            }

            {
                let mut rpc_guard = state.rpc.lock();
                if let Some(rpc) = rpc_guard.as_mut() {
                    let highscore = *state.local_highscore.lock();
                    let session_best = *state.session_highscore.lock();
                    let start_time = *state.start_time.lock();
                    let share_code = get_playlist_share_code(&settings.installation_path);
                    let _ = rpc.update_presence(
                        &scenario,
                        start_time,
                        highscore,
                        session_best,
                        &settings.installation_path,
                        share_code,
                    );
                }
            }
        } else {

            if was_running {

                {
                    let mut rpc_guard = state.rpc.lock();
                    if let Some(rpc) = rpc_guard.as_mut() {
                        let _ = rpc.clear_presence();
                    }
                }

                *state.current_scenario.lock() = String::new();
                *state.local_highscore.lock() = 0.0;
                *state.session_highscore.lock() = 0.0;
                state.send_ui_update(UiUpdate::ScenarioChanged {
                    name: String::new(),
                    highscore: 0.0,
                    session_best: 0.0,
                });
            }
        }
        thread::sleep(Duration::from_secs(10));
    }

}