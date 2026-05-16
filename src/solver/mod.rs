//! Pluggable pathfinding algorithms with step-by-step visualization support.
//!
//! The [`Solver`] type drives execution one node at a time so the UI can
//! redraw between steps. Algorithms share the same step interface but use
//! different data structures internally:
//!
//! | Algorithm | Frontier | Order |
//! |-----------|----------|-------|
//! | DFS       | Stack    | LIFO  |
//! | BFS       | Queue    | FIFO  |
//! | A*        | Min-heap | by `f = g + h` |
//!
//! See `docs/algorithms.md` for the math.

mod algorithm;
mod heuristic;
mod k_paths;
mod solver;

pub use algorithm::Algorithm;
pub use heuristic::Heuristic;
pub use solver::{Solver, SolverStatus};
