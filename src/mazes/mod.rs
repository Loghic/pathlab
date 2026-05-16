//! Maze data types, built-in generators, and PBM file I/O.
//!
//! A maze is simply a `Vec<Vec<Cell>>` where each [`Cell`] is either
//! [`Cell::Empty`] (walkable) or [`Cell::Wall`] (blocked). Rows index the
//! Y axis, columns index the X axis - i.e. `maze[y][x]`.

mod cell;
mod generators;
mod pbm;

pub use cell::{Cell, MazeGrid};
pub use generators::{
    list_builtin, maze_boxed, maze_open, maze_starting, maze_wall_split, BuiltinMaze,
};
pub use pbm::{maze_from_pbm_str, maze_to_pbm_str};

#[cfg(not(target_arch = "wasm32"))]
pub use pbm::maze_from_pbm_path;
