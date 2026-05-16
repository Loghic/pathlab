# Maze Solver

A cross-platform pathfinding visualizer built in Rust with [`egui`]/`eframe`.
Runs natively (Windows / macOS / Linux) **and** in the browser via WebAssembly
from the same codebase.

![status](https://img.shields.io/badge/status-pre--release-orange)
![rust](https://img.shields.io/badge/rust-1.74%2B-blue)

## Features

- **Four algorithms** — A*, BFS, DFS, and K-Shortest paths (Yen's
  algorithm) — each driven step-by-step (except K-Shortest, which is
  one-shot) so you can watch the frontier expand in real time.
- **Selectable heuristics for A***: Manhattan, Euclidean, Chebyshev, and Zero
  (which collapses A* into Dijkstra).
- **K-Shortest paths visualisation** — pick a `k` between 1 and 32 and
  the canvas renders every distinct path in its own colour, drawn
  longest-to-shortest so the optimal route stays on top.
- **Built-in maze presets** plus an in-app **wall editor** (draw / erase /
  fill / invert).
- **Undo** (`Ctrl+Z` / `Cmd+Z`) with stroke-level granularity — one
  press rolls back an entire drag, a Fill, a preset switch, or a PBM load.
- **Resize the grid** by adding or removing rows/columns of empties or walls.
- **PBM (P1) file I/O** — save your maze and reopen it later. Works on both
  native and web.
- **Loop mode** to continuously replay the solve, with a configurable
  dwell time on the finished path.
- **Pick start / end** by clicking on the canvas — endpoints are drawn as
  labelled, bordered tiles ("S" / "E") so they stay visible against any
  background or overlay.
- **Auto-fit** rendering — the maze always fills the available space.

## Quick start

### Native

```bash
git clone <this repo>
cd maze-solver
cargo run --release
```

### Web

The project is set up for [Trunk](https://trunkrs.dev):

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk serve --release
```

Then open <http://127.0.0.1:8080>.

> If you build with `wasm-pack` or `wasm-bindgen-cli` directly, make sure the
> output file is called `maze_solver.js` so the `index.html` import resolves.

## Project layout

```
maze-solver/
├── src/
│   ├── main.rs          # native + wasm entry points
│   ├── lib.rs           # re-exports for binary & tests
│   ├── mazes/           # Cell, MazeGrid, generators, PBM I/O
│   ├── solver/          # Algorithm, Heuristic, Solver
│   ├── app/             # egui front-end (state + panels + canvas)
│   └── platform/        # cross-platform Instant + file pickers
├── assets/              # sample mazes (.pbm)
├── docs/                # extended documentation (see below)
└── index.html           # browser entry point for the wasm build
```

## Documentation

| Doc | Topic |
| --- | --- |
| [`docs/architecture.md`](docs/architecture.md) | Module diagram and data flow |
| [`docs/algorithms.md`](docs/algorithms.md) | A*, BFS, DFS math & heuristics |
| [`docs/maze-format.md`](docs/maze-format.md) | The PBM dialect used by this app |
| [`docs/adding-a-maze.md`](docs/adding-a-maze.md) | How to add a new built-in maze |
| [`docs/adding-an-algorithm.md`](docs/adding-an-algorithm.md) | Plug in your own solver |
| [`docs/web-build.md`](docs/web-build.md) | wasm-specific notes & the timer fix |

## Testing & CI

```bash
cargo test
```

Unit tests cover the solver (A* and BFS agree on shortest-path length on
open grids; "no path" is reported when one is impossible; the Manhattan
heuristic is symmetric — a regression test for an axis-mix bug that
lived in v0.1) and the undo stack (push deduping, bounded depth, FIFO
eviction).

GitHub Actions runs three jobs on every push / PR:

- `lint` — `cargo fmt --check` and `cargo clippy --all-targets -D warnings`.
- `test` — `cargo test` on Linux / macOS / Windows.
- `wasm-check` — `cargo check --target wasm32-unknown-unknown` so a
  desktop-only API call (e.g. `std::time::Instant`) is caught
  immediately, not at deploy time.

End-to-end GUI testing isn't included on purpose: the solver and undo
stack are the parts where bugs hide and they're 100% testable as pure
Rust. The GUI module is thin glue around them.

## Controls

| Action | How |
| --- | --- |
| Draw / erase walls | Toggle **Draw wall** or **Erase wall**, drag on the canvas |
| Pick start / end | **Pick** button, then click a cell |
| Solve once | **Solve** |
| Single step | **Step** |
| Auto-replay | **Loop** checkbox |
| Undo | **Ctrl+Z** / **Cmd+Z**, or the **Undo** button |
| Save / load | File ▸ Save / Open (PBM only) |

## License

MIT.
