//! Bounded undo history for maze edits.
//!
//! Snapshots are full [`MazeGrid`] clones. A grid of 100×100 is ~10 KB, so
//! 100 snapshots is ~1 MB — fine. The cap exists mainly to keep memory
//! predictable when someone resizes to a 500×500 maze and goes wild.
//!
//! ## Granularity
//!
//! The app pushes one snapshot per *user action*, where a "drag stroke" of
//! the wall-draw tool counts as a single action. See
//! [`crate::app::canvas`] for where `push_if_new` is called.

use crate::mazes::MazeGrid;

const MAX_DEPTH: usize = 100;

pub struct UndoStack {
    history: Vec<MazeGrid>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Push a snapshot if the maze actually differs from the top of the
    /// stack. Cheap no-op otherwise, so call sites don't need to compare
    /// themselves.
    pub fn push(&mut self, maze: &MazeGrid) {
        if self.history.last().is_some_and(|top| top == maze) {
            return;
        }
        if self.history.len() >= MAX_DEPTH {
            self.history.remove(0);
        }
        self.history.push(maze.clone());
    }

    /// Pop the most recent snapshot. Returns `None` if there's nothing to
    /// undo.
    pub fn pop(&mut self) -> Option<MazeGrid> {
        self.history.pop()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    #[allow(dead_code)] // part of the public API for future callers
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mazes::{maze_open, Cell};

    #[test]
    fn pop_returns_pushed_snapshot() {
        let mut stack = UndoStack::new();
        let mut a = maze_open(2, 2);
        stack.push(&a);
        a[0][0] = Cell::Wall;
        let restored = stack.pop().expect("stack should not be empty");
        assert_eq!(restored[0][0], Cell::Empty);
    }

    #[test]
    fn duplicate_pushes_are_deduped() {
        let mut stack = UndoStack::new();
        let m = maze_open(3, 3);
        stack.push(&m);
        stack.push(&m);
        stack.push(&m);
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn pop_on_empty_returns_none() {
        let mut stack = UndoStack::new();
        assert!(stack.pop().is_none());
    }

    #[test]
    fn cap_drops_oldest() {
        let mut stack = UndoStack::new();
        // Push MAX_DEPTH + 5 distinct snapshots — vary the size so every
        // push is unique and survives the dedup check.
        for i in 0..MAX_DEPTH + 5 {
            let m = maze_open(2, 2 + i);
            stack.push(&m);
        }
        assert_eq!(stack.len(), MAX_DEPTH);
    }
}
