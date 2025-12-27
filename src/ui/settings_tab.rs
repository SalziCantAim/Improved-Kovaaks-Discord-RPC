use crate::backend::{get_autostart_enabled, set_autostart_enabled, Settings};
use crate::ui::theme::*;
use eframe::egui::{self, RichText};
pub struct SettingsForm {
    pub installation_path: String,
    pub steam_path: String,
    pub open_manually: bool,
    pub start_with_windows: bool,
    pub webapp_username: String,
    pub show_online_scores: bool,
    pub start_in_tray: bool,
    pub online_only_scenarios: bool,
    pub online_scores_synced: bool,
}
impl From<&Settings> for SettingsForm {
    fn from(settings: &Settings) -> Self {
        Self {
            installation_path: settings.installation_path.clone(),
            steam_path: settings.steam_path.clone(),
            open_manually: settings.open_manually,
            start_with_windows: settings.start_with_windows,
            webapp_username: settings.webapp_username.clone(),
            show_online_scores: settings.show_online_scores,
            start_in_tray: settings.start_in_tray,
            online_only_scenarios: settings.online_only_scenarios,
            online_scores_synced: settings.online_scores_synced,
        }
    }
}
impl SettingsForm {
    pub fn to_settings(&self, last_sync_time: u64) -> Settings {
        Settings {
            installation_path: self.installation_path.clone(),
            steam_path: self.steam_path.clone(),
            open_manually: self.open_manually,
            start_with_windows: self.start_with_windows,
            webapp_username: self.webapp_username.clone(),
            show_online_scores: self.show_online_scores,
            start_in_tray: self.start_in_tray,
            online_only_scenarios: self.online_only_scenarios,
            online_scores_synced: self.online_scores_synced,
            last_sync_time,
        }
    }
}
#[derive(Default)]
pub struct SettingsTabAction {
    pub scan_stats: bool,
    pub sync_online: bool,
    pub reset_sync: bool,
    pub save: bool,
}
pub fn render_settings_tab(
    ui: &mut egui::Ui,
    form: &mut SettingsForm,
    is_syncing: bool,
) -> SettingsTabAction {
    let mut action = SettingsTabAction::default();
    egui::ScrollArea::vertical().show(ui, |ui| {
        let max_width = 600.0;
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            ui.allocate_ui_with_layout(
                egui::vec2(max_width, 0.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    card_frame().show(ui, |ui| {
                        ui.set_max_width(max_width);
                        section_header(ui, "RPC Settings");
                        styled_checkbox(ui, &mut form.open_manually, "Start Discord RPC manually");
                        ui.label(RichText::new("When disabled, RPC starts automatically when KovaaK's launches").size(11.0).color(TEXT_DISABLED));
                        ui.add_space(12.0);
                        let current_autostart = get_autostart_enabled();
                        let mut autostart = current_autostart;
                        if styled_checkbox(ui, &mut autostart, "Start with Windows").changed() {
                            let _ = set_autostart_enabled(autostart);
                            form.start_with_windows = autostart;
                        }
                        ui.add_space(12.0);
                        styled_checkbox(ui, &mut form.start_in_tray, "Start minimized to system tray");
                    });
                },
            );
            ui.add_space(20.0);

            ui.allocate_ui_with_layout(
                egui::vec2(max_width, 0.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    card_frame().show(ui, |ui| {
                        ui.set_max_width(max_width);
                        section_header(ui, "Online Features");
                        ui.label(RichText::new("Kovaak Webapp Username").size(13.0).color(TEXT_WHITE));
                        ui.add_space(4.0);
                        styled_text_edit(ui, &mut form.webapp_username, "Enter your username");
                        ui.add_space(12.0);
                        let has_username = !form.webapp_username.is_empty();
                        ui.add_enabled_ui(has_username, |ui| {
                            styled_checkbox(ui, &mut form.show_online_scores, "Show online scenario highscores");
                        });
                        ui.add_space(8.0);
                        styled_checkbox(ui, &mut form.online_only_scenarios, "Only show scenarios available online");
                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Scan Local Stats").size(14.0).color(TEXT_WHITE));
                                ui.label(RichText::new("Import scores from your stats folder").size(11.0).color(TEXT_MUTED));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if styled_button(ui, "Scan Stats", false).clicked() {
                                    action.scan_stats = true;
                                }
                            });
                        });
                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Sync Online Scores").size(14.0).color(TEXT_WHITE));
                                ui.label(RichText::new("Fetch highscores from Kovaak's webapp").size(11.0).color(TEXT_MUTED));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if form.online_scores_synced {
                                    if styled_button(ui, "Re-sync", false).clicked() {
                                        action.reset_sync = true;
                                    }
                                } else {
                                    let sync_enabled = has_username && !is_syncing;
                                    ui.add_enabled_ui(sync_enabled, |ui| {
                                        let text = if is_syncing { "Syncing..." } else { "Sync Now" };
                                        if styled_button(ui, text, true).clicked() {
                                            action.sync_online = true;
                                        }
                                    });
                                }
                            });
                        });
                    });
                },
            );
            ui.add_space(20.0);

            ui.allocate_ui_with_layout(
                egui::vec2(max_width, 0.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    card_frame().show(ui, |ui| {
                        ui.set_max_width(max_width);
                        section_header(ui, "Path Settings");
                        ui.label(RichText::new("FPSAimTrainer Installation Path").size(13.0).color(TEXT_WHITE));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.set_max_width(max_width);

                            let text_edit_width = max_width - 120.0;
                            ui.allocate_ui_with_layout(
                                egui::vec2(text_edit_width, 0.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.set_max_width(text_edit_width);
                                    styled_text_edit(ui, &mut form.installation_path, "C:\\...\\FPSAimTrainer");
                                }
                            );

                            if styled_button(ui, "Browse", false).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    form.installation_path = path.display().to_string();
                                }
                            }
                        });
                        ui.add_space(12.0);
                        ui.label(RichText::new("Steam.exe Path").size(13.0).color(TEXT_WHITE));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.set_max_width(max_width);

                            let text_edit_width = max_width - 120.0;
                            ui.allocate_ui_with_layout(
                                egui::vec2(text_edit_width, 0.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.set_max_width(text_edit_width);
                                    styled_text_edit(ui, &mut form.steam_path, "C:\\...\\steam.exe");
                                }
                            );

                            if styled_button(ui, "Browse", false).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Executable", &["exe"])
                                    .pick_file()
                                {
                                    form.steam_path = path.display().to_string();
                                }
                            }
                        });
                    });
                },
            );
            ui.add_space(24.0);

            if styled_button(ui, "Save Settings", true).clicked() {
                action.save = true;
            }
            ui.add_space(24.0);
        });
    });
    action
}