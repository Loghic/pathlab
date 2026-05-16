//! egui front-end split across small files: state, UI panels, canvas.

mod state;
mod side_panel;
mod canvas;
mod top_bar;
mod undo;

pub use state::MazeApp;
