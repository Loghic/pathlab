//! # maze_solver
//!
//! A cross-platform pathfinding visualizer built on top of [`egui`].
//!
//! The crate exposes three high-level modules:
//!
//! - [`mazes`] - maze data structures and built-in generators / file parsers.
//! - [`solver`] - pluggable pathfinding algorithms (A*, BFS, DFS) with
//!   selectable heuristics for A*.
//! - [`app`] - the [`eframe::App`] implementation used by both the native
//!   binary and the WebAssembly target.
//!
//! See `docs/` in the repository root for an architecture overview.

pub mod mazes;
pub mod solver;
pub mod app;
pub mod platform;
