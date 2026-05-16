# Maze Solver

A cross-platform pathfinding visualizer built in Rust with [`egui`]/`eframe`.
Runs natively (Windows / macOS / Linux) **and** in the browser via WebAssembly
from the same codebase.

[![ci](https://github.com/loghi/pathlab/actions/workflows/ci.yml/badge.svg)](https://github.com/loghi/pathlab/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/loghi/pathlab/branch/main/graph/badge.svg)](https://codecov.io/gh/loghi/pathlab)
![rust](https://img.shields.io/badge/rust-1.95.0-blue)
![license](https://img.shields.io/badge/license-MIT-blue)

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
git clone https://github.com/Loghic/pathlab
cd pathlab
cargo run --release
```

The first `cargo` command in the project will pause briefly while
`rustup` installs the pinned toolchain (see [Toolchain](#toolchain)
below). After that, everything is fast.

### Web

The project is set up for [Trunk](https://trunkrs.dev):

```bash
cargo install trunk
trunk serve --release
```

The `wasm32-unknown-unknown` target is declared in `rust-toolchain.toml`,
so `rustup` installs it automatically — no separate `rustup target add`
needed.

Then open <http://127.0.0.1:8080>.

> If you build with `wasm-pack` or `wasm-bindgen-cli` directly, make sure the
> output file is called `maze_solver.js` so the `index.html` import resolves.

## Toolchain

`rust-toolchain.toml` pins the project to a specific Rust version
(currently `1.95.0`). The first `cargo` command in the project
auto-installs that version via `rustup`; nothing else for you to do.

The pin exists because clippy's lint set changes between releases.
Without it, contributors on different Rust versions would see different
warnings, and CI (always running the newest stable) would fail
mysteriously on code that was clean on someone's laptop. With the pin,
your laptop and CI run the exact same compiler and clippy.

To verify it's working: `rustup show` should print
`active toolchain: 1.95.0 (overridden by .../rust-toolchain.toml)`.

If a maintainer bumps the pin, your next `cargo build` quietly
downloads the new version. No action required from you.

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
├── scripts/             # pre-commit hook + installer
├── Cargo.toml           # crate manifest
├── rust-toolchain.toml  # pinned compiler version
├── index.html           # browser entry point for the wasm build
└── .github/workflows/   # CI: fmt, clippy, test, wasm-check
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

GitHub Actions runs four jobs on every push / PR (status visible at the
top of this README):

- `lint` — `cargo fmt --check` and `cargo clippy --all-targets -D warnings`.
- `test` — `cargo test` on Linux / macOS / Windows.
- `wasm-check` — `cargo check --target wasm32-unknown-unknown` so a
  desktop-only API call (e.g. `std::time::Instant`) is caught
  immediately, not at deploy time.
- `coverage` — runs the test suite under
  [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) and
  uploads an LCOV report to [Codecov](https://codecov.io/gh/loghi/a_star).
  Reading coverage on this project: the solver and undo modules sit
  near 100% because their tests are exhaustive; the `app/` and
  `platform/` modules sit much lower because they're driven by egui
  events and OS dialogs that unit tests can't easily exercise. Aim
  for high coverage on the pure-logic crates; don't chase it on the
  UI shell.

End-to-end GUI testing isn't included on purpose: the solver and undo
stack are the parts where bugs hide and they're 100% testable as pure
Rust. The GUI module is thin glue around them.

### Formatting and linting

The two `lint` checks CI runs are easy to fix locally — usually
automatically.

**`cargo fmt`** rewrites every `.rs` file in place using the standard
Rust formatting rules:

```bash
cargo fmt --all                     # rewrite in place
cargo fmt --all -- --check          # what CI does — exits non-zero on diffs
```

`rustfmt` and `clippy` are both listed as components in
`rust-toolchain.toml`, so `rustup` installed them with the toolchain
on your first `cargo` invocation. Nothing extra to do.

**`cargo clippy`** catches everything else — dead code, dubious idioms,
performance footguns, deprecated API usage. CI runs it with
`-D warnings`, meaning *any* warning fails the job.

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Most clippy warnings have an auto-fix:

```bash
# Apply all mechanically-safe fixes, then check what's left.
cargo clippy --fix --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

If `--fix` complains about a dirty working tree, add
`--allow-dirty --allow-staged`. Always `git diff` after a `--fix` run —
the auto-rewrites are usually right, but a few lints (especially around
loop-to-iterator rewrites) need a human eye. Lints that need judgement
get printed as warnings instead of being auto-fixed; clear those by
hand. The lint name in each warning links to documentation explaining
the rationale, so you can decide between fixing the code and adding
`#[allow(clippy::lint_name)]` with a short justification.

A typical pre-push routine:

```bash
cargo fmt --all
cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings   # confirm
cargo test
```

### Running coverage locally

The same tool CI uses is installable as a cargo subcommand:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Print a per-file summary to the terminal.
cargo llvm-cov --all-features

# Open an interactive HTML report in your browser.
cargo llvm-cov --all-features --open
```

The HTML report highlights covered lines green and uncovered ones red.
Useful when you've just added a feature and want to confirm the new
branches actually run.

### Local pre-commit hook

To run the same checks as CI before every commit (so you never push a
red build), install the included git hook. Pick whichever path matches
how you already work:

**Native git hook (zero dependencies):**

```bash
./scripts/install-hooks.sh
```

This symlinks `scripts/pre-commit` into `.git/hooks/`. Future updates
to the script are picked up automatically.

**Via the [`pre-commit`](https://pre-commit.com) framework:**

```bash
pip install pre-commit
pre-commit install
```

This reads `.pre-commit-config.yaml`. Same three checks, different
runner.

Either way, the hook runs `cargo fmt --check`, `cargo clippy -D warnings`,
and `cargo test`, in that order, and blocks the commit if anything
fails. Skip the hook for a single commit (e.g. a WIP push) with:

```bash
git commit --no-verify
```

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
