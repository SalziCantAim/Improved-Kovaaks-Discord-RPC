use crate::state::AppState;
use crate::ui::theme::*;
use eframe::egui::{self, RichText};
use std::sync::Arc;
#[derive(Default)]
pub struct MainTabAction {
    pub start_rpc: bool,
    pub stop_rpc: bool,
    pub minimize: bool,
}
pub fn render_main_tab(ui: &mut egui::Ui, state: &Arc<AppState>) -> MainTabAction {
    let mut action = MainTabAction::default();
    let rpc_running = state.is_rpc_running();
    let current_scenario = state.get_current_scenario();
    let local_highscore = state.get_local_highscore();
    let session_highscore = state.get_session_highscore();

    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        let max_width = 600.0;

        ui.allocate_ui_with_layout(
            egui::vec2(max_width, 0.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                card_frame().show(ui, |ui| {
                    ui.set_max_width(max_width);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Kovaaks Discord RPC").size(28.0).color(TEXT_WHITE).strong());
                        ui.add_space(8.0);
                        ui.label(RichText::new("Display your Kovaak's gameplay on Discord").size(14.0).color(TEXT_MUTED));
                    });
                });
            },
        );
        ui.add_space(24.0);

        ui.allocate_ui_with_layout(
            egui::vec2(max_width, 0.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                card_frame().show(ui, |ui| {
                    ui.set_max_width(max_width);
                    ui.vertical_centered(|ui| {
                        let status_text = if rpc_running { "Running" } else { "Stopped" };
                        let full_text = format!("Discord RPC Status: {}", status_text);

                        // Measure text width dynamically
                        let text_width = ui.fonts(|f| {
                            f.layout_no_wrap(
                                full_text.clone(),
                                egui::FontId::proportional(16.0),
                                TEXT_WHITE,
                            ).size().x
                        });

                        let content_width = 12.0 + 8.0 + text_width;

                        ui.allocate_ui_with_layout(
                            egui::vec2(content_width, 0.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                status_dot(ui, rpc_running);
                                ui.add_space(8.0);
                                ui.label(RichText::new(full_text).size(16.0).color(TEXT_WHITE));
                            }
                        );

                        if rpc_running && !current_scenario.is_empty() && current_scenario != "Unknown Scenario" {
                            ui.add_space(20.0);
                            ui.label(RichText::new("Current Scenario").size(12.0).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            ui.label(RichText::new(&current_scenario).size(18.0).color(TEXT_WHITE).strong());

                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Highscore: {:.1}", local_highscore)).size(14.0).color(TEXT_MUTED));
                                ui.add_space(16.0);
                                if session_highscore > 0.0 {
                                    ui.label(RichText::new(format!("Session Best: {:.1}", session_highscore)).size(14.0).color(STATUS_GREEN));
                                } else {
                                    ui.label(RichText::new("No session plays yet").size(14.0).color(TEXT_MUTED));
                                }
                            });
                        } else if rpc_running {
                            ui.add_space(20.0);
                            ui.label(RichText::new("Waiting for scenario...").size(14.0).color(TEXT_MUTED));
                        }
                    });
                });
            },
        );
        ui.add_space(24.0);

        let button_group_width = 216.0;

        ui.allocate_ui_with_layout(
            egui::vec2(button_group_width, 0.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if rpc_running {
                    if styled_button(ui, "Stop RPC", true).clicked() {
                        action.stop_rpc = true;
                    }
                } else {
                    if styled_button(ui, "Start RPC", true).clicked() {
                        action.start_rpc = true;
                    }
                }
                ui.add_space(16.0);
                if styled_button(ui, "Minimize to Tray", false).clicked() {
                    action.minimize = true;
                }
            }
        );

        ui.add_space(32.0);

        ui.label(
            RichText::new("This application is not affiliated with the official Kovaaks team.")
                .size(11.0)
                .color(TEXT_DISABLED),
        );
    });
    action
}