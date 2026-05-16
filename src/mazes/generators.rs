//! Built-in maze generators.
//!
//! Each generator returns a fresh [`MazeGrid`]. To make adding more
//! generators painless, every preset is also reachable through the
//! [`BuiltinMaze`] enum and [`list_builtin`].

use super::cell::{Cell, MazeGrid};

/// Identifier for every built-in preset.
///
/// Adding a new built-in maze means:
///   1. Add a new variant here.
///   2. Add it to [`list_builtin`].
///   3. Add a match arm in [`BuiltinMaze::generate`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinMaze {
    Starting,
    Open,
    WallSplit,
    Boxed,
}

impl BuiltinMaze {
    /// Human-readable label used in the UI dropdown.
    pub fn label(self) -> &'static str {
        match self {
            BuiltinMaze::Starting => "Starting",
            BuiltinMaze::Open => "Open",
            BuiltinMaze::WallSplit => "Wall Split",
            BuiltinMaze::Boxed => "Boxed",
        }
    }

    /// Produce the maze, using `rows`/`cols` where the preset is parametric.
    /// Presets with a hard-coded size (e.g. [`BuiltinMaze::Starting`]) ignore
    /// the arguments.
    pub fn generate(self, rows: usize, cols: usize) -> MazeGrid {
        match self {
            BuiltinMaze::Starting => maze_starting(),
            BuiltinMaze::Open => maze_open(rows, cols),
            BuiltinMaze::WallSplit => maze_wall_split(rows, cols),
            BuiltinMaze::Boxed => maze_boxed(rows, cols),
        }
    }
}

/// Returns every built-in maze in the order they should appear in the UI.
pub fn list_builtin() -> &'static [BuiltinMaze] {
    &[
        BuiltinMaze::Starting,
        BuiltinMaze::Open,
        BuiltinMaze::WallSplit,
        BuiltinMaze::Boxed,
    ]
}

/// Completely empty maze - useful as a blank canvas for drawing walls.
pub fn maze_open(rows: usize, cols: usize) -> MazeGrid {
    vec![vec![Cell::Empty; cols.max(1)]; rows.max(1)]
}

/// A vertical wall down the middle with a single one-cell opening, useful
/// for showing how the heuristic affects path shape.
pub fn maze_wall_split(rows: usize, cols: usize) -> MazeGrid {
    let rows = rows.max(1);
    let cols = cols.max(1);
    let mut maze = vec![vec![Cell::Empty; cols]; rows];
    let mid = cols / 2;
    for (row, line) in maze.iter_mut().enumerate() {
        if row != rows / 2 {
            line[mid] = Cell::Wall;
        }
    }
    maze
}

/// Bordered maze with a regular grid of inner obstacles.
pub fn maze_boxed(rows: usize, cols: usize) -> MazeGrid {
    let rows = rows.max(3);
    let cols = cols.max(3);
    let mut maze = vec![vec![Cell::Empty; cols]; rows];

    // Outer border.
    for x in 0..cols {
        maze[0][x] = Cell::Wall;
        maze[rows - 1][x] = Cell::Wall;
    }
    for line in maze.iter_mut().take(rows) {
        line[0] = Cell::Wall;
        line[cols - 1] = Cell::Wall;
    }

    // Inner obstacles.
    for y in (2..rows - 2).step_by(2) {
        for x in (2..cols - 2).step_by(3) {
            maze[y][x] = Cell::Wall;
        }
    }
    maze
}

/// The hand-crafted demo maze shown on first launch.
pub fn maze_starting() -> MazeGrid {
    // 1 = walkable, 0 = wall. Mirrors the original layout.
    let layout = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 0, 0, 0, 0, 0, 1, 0],
        [0, 1, 0, 1, 1, 1, 1, 0, 1, 0],
        [0, 1, 0, 1, 0, 0, 1, 0, 1, 0],
        [0, 1, 0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
        [0, 1, 1, 1, 0, 1, 1, 1, 1, 0],
        [0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    layout
        .iter()
        .map(|row| {
            row.iter()
                .map(|&v| if v == 1 { Cell::Empty } else { Cell::Wall })
                .collect()
        })
        .collect()
}
