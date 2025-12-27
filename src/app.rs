use crate::backend::{save_settings, DiscordRPC};
use crate::state::{AppState, TrayMessage, UiUpdate};
use crate::ui::{apply_dark_theme, render_main_tab, render_settings_tab, SettingsForm};
use crate::workers::start_monitoring_thread;
use eframe::egui::{self, RichText, ViewportCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_HIDE, SW_SHOW,
};
#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Main,
    Settings,
}
pub struct KovaaksApp {
    state: Arc<AppState>,
    active_tab: Tab,
    tray_rx: Receiver<TrayMessage>,
    ui_rx: Receiver<UiUpdate>,

    settings_form: SettingsForm,
    toast_message: Option<(String, Instant)>,
    is_syncing: bool,
    tray_icon: Option<tray_icon::TrayIcon>,
    should_exit: Arc<AtomicBool>,
    shutdown_tx: Option<std::sync::mpsc::Sender<()>>,
    tray_thread_handle: Option<std::thread::JoinHandle<()>>,
    #[cfg(windows)]
    window_handle: Option<HWND>,
}
impl KovaaksApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        state: Arc<AppState>,
        tray_rx: Receiver<TrayMessage>,
        ui_rx: Receiver<UiUpdate>,
        tray_icon: tray_icon::TrayIcon,
        shutdown_tx: std::sync::mpsc::Sender<()>,
        tray_thread_handle: std::thread::JoinHandle<()>,
    ) -> Self {
        let settings = state.settings.lock().clone();
        let settings_form = SettingsForm::from(&settings);
        Self {
            state,
            active_tab: Tab::Main,
            tray_rx,
            ui_rx,
            settings_form,
            toast_message: None,
            is_syncing: false,
            tray_icon: Some(tray_icon),
            should_exit: Arc::new(AtomicBool::new(false)),
            shutdown_tx: Some(shutdown_tx),
            tray_thread_handle: Some(tray_thread_handle),
            #[cfg(windows)]
            window_handle: None,
        }
    }

    #[cfg(windows)]
    fn get_window_handle(&mut self) -> Option<HWND> {
        if self.window_handle.is_none() {
            let title: Vec<u16> = "Kovaaks Discord RPC\0".encode_utf16().collect();
            if let Ok(hwnd) = unsafe { FindWindowW(None, windows::core::PCWSTR(title.as_ptr())) } {
                if !hwnd.is_invalid() {
                    self.window_handle = Some(hwnd);
                }
            }
        }
        self.window_handle
    }

    #[cfg(windows)]
    fn show_window(&mut self) {
        if let Some(hwnd) = self.get_window_handle() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }

    #[cfg(windows)]
    fn hide_window(&mut self) {
        if let Some(hwnd) = self.get_window_handle() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
    fn handle_tray_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.tray_rx.try_recv() {
            match msg {
                TrayMessage::Show => {
                    #[cfg(windows)]
                    self.show_window();
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                    ctx.request_repaint();
                }
                TrayMessage::StartRpc => {
                    self.start_rpc();
                }
                TrayMessage::StopRpc => {
                    self.stop_rpc();
                }
                TrayMessage::Quit => {
                    self.stop_rpc();
                    drop(self.tray_icon.take());
                    self.should_exit.store(true, Ordering::Relaxed);
                    if let Some(tx) = self.shutdown_tx.as_ref() {
                        let _ = tx.send(());
                    }
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                    std::process::exit(0);
                }
            }
        }
    }
    fn handle_ui_updates(&mut self) {
        while let Ok(update) = self.ui_rx.try_recv() {
            match update {
                UiUpdate::RpcStateChanged { .. } => {

                }
                UiUpdate::ScenarioChanged { .. } => {

                }
                UiUpdate::ScoresUpdated => {

                }
                UiUpdate::SyncProgress { message } => {
                    self.show_toast(&message);
                }
                UiUpdate::SyncComplete { success, message } => {
                    self.is_syncing = false;
                    if success {
                        self.settings_form.online_scores_synced = true;
                    }
                    self.show_toast(&message);
                }
                UiUpdate::Toast { message } => {
                    self.show_toast(&message);
                }
            }
        }
    }
    fn show_toast(&mut self, message: &str) {
        self.toast_message = Some((message.to_string(), Instant::now()));
    }
    fn start_rpc(&mut self) {
        if self.state.is_rpc_running() {
            return;
        }
        match DiscordRPC::new() {
            Ok(mut rpc) => {
                if let Err(e) = rpc.connect() {
                    self.show_toast(&format!("Failed to connect: {}", e));
                    return;
                }
                *self.state.rpc.lock() = Some(rpc);
                self.state.rpc_running.store(true, Ordering::Relaxed);

                let now = chrono::Utc::now().timestamp();
                *self.state.start_time.lock() = Some(now);

                *self.state.session_start_time.lock() = std::time::SystemTime::now();
                self.state.session_best_scores.lock().clear();
                self.state.checked_files.lock().clear();

                let state_clone = self.state.clone();
                std::thread::spawn(move || {
                    start_monitoring_thread(state_clone);
                });
                self.show_toast("Discord RPC started");
            }
            Err(e) => {
                self.show_toast(&format!("Failed to create RPC: {}", e));
            }
        }
    }
    fn stop_rpc(&mut self) {
        if !self.state.is_rpc_running() {
            return;
        }
        self.state.rpc_running.store(false, Ordering::Relaxed);
        if let Some(mut rpc) = self.state.rpc.lock().take() {
            let _ = rpc.clear_presence();
            let _ = rpc.disconnect();
        }
        *self.state.start_time.lock() = None;
        *self.state.current_scenario.lock() = String::new();
        *self.state.local_highscore.lock() = 0.0;
        *self.state.session_highscore.lock() = 0.0;
        self.show_toast("Discord RPC stopped");
    }
    fn scan_local_stats(&mut self) {
        let settings = self.state.settings.lock().clone();
        let stats_dir = std::path::PathBuf::from(&settings.installation_path).join("stats");
        if !stats_dir.exists() {
            self.show_toast("Stats folder not found");
            return;
        }
        match crate::backend::scan_all_stats_folder(&stats_dir) {
            Ok(scores) => {
                let count = scores.len();
                if let Err(e) = self.state.local_scores_manager.populate_from_stats_folder(scores) {
                    self.show_toast(&format!("Failed to save scores: {}", e));
                } else {

                    if let Ok(all_scores) = self.state.local_scores_manager.get_all_scores() {
                        *self.state.score_cache.lock() = all_scores;
                    }
                    self.show_toast(&format!("Imported {} scenarios", count));
                }
            }
            Err(e) => {
                self.show_toast(&format!("Scan failed: {}", e));
            }
        }
    }
    fn sync_online_scores(&mut self) {
        if self.is_syncing {
            return;
        }
        let username = self.settings_form.webapp_username.clone();
        if username.is_empty() {
            self.show_toast("Please enter a username first");
            return;
        }
        self.is_syncing = true;
        self.state.sync_in_progress.store(true, Ordering::Relaxed);
        let state = self.state.clone();
        std::thread::spawn(move || {
            match state.online_api.fetch_user_scenario_scores(&username) {
                Ok(online_scores) => {
                    let count = online_scores.len();
                    *state.online_scores.lock() = online_scores.clone();
                    if let Err(e) = state.local_scores_manager.merge_online_scores(online_scores) {
                        state.send_ui_update(UiUpdate::SyncComplete {
                            success: false,
                            message: format!("Failed to merge scores: {}", e),
                        });
                    } else {

                        if let Ok(all_scores) = state.local_scores_manager.get_all_scores() {
                            *state.score_cache.lock() = all_scores;
                        }

                        {
                            let mut settings = state.settings.lock();
                            settings.online_scores_synced = true;
                            settings.last_sync_time = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let _ = save_settings(&settings);
                        }
                        state.send_ui_update(UiUpdate::SyncComplete {
                            success: true,
                            message: format!("Synced {} scenarios", count),
                        });
                    }
                }
                Err(e) => {
                    state.send_ui_update(UiUpdate::SyncComplete {
                        success: false,
                        message: format!("Sync failed: {}", e),
                    });
                }
            }
            state.sync_in_progress.store(false, Ordering::Relaxed);
        });
    }
    fn reset_sync_flag(&mut self) {
        self.settings_form.online_scores_synced = false;
        {
            let mut settings = self.state.settings.lock();
            settings.online_scores_synced = false;
            let _ = save_settings(&settings);
        }
        self.show_toast("Sync flag reset");
    }
    fn save_settings(&mut self) {
        let last_sync_time = self.state.settings.lock().last_sync_time;
        let new_settings = self.settings_form.to_settings(last_sync_time);
        if let Err(e) = save_settings(&new_settings) {
            self.show_toast(&format!("Failed to save: {}", e));
            return;
        }
        *self.state.settings.lock() = new_settings;
        self.show_toast("Settings saved");
    }
    fn render_navbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            let main_selected = self.active_tab == Tab::Main;
            let main_text = RichText::new("Main")
                .size(14.0)
                .color(if main_selected {
                    crate::ui::TEXT_WHITE
                } else {
                    crate::ui::TEXT_MUTED
                });
            if ui.add(egui::Button::new(main_text).frame(false)).clicked() {
                self.active_tab = Tab::Main;
            }
            ui.add_space(16.0);
            let settings_selected = self.active_tab == Tab::Settings;
            let settings_text = RichText::new("Settings")
                .size(14.0)
                .color(if settings_selected {
                    crate::ui::TEXT_WHITE
                } else {
                    crate::ui::TEXT_MUTED
                });
            if ui.add(egui::Button::new(settings_text).frame(false)).clicked() {
                self.active_tab = Tab::Settings;
            }
        });
        ui.separator();
    }
    fn render_toast(&self, ctx: &egui::Context) {
        if let Some((message, shown_at)) = &self.toast_message {
            let elapsed = shown_at.elapsed();
            if elapsed < Duration::from_secs(3) {
                egui::Area::new(egui::Id::new("toast"))
                    .fixed_pos(egui::pos2(
                        ctx.screen_rect().center().x - 100.0,
                        ctx.screen_rect().max.y - 60.0,
                    ))
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(crate::ui::BG_DARK)
                            .stroke(egui::Stroke::new(1.0, crate::ui::BORDER_SUBTLE))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(message).size(13.0).color(crate::ui::TEXT_WHITE));
                            });
                    });
            }
        }
    }
}
impl eframe::App for KovaaksApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.should_exit.load(Ordering::Relaxed) {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                #[cfg(windows)]
                self.hide_window();
            }
        }

        self.handle_tray_messages(ctx);
        self.handle_ui_updates();

        ctx.request_repaint_after(Duration::from_millis(100));

        apply_dark_theme(ctx);

        if let Some((_, shown_at)) = &self.toast_message {
            if shown_at.elapsed() > Duration::from_secs(3) {
                self.toast_message = None;
            }
        }

        egui::TopBottomPanel::top("navbar").show(ctx, |ui| {
            ui.add_space(8.0);
            self.render_navbar(ui);
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Main => {
                    let action = render_main_tab(ui, &self.state);
                    if action.start_rpc {
                        self.start_rpc();
                    }
                    if action.stop_rpc {
                        self.stop_rpc();
                    }
                    if action.minimize {
                        #[cfg(windows)]
                        self.hide_window();
                    }
                }
                Tab::Settings => {
                    let action = render_settings_tab(ui, &mut self.settings_form, self.is_syncing);
                    if action.scan_stats {
                        self.scan_local_stats();
                    }
                    if action.sync_online {
                        self.sync_online_scores();
                    }
                    if action.reset_sync {
                        self.reset_sync_flag();
                    }
                    if action.save {
                        self.save_settings();
                    }
                }
            }
        });

        self.render_toast(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_rpc();
        drop(self.tray_icon.take());
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if let Some(handle) = self.tray_thread_handle.take() {
            let _ = handle.join();
        }
    }
}