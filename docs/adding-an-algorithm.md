# Adding a new algorithm

The [`Solver`] is built so the algorithm choice changes only **which
frontier container is consulted** in `push_open` / `pop_open` and which
priority is written. Adding a new algorithm is therefore mostly a
matter of extending two `match` expressions plus the enum.

## Example: bidirectional BFS

### 1 — Extend the enum

```rust
// src/solver/algorithm.rs
pub enum Algorithm {
    AStar,
    BFS,
    DFS,
    KShortest,
    BiBFS,            // <-- new
}
```

Wire it into `label`, `all`, `uses_heuristic`, and the two capability
predicates `supports_stepping` (set false if it's a one-shot algorithm
like K-Shortest) and `is_multi_path` (set true if it returns multiple
paths and you want the canvas to colour them distinctly).

### 2 — Add state to the solver

If your algorithm needs new bookkeeping (a second frontier, a meet-in-
the-middle node, etc.), add it as a private field on `Solver`. Keep
existing fields untouched to avoid breaking the other algorithms.

```rust
// src/solver/solver.rs
pub struct Solver {
    // ...
    open_queue_back: VecDeque<Coord>,
    closed_back: HashSet<Coord>,
    came_from_back: HashMap<Coord, Coord>,
}
```

### 3 — Pick the right container in `push_open` / `pop_open`

Both helpers already match on `self.algorithm`. Add an arm:

```rust
Algorithm::BiBFS => {
    // forward layer first, then backward layer
    // ...
}
```

### 4 — Drive it in `step`

`step` is the only place the algorithm actually runs. Add an arm to
the neighbour-handling block (or to a higher-level branch if the whole
step looks different, as bidirectional search does).

### 5 — Test it

The existing tests already prove that BFS and A* return paths of the
same length on open grids. A reasonable addition for BiBFS is to assert
that *its* path length matches BFS's on a few canned mazes.

```rust
#[test]
fn bibfs_matches_bfs_length() {
    let maze = maze_open(8, 8);
    let mut bfs = Solver::new(Algorithm::BFS, Heuristic::Manhattan, (0,0), (7,7));
    let mut bib = Solver::new(Algorithm::BiBFS, Heuristic::Manhattan, (0,0), (7,7));
    solve_to_completion(&mut bfs, &maze);
    solve_to_completion(&mut bib, &maze);
    assert_eq!(bfs.path().len(), bib.path().len());
}
```

## Adding a heuristic instead

If you just want a new A* heuristic — for instance, Octile — only
`src/solver/heuristic.rs` has to change: add a variant, an arm in
`estimate`, an arm in `label`, and an entry in `all`. The UI picks it
up automatically.
