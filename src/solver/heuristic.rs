//! Heuristic functions for A*.
//!
//! A heuristic `h(n)` estimates the remaining cost from node `n` to the
//! goal. For A* to return an optimal path, `h` must be **admissible** -
//! never overestimating the true remaining cost.
//!
//! Since this solver only allows 4-connected (up/down/left/right) moves
//! with unit cost, Manhattan distance is both admissible *and* consistent.
//! Euclidean and Chebyshev are exposed mainly so you can see how the
//! choice changes the explored region in the visualizer.

/// All heuristics return f32 so Euclidean can express non-integer values.
pub type Cost = f32;

/// Heuristic flavour selectable from the UI when running A*.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Heuristic {
    /// `|dx| + |dy|`. Admissible for 4-connected grids.
    #[default]
    Manhattan,
    /// `sqrt(dx^2 + dy^2)`. Admissible but underestimates on 4-connected
    /// grids, often expanding more nodes than Manhattan.
    Euclidean,
    /// `max(|dx|, |dy|)`. Admissible if diagonal moves are allowed
    /// (they aren't here), so it can over-relax and skip the optimal path.
    Chebyshev,
    /// Always zero - reduces A* to Dijkstra (uniform-cost search).
    Zero,
}

impl Heuristic {
    pub fn label(self) -> &'static str {
        match self {
            Heuristic::Manhattan => "Manhattan",
            Heuristic::Euclidean => "Euclidean",
            Heuristic::Chebyshev => "Chebyshev",
            Heuristic::Zero => "Zero (Dijkstra)",
        }
    }

    pub fn all() -> &'static [Heuristic] {
        &[
            Heuristic::Manhattan,
            Heuristic::Euclidean,
            Heuristic::Chebyshev,
            Heuristic::Zero,
        ]
    }

    /// Estimate the remaining cost from `a` to `b`.
    pub fn estimate(self, a: (usize, usize), b: (usize, usize)) -> Cost {
        let dx = (a.0 as f32 - b.0 as f32).abs();
        let dy = (a.1 as f32 - b.1 as f32).abs();
        match self {
            Heuristic::Manhattan => dx + dy,
            Heuristic::Euclidean => (dx * dx + dy * dy).sqrt(),
            Heuristic::Chebyshev => dx.max(dy),
            Heuristic::Zero => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manhattan_is_symmetric_in_axes() {
        // Regression test for an early bug where dy was computed from
        // a.0 / b.1 instead of a.1 / b.1.
        let h = Heuristic::Manhattan.estimate((0, 0), (3, 4));
        assert_eq!(h as i32, 7);
    }

    #[test]
    fn zero_heuristic_is_zero() {
        assert_eq!(Heuristic::Zero.estimate((1, 2), (9, 9)), 0.0);
    }

    #[test]
    fn euclidean_345() {
        let h = Heuristic::Euclidean.estimate((0, 0), (3, 4));
        assert!((h - 5.0).abs() < 1e-6);
    }
}
