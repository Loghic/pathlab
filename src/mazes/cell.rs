//! Basic cell type used by every maze.

/// A single tile in a maze.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    /// Walkable.
    Empty,
    /// Blocked - the solver may not step here.
    Wall,
}

impl Cell {
    /// Returns `true` if a solver may step onto this cell.
    #[inline]
    pub fn is_walkable(self) -> bool {
        matches!(self, Cell::Empty)
    }

    /// Flips the cell between [`Cell::Empty`] and [`Cell::Wall`].
    #[inline]
    pub fn invert(self) -> Self {
        match self {
            Cell::Empty => Cell::Wall,
            Cell::Wall => Cell::Empty,
        }
    }
}

/// A maze is a rectangular grid of [`Cell`]s indexed `maze[y][x]`.
pub type MazeGrid = Vec<Vec<Cell>>;
