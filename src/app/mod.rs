//! egui front-end split across small files: state, UI panels, canvas.

mod canvas;
mod side_panel;
mod state;
mod top_bar;
mod undo;

pub use state::MazeApp;
