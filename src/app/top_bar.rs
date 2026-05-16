//! Top menu bar (File, Theme).

use eframe::egui;

use super::state::MazeApp;
use crate::platform::fileio;

pub(super) fn show(app: &mut MazeApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open PBM…").clicked() {
                    fileio::open_maze(&app.file_inbox);
                    ui.close();
                }
                if ui.button("Save as PBM…").clicked() {
                    fileio::save_maze(&app.maze);
                    ui.close();
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

            ui.separator();
            egui::widgets::global_theme_preference_buttons(ui);

            if let Some(err) = &app.last_file_error {
                ui.separator();
                ui.colored_label(egui::Color32::LIGHT_RED, format!("file error: {err}"));
            }
        });
    });
}
