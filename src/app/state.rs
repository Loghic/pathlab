//! Application state and the [`eframe::App`] glue.

use eframe::egui;

use crate::mazes::{self, BuiltinMaze, Cell, MazeGrid};
use crate::platform::fileio::FileInbox;
use crate::platform::time::{Duration, Instant};
use crate::solver::{Algorithm, Heuristic, Solver, SolverStatus};

use super::undo::UndoStack;

/// User interactions with the canvas come in two flavours: editing walls
/// and picking start/end markers. They're mutually exclusive.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) enum InteractionMode {
    None,
    DrawWall,
    EraseWall,
    PickStart,
    PickEnd,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Point {
    pub x: usize,
    pub y: usize,
}

/// The root application object.
///
/// Keeping every field in one place makes it easy to see what is
/// serializable, what survives a hot reload, and what is purely
/// per-frame scratch space.
pub struct MazeApp {
    // ---- Grid -------------------------------------------------------
    pub(super) maze: MazeGrid,
    pub(super) selected_maze: BuiltinMaze,
    pub(super) rows: usize,
    pub(super) cols: usize,

    // ---- Rendering --------------------------------------------------
    pub(super) cell_size: f32,
    pub(super) target_cell_size: f32,
    pub(super) last_canvas_size: egui::Vec2,
    pub(super) auto_fit: bool,

    // ---- Solver settings -------------------------------------------
    pub(super) start: Point,
    pub(super) end: Point,
    pub(super) algorithm: Algorithm,
    pub(super) heuristic: Heuristic,
    pub(super) speed_ms: u32,
    /// How long the finished path is shown before loop-mode restarts
    /// the solve. Has no effect when [`Self::loop_mode`] is off.
    pub(super) loop_dwell_ms: u32,
    /// How many paths to find when [`Algorithm::KShortest`] is selected.
    pub(super) k_paths: usize,

    // ---- Solver runtime --------------------------------------------
    pub(super) solver: Option<Solver>,
    pub(super) last_step_time: Instant,
    pub(super) last_finish_time: Option<Instant>,
    pub(super) auto_run: bool,         // single-shot "Solve"
    pub(super) loop_mode: bool,        // restart after each finish
    pub(super) step_pending: bool,     // single-step request

    // ---- Interaction -----------------------------------------------
    pub(super) interaction: InteractionMode,

    // ---- File I/O --------------------------------------------------
    pub(super) file_inbox: FileInbox,
    pub(super) last_file_error: Option<String>,

    // ---- Undo ------------------------------------------------------
    pub(super) undo: UndoStack,
}

impl Default for MazeApp {
    fn default() -> Self {
        let maze = mazes::maze_starting();
        let rows = maze.len();
        let cols = maze.first().map(|r| r.len()).unwrap_or(0);
        Self {
            maze,
            selected_maze: BuiltinMaze::Starting,
            rows,
            cols,

            cell_size: 30.0,
            target_cell_size: 30.0,
            last_canvas_size: egui::vec2(800.0, 600.0),
            auto_fit: true,

            start: Point { x: 1, y: 1 },
            // The starting maze has a wall at (8, 8); (8, 7) sits one
            // block up in the outer corridor so the demo solves on
            // first launch.
            end: Point { x: 8, y: 7 },
            algorithm: Algorithm::AStar,
            heuristic: Heuristic::Manhattan,
            speed_ms: 100,
            loop_dwell_ms: 1000,
            k_paths: 5,

            solver: None,
            last_step_time: Instant::now(),
            last_finish_time: None,
            auto_run: false,
            loop_mode: false,
            step_pending: false,

            interaction: InteractionMode::None,

            file_inbox: FileInbox::new(),
            last_file_error: None,

            undo: UndoStack::new(),
        }
    }
}

impl MazeApp {
    // ------------------------------------------------------------------
    // Undo helpers
    // ------------------------------------------------------------------

    /// Snapshot the current maze onto the undo stack. Call BEFORE any
    /// bulk mutation (fill, invert, resize, preset switch, file load).
    /// Drag strokes call this once on `drag_started`, not per cell.
    pub(super) fn snapshot_for_undo(&mut self) {
        self.undo.push(&self.maze);
    }

    /// Roll back the last user action. Returns whether anything changed.
    pub(super) fn undo_last(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.maze = previous;
        // Restore dimensions to match the restored grid (resize ops may
        // have changed them).
        self.rows = self.maze.len();
        self.cols = self.maze.first().map(|r| r.len()).unwrap_or(0);
        self.clamp_endpoints();
        // The current solver run is no longer meaningful against the
        // previous maze; clear it so the user sees a clean state.
        self.solver = None;
        true
    }

    /// Replace the current maze and refit the viewport.
    pub(super) fn set_maze(&mut self, maze: MazeGrid) {
        self.snapshot_for_undo();
        self.rows = maze.len();
        self.cols = maze.first().map(|r| r.len()).unwrap_or(0);
        self.maze = maze;
        self.clamp_endpoints();
        self.solver = None;
        self.auto_fit_to_window(self.last_canvas_size);
    }

    pub(super) fn clamp_endpoints(&mut self) {
        let max_x = self.cols.saturating_sub(1);
        let max_y = self.rows.saturating_sub(1);
        self.start.x = self.start.x.min(max_x);
        self.start.y = self.start.y.min(max_y);
        self.end.x = self.end.x.min(max_x);
        self.end.y = self.end.y.min(max_y);
    }

    pub(super) fn auto_fit_to_window(&mut self, available: egui::Vec2) {
        if !self.auto_fit || self.rows == 0 || self.cols == 0 {
            return;
        }
        let nx = available.x / self.cols as f32;
        let ny = available.y / self.rows as f32;
        self.target_cell_size = nx.min(ny).clamp(5.0, 100.0);
    }

    pub(super) fn invert_maze(&mut self) {
        self.snapshot_for_undo();
        for row in &mut self.maze {
            for cell in row {
                *cell = cell.invert();
            }
        }
    }

    pub(super) fn fill_with(&mut self, cell: Cell) {
        self.snapshot_for_undo();
        for row in &mut self.maze {
            for c in row {
                *c = cell;
            }
        }
    }

    pub(super) fn add_row(&mut self, cell: Cell) {
        self.snapshot_for_undo();
        self.maze.push(vec![cell; self.cols]);
        self.rows += 1;
    }

    pub(super) fn add_col(&mut self, cell: Cell) {
        self.snapshot_for_undo();
        for row in &mut self.maze {
            row.push(cell);
        }
        self.cols += 1;
    }

    pub(super) fn remove_row(&mut self) {
        if self.rows > 1 {
            self.snapshot_for_undo();
            self.maze.pop();
            self.rows -= 1;
            self.clamp_endpoints();
        }
    }

    pub(super) fn remove_col(&mut self) {
        if self.cols > 1 {
            self.snapshot_for_undo();
            for row in &mut self.maze {
                row.pop();
            }
            self.cols -= 1;
            self.clamp_endpoints();
        }
    }

    pub(super) fn new_solver(&self) -> Solver {
        Solver::new(
            self.algorithm,
            self.heuristic,
            (self.start.x, self.start.y),
            (self.end.x, self.end.y),
        )
        .with_k(self.k_paths)
    }

    pub(super) fn start_solve(&mut self) {
        self.solver = Some(self.new_solver());
        self.auto_run = true;
        self.step_pending = false;
        self.last_step_time = Instant::now();
        self.last_finish_time = None;
    }

    pub(super) fn request_step(&mut self) {
        if self.solver.is_none() {
            self.solver = Some(self.new_solver());
            self.last_finish_time = None;
        }
        self.step_pending = true;
    }

    pub(super) fn clear_solver(&mut self) {
        self.solver = None;
        self.auto_run = false;
        self.step_pending = false;
        self.last_finish_time = None;
    }

    /// Drive the solver from inside the main update loop. Returns
    /// whether the UI should request another repaint.
    pub(super) fn tick_solver(&mut self) -> bool {
        // Pull any file the picker delivered.
        if let Some(result) = self.file_inbox.take() {
            match result {
                Ok(maze) => {
                    self.set_maze(maze);
                    self.last_file_error = None;
                }
                Err(e) => self.last_file_error = Some(e),
            }
        }

        let speed = Duration::from_millis(self.speed_ms as u64);
        let dwell = Duration::from_millis(self.loop_dwell_ms as u64);
        let now = Instant::now();
        let mut want_repaint = false;

        // ----- Phase 1: advance or finalise the existing solver --------
        //
        // Move the solver out so we can call `solver.step(&self.maze)`
        // without simultaneously holding a mutable borrow of `self`.
        let mut solver = self.solver.take();
        if let Some(s) = solver.as_mut() {
            if !s.finished() {
                if self.step_pending {
                    s.step(&self.maze);
                    self.step_pending = false;
                    want_repaint = true;
                } else if self.auto_run
                    && now.duration_since(self.last_step_time) >= speed
                {
                    s.step(&self.maze);
                    self.last_step_time = now;
                    want_repaint = true;
                } else if self.auto_run {
                    want_repaint = true; // keep ticking each frame
                }

                // The step we just ran may have transitioned the solver
                // into a finished state - record the timestamp NOW so
                // the dwell check in phase 2 can use it on this same
                // frame. Without this, phase 2 would see
                // `last_finish_time = None` and restart immediately,
                // skipping the dwell.
                if s.finished() && self.last_finish_time.is_none() {
                    self.last_finish_time = Some(now);
                    self.auto_run = false;
                }
            } else {
                // Already finished on a previous frame - nothing to do
                // here. `last_finish_time` was set then. If loop_mode is
                // on we still need to repaint so phase 2 fires.
                if self.loop_mode {
                    want_repaint = true;
                }
            }
        }
        self.solver = solver;

        // ----- Phase 2: optionally restart for loop mode ---------------
        //
        // This runs AFTER phase 1 so a solver that finished on this
        // frame has already stamped `last_finish_time`.
        if self.loop_mode {
            let should_restart = match &self.solver {
                None => true,
                Some(s) => s.finished(),
            };
            if should_restart {
                let ready = match self.last_finish_time {
                    // No finish recorded yet - this is the very first
                    // start of the loop, so go immediately.
                    None => self.solver.is_none(),
                    Some(t) => now.duration_since(t) >= dwell,
                };
                if ready {
                    self.solver = Some(self.new_solver());
                    self.last_finish_time = None;
                    self.auto_run = true;
                    self.last_step_time = now;
                    want_repaint = true;
                } else {
                    // Dwell still elapsing - keep redrawing so the
                    // timer keeps progressing each frame.
                    want_repaint = true;
                }
            }
        }

        want_repaint
    }

    /// Convenience for the UI - status of the active solver.
    pub(super) fn solver_status_label(&self) -> &'static str {
        match self.solver.as_ref().map(|s| s.status()) {
            None => "idle",
            Some(SolverStatus::Running) => "running",
            Some(SolverStatus::Found) => "path found",
            Some(SolverStatus::NoPath) => "no path",
        }
    }
}

// -----------------------------------------------------------------
// eframe glue
// -----------------------------------------------------------------
impl eframe::App for MazeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global Ctrl/Cmd+Z hotkey. `consume_shortcut` returns true once
        // per press and clears the event so other widgets don't see it.
        let undo_shortcut =
            egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
        if ctx.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
            self.undo_last();
        }

        super::top_bar::show(self, ctx);
        super::side_panel::show(self, ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            super::canvas::show(self, ui);
        });

        if self.tick_solver() {
            ctx.request_repaint();
        }
    }
}
