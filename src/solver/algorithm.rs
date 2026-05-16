//! Enum of supported pathfinding algorithms.

/// Which pathfinding strategy the [`crate::solver::Solver`] should use.
///
/// A* takes a [`crate::solver::Heuristic`] argument; BFS and DFS are
/// uninformed. K-Shortest finds the top-`k` distinct paths in one shot
/// and does not support step-by-step visualisation.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    /// Informed search using `f = g + h`. The heuristic determines the
    /// notion of "distance to goal".
    #[default]
    AStar,
    /// Breadth-first search. Finds the shortest path on an unweighted grid
    /// but explores in concentric rings.
    BFS,
    /// Depth-first search. Fast and memory-light but does **not**
    /// guarantee a shortest path.
    DFS,
    /// Yen's algorithm — find the `k` shortest *loopless* paths between
    /// start and goal, sorted by length. Runs to completion in a single
    /// solver call; step-by-step visualisation is not supported.
    KShortest,
}

impl Algorithm {
    pub fn label(self) -> &'static str {
        match self {
            Algorithm::AStar => "A*",
            Algorithm::BFS => "BFS",
            Algorithm::DFS => "DFS",
            Algorithm::KShortest => "K-Shortest paths",
        }
    }

    /// All variants in display order.
    pub fn all() -> &'static [Algorithm] {
        &[
            Algorithm::AStar,
            Algorithm::BFS,
            Algorithm::DFS,
            Algorithm::KShortest,
        ]
    }

    /// Whether this algorithm uses a heuristic.
    pub fn uses_heuristic(self) -> bool {
        matches!(self, Algorithm::AStar)
    }

    /// Whether this algorithm can be advanced one node at a time. The
    /// `Step` button is disabled when this returns `false`.
    pub fn supports_stepping(self) -> bool {
        !matches!(self, Algorithm::KShortest)
    }

    /// Whether this algorithm produces multiple paths.
    pub fn is_multi_path(self) -> bool {
        matches!(self, Algorithm::KShortest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lists_every_variant_with_unique_labels() {
        let variants = Algorithm::all();
        assert_eq!(variants.len(), 4);
        let labels: Vec<&str> = variants.iter().map(|a| a.label()).collect();
        // Compare every distinct pair.
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate label: {a}");
            }
        }
    }

    #[test]
    fn capability_predicates_agree_with_intent() {
        // Only A* uses a heuristic.
        for &a in Algorithm::all() {
            let expected = matches!(a, Algorithm::AStar);
            assert_eq!(a.uses_heuristic(), expected, "{:?}", a);
        }
        // Only K-Shortest is multi-path.
        for &a in Algorithm::all() {
            let expected = matches!(a, Algorithm::KShortest);
            assert_eq!(a.is_multi_path(), expected, "{:?}", a);
        }
        // K-Shortest is the only one that doesn't step.
        for &a in Algorithm::all() {
            let expected = !matches!(a, Algorithm::KShortest);
            assert_eq!(a.supports_stepping(), expected, "{:?}", a);
        }
    }

    #[test]
    fn default_is_astar() {
        // App boots with A* selected; pin to avoid quietly changing
        // first-launch UX.
        assert_eq!(Algorithm::default(), Algorithm::AStar);
    }
}
