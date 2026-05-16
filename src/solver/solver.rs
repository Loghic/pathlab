//! Step-by-step pathfinder driving A*, BFS, and DFS over a [`MazeGrid`].

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use super::algorithm::Algorithm;
use super::heuristic::{Cost, Heuristic};
use super::k_paths::k_shortest_paths;
use crate::mazes::{Cell, MazeGrid};

/// `(x, y)` grid coordinate.
pub type Coord = (usize, usize);

/// What the [`Solver`] is currently doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SolverStatus {
    Running,
    Found,
    NoPath,
}

/// Entry in the A* min-heap.
///
/// `BinaryHeap` is a max-heap, so we invert the comparison on `f` to get
/// min-heap behaviour.
#[derive(Clone, Copy)]
struct OpenNode {
    f: Cost,
    coord: Coord,
}

impl PartialEq for OpenNode {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}
impl Eq for OpenNode {}

impl Ord for OpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Smaller f-score = higher priority.
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for OpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Drives one pathfinding run.
///
/// Construct with [`Solver::new`] and call [`Solver::step`] repeatedly from
/// the UI loop. Inspect [`Solver::open_cells`], [`Solver::closed_cells`],
/// and [`Solver::path`] to render the current state.
pub struct Solver {
    algorithm: Algorithm,
    heuristic: Heuristic,
    start: Coord,
    goal: Coord,
    status: SolverStatus,

    // Frontier - the right structure for each algorithm.
    open_stack: Vec<Coord>,          // DFS
    open_queue: VecDeque<Coord>,     // BFS
    open_heap: BinaryHeap<OpenNode>, // A*

    // Membership index for the frontier - O(1) `contains` checks.
    open_set: HashSet<Coord>,

    // Already-expanded nodes.
    closed: HashSet<Coord>,

    // Reconstruction info.
    came_from: HashMap<Coord, Coord>,
    g_score: HashMap<Coord, Cost>,

    // Result. For single-path algorithms `path` is the result and
    // `paths` mirrors it as a length-1 vec. For K-Shortest, `paths`
    // is the full list (longest to shortest by length) and `path` is
    // the shortest one (paths[0]) for backwards compatibility.
    path: Vec<Coord>,
    paths: Vec<Vec<Coord>>,

    /// Only consulted when `algorithm == KShortest`. How many distinct
    /// paths to look for.
    k: usize,
}

impl Solver {
    /// Build a fresh solver. The `start` cell is immediately added to the
    /// open set; no work has been done yet.
    pub fn new(algorithm: Algorithm, heuristic: Heuristic, start: Coord, goal: Coord) -> Self {
        let mut s = Self {
            algorithm,
            heuristic,
            start,
            goal,
            status: SolverStatus::Running,
            open_stack: Vec::new(),
            open_queue: VecDeque::new(),
            open_heap: BinaryHeap::new(),
            open_set: HashSet::new(),
            closed: HashSet::new(),
            came_from: HashMap::new(),
            g_score: HashMap::new(),
            path: Vec::new(),
            paths: Vec::new(),
            k: 1,
        };
        s.push_open(start, 0.0);
        s.g_score.insert(start, 0.0);
        s
    }

    /// Builder-style: set how many paths to look for. Only meaningful
    /// for [`Algorithm::KShortest`]; other algorithms always return one
    /// path and ignore this.
    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k.max(1);
        self
    }

    // ------------------------------------------------------------------
    // Public accessors
    // ------------------------------------------------------------------

    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
    pub fn status(&self) -> SolverStatus {
        self.status
    }
    pub fn finished(&self) -> bool {
        !matches!(self.status, SolverStatus::Running)
    }
    pub fn path(&self) -> &[Coord] {
        &self.path
    }
    /// All paths found by this solver. For single-path algorithms this
    /// is a length-1 slice mirroring [`Self::path`]. For K-Shortest it
    /// holds up to `k` distinct paths, sorted by length.
    pub fn paths(&self) -> &[Vec<Coord>] {
        &self.paths
    }

    /// All cells currently waiting to be expanded (the "open set" / frontier).
    pub fn open_cells(&self) -> &HashSet<Coord> {
        &self.open_set
    }

    /// All cells that have already been expanded.
    pub fn closed_cells(&self) -> &HashSet<Coord> {
        &self.closed
    }

    // ------------------------------------------------------------------
    // Core step
    // ------------------------------------------------------------------

    /// Expand exactly one node. Cheap if the search has already finished.
    pub fn step(&mut self, maze: &MazeGrid) {
        if self.finished() {
            return;
        }

        // K-Shortest doesn't fit the incremental model — it runs Yen's
        // algorithm to completion in one shot. This means a single
        // `step()` call solves the whole problem; subsequent calls are
        // no-ops because `finished()` is true.
        if self.algorithm == Algorithm::KShortest {
            self.run_k_shortest(maze);
            return;
        }

        let Some(current) = self.pop_open() else {
            self.status = SolverStatus::NoPath;
            return;
        };

        // It is possible for A* to leave stale heap entries from an
        // earlier, more expensive path to the same node. Skip them.
        if self.closed.contains(&current) {
            return;
        }

        if current == self.goal {
            self.reconstruct_path(current);
            self.status = SolverStatus::Found;
            return;
        }

        self.closed.insert(current);

        for neighbor in neighbors_4(current, maze) {
            if self.closed.contains(&neighbor) {
                continue;
            }
            // Wall check - bounds already validated by neighbors_4.
            if maze[neighbor.1][neighbor.0] == Cell::Wall {
                continue;
            }

            match self.algorithm {
                Algorithm::DFS | Algorithm::BFS => {
                    if !self.open_set.contains(&neighbor) {
                        self.came_from.insert(neighbor, current);
                        self.push_open(neighbor, 0.0); // priority unused
                    }
                }
                Algorithm::AStar => {
                    let tentative_g =
                        self.g_score.get(&current).copied().unwrap_or(Cost::MAX) + 1.0;

                    let known_g = self
                        .g_score
                        .get(&neighbor)
                        .copied()
                        .unwrap_or(Cost::INFINITY);

                    if tentative_g < known_g {
                        self.came_from.insert(neighbor, current);
                        self.g_score.insert(neighbor, tentative_g);
                        let f = tentative_g + self.heuristic.estimate(neighbor, self.goal);
                        self.push_open(neighbor, f);
                    }
                }
                Algorithm::KShortest => {
                    // Handled by the early return above; unreachable in
                    // practice. The arm exists so the match is exhaustive.
                    debug_assert!(false, "k-shortest should not reach step's neighbour loop");
                }
            }
        }
    }

    /// One-shot driver for [`Algorithm::KShortest`].
    fn run_k_shortest(&mut self, maze: &MazeGrid) {
        self.paths = k_shortest_paths(maze, self.start, self.goal, self.k);
        if self.paths.is_empty() {
            self.status = SolverStatus::NoPath;
            self.path.clear();
        } else {
            // path[0] is the shortest; keep it as `self.path` for code
            // that only cares about "the" best route.
            self.path = self.paths[0].clone();
            self.status = SolverStatus::Found;
        }
    }

    // ------------------------------------------------------------------
    // Frontier helpers
    // ------------------------------------------------------------------

    fn push_open(&mut self, c: Coord, f: Cost) {
        // For A* we always push (cheap path may discover later); the open
        // set tracks membership so duplicate heap entries get filtered on
        // pop. For BFS/DFS we strictly avoid pushing duplicates.
        match self.algorithm {
            Algorithm::DFS => {
                if self.open_set.insert(c) {
                    self.open_stack.push(c);
                }
            }
            Algorithm::BFS => {
                if self.open_set.insert(c) {
                    self.open_queue.push_back(c);
                }
            }
            Algorithm::AStar => {
                self.open_set.insert(c);
                self.open_heap.push(OpenNode { f, coord: c });
            }
            Algorithm::KShortest => {
                // KShortest doesn't use the incremental frontier - the
                // whole thing runs in one shot via run_k_shortest. The
                // initial push from Solver::new is silently dropped.
            }
        }
    }

    fn pop_open(&mut self) -> Option<Coord> {
        match self.algorithm {
            Algorithm::DFS => {
                let c = self.open_stack.pop()?;
                self.open_set.remove(&c);
                Some(c)
            }
            Algorithm::BFS => {
                let c = self.open_queue.pop_front()?;
                self.open_set.remove(&c);
                Some(c)
            }
            Algorithm::AStar => {
                // Discard stale entries (those whose coord was already
                // closed or re-prioritised under a smaller f).
                loop {
                    let node = self.open_heap.pop()?;
                    // We treat the heap as authoritative for ordering but
                    // open_set for membership. Keep popping until we find
                    // a node still in the open set.
                    if self.open_set.remove(&node.coord) {
                        return Some(node.coord);
                    }
                }
            }
            Algorithm::KShortest => None,
        }
    }

    fn reconstruct_path(&mut self, mut current: Coord) {
        self.path.clear();
        self.path.push(current);
        while let Some(&prev) = self.came_from.get(&current) {
            current = prev;
            self.path.push(current);
            if current == self.start {
                break;
            }
        }
        self.path.reverse();
        // Mirror the single path into `paths` so the canvas can iterate
        // uniformly over all algorithms.
        self.paths = vec![self.path.clone()];
    }
}

/// 4-connected neighbourhood with bounds checking.
fn neighbors_4(c: Coord, maze: &MazeGrid) -> impl Iterator<Item = Coord> + '_ {
    let (x, y) = c;
    let height = maze.len();
    let width = if height > 0 { maze[0].len() } else { 0 };
    const DIRS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    DIRS.into_iter().filter_map(move |(dx, dy)| {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
            None
        } else {
            Some((nx as usize, ny as usize))
        }
    })
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mazes::maze_open;

    fn solve_to_completion(s: &mut Solver, maze: &MazeGrid) {
        for _ in 0..10_000 {
            s.step(maze);
            if s.finished() {
                return;
            }
        }
        panic!("solver did not finish");
    }

    #[test]
    fn astar_finds_shortest_on_open_grid() {
        let maze = maze_open(5, 5);
        let mut s = Solver::new(Algorithm::AStar, Heuristic::Manhattan, (0, 0), (4, 4));
        solve_to_completion(&mut s, &maze);
        assert_eq!(s.status(), SolverStatus::Found);
        // 4 right + 4 down + start = 9 cells.
        assert_eq!(s.path().len(), 9);
    }

    #[test]
    fn bfs_matches_astar_length() {
        let maze = maze_open(6, 6);
        let mut a = Solver::new(Algorithm::AStar, Heuristic::Manhattan, (0, 0), (5, 5));
        let mut b = Solver::new(Algorithm::BFS, Heuristic::Manhattan, (0, 0), (5, 5));
        solve_to_completion(&mut a, &maze);
        solve_to_completion(&mut b, &maze);
        assert_eq!(a.path().len(), b.path().len());
    }

    #[test]
    fn no_path_through_walls() {
        // Vertical wall splitting the grid.
        let mut maze = maze_open(3, 3);
        for row in maze.iter_mut().take(3) {
            row[1] = Cell::Wall;
        }
        let mut s = Solver::new(Algorithm::BFS, Heuristic::Manhattan, (0, 0), (2, 2));
        solve_to_completion(&mut s, &maze);
        assert_eq!(s.status(), SolverStatus::NoPath);
    }

    #[test]
    fn dfs_reaches_goal_even_if_not_shortest() {
        let maze = maze_open(4, 4);
        let mut s = Solver::new(Algorithm::DFS, Heuristic::Manhattan, (0, 0), (3, 3));
        solve_to_completion(&mut s, &maze);
        assert_eq!(s.status(), SolverStatus::Found);
        assert_eq!(*s.path().first().unwrap(), (0, 0));
        assert_eq!(*s.path().last().unwrap(), (3, 3));
    }

    #[test]
    fn k_shortest_returns_multiple_paths() {
        let maze = maze_open(3, 3);
        let mut s =
            Solver::new(Algorithm::KShortest, Heuristic::Manhattan, (0, 0), (2, 2)).with_k(4);
        // One step is enough; KShortest runs to completion in one call.
        s.step(&maze);
        assert!(s.finished());
        assert_eq!(s.status(), SolverStatus::Found);
        assert!(s.paths().len() >= 2);
        // Shortest path is mirrored into `path()`.
        assert_eq!(s.path(), s.paths()[0]);
    }

    #[test]
    fn single_path_algorithms_populate_paths() {
        // Whoever wires the canvas can iterate over `paths()` uniformly.
        let maze = maze_open(4, 4);
        let mut s = Solver::new(Algorithm::BFS, Heuristic::Manhattan, (0, 0), (3, 3));
        solve_to_completion(&mut s, &maze);
        assert_eq!(s.paths().len(), 1);
        assert_eq!(s.paths()[0], s.path());
    }
}
