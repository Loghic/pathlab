//! Settings side panel (algorithm, heuristic, start/end, edit tools).

use eframe::egui;

use crate::mazes::{self, Cell};
use crate::solver::{Algorithm, Heuristic};

use super::state::{InteractionMode, MazeApp};

pub(super) fn show(app: &mut MazeApp, ctx: &egui::Context) {
    egui::SidePanel::right("settings_panel")
        .min_width(260.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                section_maze_preset(app, ui);
                section_endpoints(app, ui);
                section_view(app, ui);
                section_algorithm(app, ui);
                section_run(app, ui);
                section_editing(app, ui);
                section_resize(app, ui);

                ui.separator();
                if ui.button("Invert maze").clicked() {
                    app.invert_maze();
                }
            });
        });
}

fn section_maze_preset(app: &mut MazeApp, ui: &mut egui::Ui) {
    ui.heading("Maze");
    let previous = app.selected_maze;
    egui::ComboBox::from_id_salt("maze_preset")
        .selected_text(app.selected_maze.label())
        .show_ui(ui, |ui| {
            for &m in mazes::list_builtin() {
                ui.selectable_value(&mut app.selected_maze, m, m.label());
            }
        });
    if app.selected_maze != previous {
        let new_maze = app.selected_maze.generate(app.rows, app.cols);
        app.set_maze(new_maze);
    }
}

fn section_endpoints(app: &mut MazeApp, ui: &mut egui::Ui) {
    let active = ui.visuals().selection.bg_fill;
    let inactive = ui.visuals().widgets.inactive.bg_fill;

    ui.group(|ui| {
        ui.label("Start (x, y)");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut app.start.x).range(0..=app.cols.saturating_sub(1)));
            ui.add(egui::DragValue::new(&mut app.start.y).range(0..=app.rows.saturating_sub(1)));
            let col = if app.interaction == InteractionMode::PickStart {
                active
            } else {
                inactive
            };
            if ui.add(egui::Button::new("Pick").fill(col)).clicked() {
                app.interaction = if app.interaction == InteractionMode::PickStart {
                    InteractionMode::None
                } else {
                    InteractionMode::PickStart
                };
            }
        });
    });

    ui.group(|ui| {
        ui.label("End (x, y)");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut app.end.x).range(0..=app.cols.saturating_sub(1)));
            ui.add(egui::DragValue::new(&mut app.end.y).range(0..=app.rows.saturating_sub(1)));
            let col = if app.interaction == InteractionMode::PickEnd {
                active
            } else {
                inactive
            };
            if ui.add(egui::Button::new("Pick").fill(col)).clicked() {
                app.interaction = if app.interaction == InteractionMode::PickEnd {
                    InteractionMode::None
                } else {
                    InteractionMode::PickEnd
                };
            }
        });
    });
}

fn section_view(app: &mut MazeApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("View");
        ui.add(egui::Slider::new(&mut app.target_cell_size, 5.0..=100.0).text("cell px"));
        ui.horizontal(|ui| {
            if ui.button("Fit").clicked() {
                let nx = app.last_canvas_size.x / app.cols.max(1) as f32;
                let ny = app.last_canvas_size.y / app.rows.max(1) as f32;
                app.target_cell_size = nx.min(ny).clamp(5.0, 100.0);
            }
            ui.checkbox(&mut app.auto_fit, "Auto-fit");
        });
    });
}

fn section_algorithm(app: &mut MazeApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("Algorithm");
        egui::ComboBox::from_id_salt("algo")
            .selected_text(app.algorithm.label())
            .show_ui(ui, |ui| {
                for &a in Algorithm::all() {
                    ui.selectable_value(&mut app.algorithm, a, a.label());
                }
            });

        ui.add_enabled_ui(app.algorithm.uses_heuristic(), |ui| {
            egui::ComboBox::from_id_salt("heur")
                .selected_text(app.heuristic.label())
                .show_ui(ui, |ui| {
                    for &h in Heuristic::all() {
                        ui.selectable_value(&mut app.heuristic, h, h.label());
                    }
                });
        });

        // K-Shortest paths spinner. Only shown for that algorithm to
        // keep the panel uncluttered for the others.
        if app.algorithm.is_multi_path() {
            ui.horizontal(|ui| {
                ui.label("Paths to find (k):");
                ui.add(
                    egui::DragValue::new(&mut app.k_paths)
                        .range(1usize..=32)
                        .speed(1),
                );
            });
        }

        egui::ComboBox::from_id_salt("speed")
            .selected_text(format!("{} ms/step", app.speed_ms))
            .show_ui(ui, |ui| {
                for &s in &[10u32, 20, 50, 100, 200, 500, 1000] {
                    ui.selectable_value(&mut app.speed_ms, s, format!("{s} ms"));
                }
            });
    });
}

fn section_run(app: &mut MazeApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("Run");
        ui.horizontal(|ui| {
            if ui.button("Solve").clicked() {
                app.start_solve();
            }
            ui.add_enabled_ui(app.algorithm.supports_stepping(), |ui| {
                if ui.button("Step").clicked() {
                    app.request_step();
                }
            });
            if ui.button("Clear").clicked() {
                app.clear_solver();
            }
        });
        ui.checkbox(&mut app.loop_mode, "Loop");
        ui.add_enabled_ui(app.loop_mode, |ui| {
            egui::ComboBox::from_id_salt("loop_dwell")
                .selected_text(format_dwell(app.loop_dwell_ms))
                .show_ui(ui, |ui| {
                    for &ms in &[0u32, 250, 500, 1000, 2000, 3000, 5000] {
                        ui.selectable_value(&mut app.loop_dwell_ms, ms, format_dwell(ms));
                    }
                });
            ui.label("Pause on finished path");
        });
        ui.label(format!("Status: {}", app.solver_status_label()));
        if let Some(s) = &app.solver {
            if app.algorithm.is_multi_path() {
                ui.label(format!(
                    "paths found: {}   shortest: {}",
                    s.paths().len(),
                    s.path().len(),
                ));
            } else {
                ui.label(format!(
                    "open: {}   closed: {}   path: {}",
                    s.open_cells().len(),
                    s.closed_cells().len(),
                    s.path().len(),
                ));
            }
        }
    });
}

/// Format a loop-dwell value for the dropdown label.
fn format_dwell(ms: u32) -> String {
    if ms == 0 {
        "instant".to_string()
    } else if ms < 1000 {
        format!("{ms} ms")
    } else if ms.is_multiple_of(1000) {
        format!("{} s", ms / 1000)
    } else {
        format!("{:.1} s", ms as f32 / 1000.0)
    }
}

fn section_editing(app: &mut MazeApp, ui: &mut egui::Ui) {
    let active = ui.visuals().selection.bg_fill;
    let inactive = ui.visuals().widgets.inactive.bg_fill;

    ui.separator();
    ui.heading("Edit");

    ui.horizontal(|ui| {
        let col = if app.interaction == InteractionMode::DrawWall {
            active
        } else {
            inactive
        };
        if ui.add(egui::Button::new("Draw wall").fill(col)).clicked() {
            app.interaction = if app.interaction == InteractionMode::DrawWall {
                InteractionMode::None
            } else {
                InteractionMode::DrawWall
            };
        }
        let col = if app.interaction == InteractionMode::EraseWall {
            active
        } else {
            inactive
        };
        if ui.add(egui::Button::new("Erase wall").fill(col)).clicked() {
            app.interaction = if app.interaction == InteractionMode::EraseWall {
                InteractionMode::None
            } else {
                InteractionMode::EraseWall
            };
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Fill walls").clicked() {
            app.fill_with(Cell::Wall);
        }
        if ui.button("Clear maze").clicked() {
            app.fill_with(Cell::Empty);
        }
    });

    ui.horizontal(|ui| {
        let label = if app.undo.is_empty() {
            "Undo (Ctrl+Z)".to_string()
        } else {
            format!("Undo (Ctrl+Z)  [{}]", app.undo.len())
        };
        ui.add_enabled_ui(!app.undo.is_empty(), |ui| {
            if ui.button(label).clicked() {
                app.undo_last();
            }
        });
    });
}

fn section_resize(app: &mut MazeApp, ui: &mut egui::Ui) {
    ui.separator();
    ui.label("Resize");
    ui.horizontal(|ui| {
        if ui.button("+ row wall").clicked() {
            app.add_row(Cell::Wall);
            app.auto_fit_to_window(app.last_canvas_size);
        }
        if ui.button("+ col wall").clicked() {
            app.add_col(Cell::Wall);
            app.auto_fit_to_window(app.last_canvas_size);
        }
    });
    ui.horizontal(|ui| {
        if ui.button("+ row empty").clicked() {
            app.add_row(Cell::Empty);
            app.auto_fit_to_window(app.last_canvas_size);
        }
        if ui.button("+ col empty").clicked() {
            app.add_col(Cell::Empty);
            app.auto_fit_to_window(app.last_canvas_size);
        }
    });
    ui.horizontal(|ui| {
        if ui.button("- row").clicked() {
            app.remove_row();
            app.auto_fit_to_window(app.last_canvas_size);
        }
        if ui.button("- col").clicked() {
            app.remove_col();
            app.auto_fit_to_window(app.last_canvas_size);
        }
    });
}
