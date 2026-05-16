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

    // Outer border. The top and bottom rows are filled with walls;
    // the left and right columns of every row likewise.
    maze[0].fill(Cell::Wall);
    maze[rows - 1].fill(Cell::Wall);
    for line in maze.iter_mut() {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: assert that every row of a maze has the same length, and
    /// returns `(rows, cols)`.
    fn rectangular_dims(maze: &MazeGrid) -> (usize, usize) {
        let rows = maze.len();
        let cols = maze.first().map(|r| r.len()).unwrap_or(0);
        for (i, row) in maze.iter().enumerate() {
            assert_eq!(row.len(), cols, "row {i} has irregular width");
        }
        (rows, cols)
    }

    // ---- maze_open --------------------------------------------------

    #[test]
    fn open_has_requested_dimensions() {
        let m = maze_open(7, 4);
        assert_eq!(rectangular_dims(&m), (7, 4));
    }

    #[test]
    fn open_is_all_empty() {
        let m = maze_open(5, 5);
        for row in &m {
            for &cell in row {
                assert_eq!(cell, Cell::Empty);
            }
        }
    }

    #[test]
    fn open_clamps_zero_to_one() {
        // Zero-sized mazes break the renderer; the generator clamps up
        // to the smallest non-empty grid. Pinning the behaviour because
        // the UI relies on `rows.max(1)`.
        let m = maze_open(0, 0);
        assert_eq!(rectangular_dims(&m), (1, 1));
    }

    // ---- maze_wall_split -------------------------------------------

    #[test]
    fn wall_split_has_a_gap_in_the_middle_row() {
        let rows = 5;
        let cols = 7;
        let m = maze_wall_split(rows, cols);
        let mid_col = cols / 2;
        let mid_row = rows / 2;
        // Every row except the middle one has a wall at the centre column.
        for (y, row) in m.iter().enumerate() {
            if y == mid_row {
                assert_eq!(row[mid_col], Cell::Empty, "gap row should be empty");
            } else {
                assert_eq!(row[mid_col], Cell::Wall, "row {y} centre should be a wall");
            }
        }
    }

    #[test]
    fn wall_split_is_solvable_through_the_gap() {
        // Sanity: BFS should be able to traverse from left to right.
        use crate::solver::{Algorithm, Heuristic, Solver, SolverStatus};
        let m = maze_wall_split(5, 5);
        let mut s = Solver::new(Algorithm::BFS, Heuristic::Manhattan, (0, 0), (4, 0));
        for _ in 0..200 {
            s.step(&m);
            if s.finished() {
                break;
            }
        }
        assert_eq!(s.status(), SolverStatus::Found);
    }

    // ---- maze_boxed ------------------------------------------------

    #[test]
    fn boxed_has_full_border_of_walls() {
        let rows = 8;
        let cols = 10;
        let m = maze_boxed(rows, cols);
        assert_eq!(rectangular_dims(&m), (rows, cols));

        // Top and bottom rows are entirely walls.
        assert!(
            m.first().unwrap().iter().all(|&c| c == Cell::Wall),
            "top border"
        );
        assert!(
            m.last().unwrap().iter().all(|&c| c == Cell::Wall),
            "bottom border"
        );

        // Left and right columns of every row are walls.
        for (y, row) in m.iter().enumerate() {
            assert_eq!(row[0], Cell::Wall, "row {y} left border");
            assert_eq!(*row.last().unwrap(), Cell::Wall, "row {y} right border");
        }
    }

    #[test]
    fn boxed_clamps_tiny_input_to_minimum() {
        // Smaller than 3x3 would have no interior; the generator floors
        // both dimensions at 3. Pin this so the UI's resize buttons
        // stay safe.
        let m = maze_boxed(1, 1);
        assert_eq!(rectangular_dims(&m), (3, 3));
    }

    // ---- maze_starting ---------------------------------------------

    #[test]
    fn starting_maze_dimensions_are_pinned() {
        // The default app launches into this and its endpoints assume
        // these dimensions, so changing them is a UX-breaking change.
        let m = maze_starting();
        assert_eq!(rectangular_dims(&m), (10, 10));
    }

    #[test]
    fn starting_maze_has_default_endpoints_walkable() {
        // The MazeApp::default impl puts start at (1,1) and end at (8,7).
        // If either lands on a wall, the demo won't solve. Pin it.
        let m = maze_starting();
        assert_eq!(m[1][1], Cell::Empty, "start cell must be walkable");
        assert_eq!(m[7][8], Cell::Empty, "end cell must be walkable");
    }

    // ---- BuiltinMaze dispatch --------------------------------------

    #[test]
    fn list_builtin_covers_every_variant() {
        // If someone adds a new BuiltinMaze variant but forgets to put
        // it in list_builtin, the UI dropdown silently misses it. This
        // test fails loudly when the count drifts.
        assert_eq!(list_builtin().len(), 4);
    }

    #[test]
    fn every_builtin_label_is_unique_and_non_empty() {
        let labels: Vec<&str> = list_builtin().iter().map(|m| m.label()).collect();
        for &l in &labels {
            assert!(!l.is_empty(), "empty label");
        }
        // Uniqueness — duplicate labels would confuse the dropdown.
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate label: {a}");
            }
        }
    }

    #[test]
    fn every_builtin_generates_a_non_empty_maze() {
        for &b in list_builtin() {
            let m = b.generate(10, 10);
            let (rows, cols) = rectangular_dims(&m);
            assert!(rows > 0 && cols > 0, "{:?} produced empty maze", b);
        }
    }
}
