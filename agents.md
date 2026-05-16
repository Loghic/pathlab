# AGENTS.md

Onboarding notes for AI coding assistants (Claude, Cursor, Copilot,
Aider, etc.) working on this repo. Humans, read `README.md` first;
this file assumes you've already skimmed it.

## What this project is

A cross-platform pathfinding visualizer written in Rust on top of
`egui` / `eframe`. The same `src/main.rs` compiles to a native
desktop binary **and** a WebAssembly module. The interesting code is
the pluggable solver (A* / BFS / DFS with selectable heuristics); the
GUI is thin glue over it.

## Repo map

```
pathlab/
├── Cargo.toml            # one workspace, lib + bin
├── index.html            # served as the wasm entry point
├── assets/               # sample .pbm mazes
├── docs/                 # extended docs (architecture, algorithms…)
├── .github/workflows/    # CI: fmt, clippy, test, wasm-check
└── src/
    ├── lib.rs            # re-exports
    ├── main.rs           # native + wasm entry points
    ├── mazes/            # Cell, MazeGrid, generators, PBM I/O
    ├── solver/           # Algorithm, Heuristic, Solver
    ├── platform/         # cross-platform Instant + file pickers
    └── app/              # egui front-end (state, panels, canvas, undo)
```

Every module has a top-level `//!` doc comment explaining its role.
Read those before changing anything — they're short.

## Hard rules

These are the rules that, if broken, will silently produce a broken
build *somewhere* (often on the platform you didn't test).

1. **Never call `std::time::Instant` directly.** It panics on
   `wasm32-unknown-unknown`. Always use `crate::platform::time::{Instant, Duration}`
   which re-exports `web-time` and works on both targets.
2. **Never reach for `std::fs` or `rfd` outside `src/platform/`.** File I/O
   crosses the native/web boundary; that's the only module allowed
   to know which platform it's on. Web file pickers are async and
   route results through `FileInbox` (an `Arc<Mutex<Option<...>>>`).
3. **The solver does not know about the GUI.** `src/solver/` only
   depends on `src/mazes/`. Don't import `eframe` or `egui` there.
4. **Wall/Empty are the only cell types.** PBM is a two-bit format
   and adding a third `Cell` variant breaks round-tripping. See the
   "design notes" section for an explanation of why "endpoint in a
   wall" is *not* a reason to add a new variant.
5. **Don't call `cargo` commands the user didn't ask for.** Builds
   can be slow and pollute their terminal. Suggest commands; let them
   run them.
6. **The Rust toolchain is pinned in `rust-toolchain.toml`.** Don't
   bump the version casually — bumping it can introduce new clippy
   lints that break CI. If you genuinely need a newer compiler
   feature, bump the pin *and* fix any new lints in the same commit.
   Don't write conditional code that targets multiple toolchains;
   one project, one toolchain.

## Module conventions

### `src/mazes/`

- `Cell` is `Copy`. Don't add owned data to it.
- New built-in mazes go through the `BuiltinMaze` enum so the UI
  dropdown picks them up automatically. The guide is in
  `docs/adding-a-maze.md`.
- `pbm.rs` has the only parser; both desktop and web go through
  `maze_from_pbm_str`. The native helper `maze_from_pbm_path` is a
  thin wrapper around `read_to_string` + the same parser. Keep it
  that way.

### `src/solver/`

- `Solver::step` advances exactly one node, regardless of algorithm.
  There used to be a `step_bfs_layer` that advanced a whole ring; it
  was removed because it made step-mode behave differently from
  solve-mode. Don't reintroduce it.
- **`Algorithm::KShortest` is the exception** — it runs Yen's algorithm
  to completion in a single `step()` call. The Step button is
  disabled for it through `Algorithm::supports_stepping()`. If you add
  another one-shot algorithm, return `false` there too.
- **Multi-path algorithms** populate `Solver::paths()`. Single-path
  algorithms mirror their result there as well (a length-1 vec), so
  the canvas can iterate `paths()` uniformly. `path()` is kept around
  as a convenience that returns the shortest one.
- For A*, the open set uses a `BinaryHeap<OpenNode>` keyed on
  `f = g + h`. Stale entries (when a node is re-prioritised) are
  lazily filtered on pop using the `open_set` HashSet as the source
  of truth for membership. **Don't** try to remove from the heap
  directly — there's no decrease-key operation.
- Heuristics return `f32` (so Euclidean works). Manhattan must
  compute `|dx| + |dy|` with `dx = a.0 - b.0`, `dy = a.1 - b.1`. A
  regression test (`manhattan_is_symmetric_in_axes`) pins this
  because v0.1 of the codebase mixed the axes.
- `k_paths.rs` is the only place that knows about Yen's algorithm.
  If you need to call it from anywhere else, prefer going through
  `Solver` (with `Algorithm::KShortest` and `with_k`) rather than
  hitting the function directly.

### `src/app/`

- All app state lives in `MazeApp` (in `state.rs`). The
  `eframe::App::update` impl only orchestrates: it dispatches the
  top bar, side panel, and canvas, then calls `tick_solver`. Logic
  belongs on `MazeApp`, not in the panel functions.
- **Undo is per user action**, not per cell. The contract is:
  - Drag strokes call `app.snapshot_for_undo()` *once* on
    `response.drag_started()`.
  - Bulk mutators (`set_maze`, `invert_maze`, `fill_with`, resize ops)
    call `snapshot_for_undo()` before mutating.
  - Per-cell wall toggles inside a drag do **not** snapshot.
  - The cap is `MAX_DEPTH = 100`; identical consecutive states are
    deduped. See `src/app/undo.rs`.
- Method `MazeApp::undo_last` is named that way to avoid shadowing
  the `undo: UndoStack` field — `app.undo` is the field,
  `app.undo_last()` is the method. Don't rename either back.
- Visual constants (`PATH_COLOR`, `START_FILL`, etc.) live at the top
  of `canvas.rs`. If you want to change the look, edit them there —
  don't sprinkle `Color32::from_rgb(...)` through the drawing code.

### `src/platform/`

- `time.rs` re-exports `web-time::{Instant, Duration}`. That's the
  whole file. If you need any time-related API, add it here.
- `fileio.rs` has two `#[cfg]`-gated implementations sharing a
  public API. The shared `FileInbox` lets the async web picker
  deliver into the same code path the native picker uses.

## Build / test / run

```bash
# Native
cargo run --release

# Tests
cargo test

# Lint as CI does it
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Web (requires `trunk` and the wasm32 target)
rustup target add wasm32-unknown-unknown
trunk serve --release
```

CI runs all of the above plus `cargo check --target wasm32-unknown-unknown`
on every push, and a `coverage` job that uploads to Codecov. If you
change `Cargo.toml` or `src/platform/`, run the wasm check locally
before pushing — it's the cheapest way to catch "compiled fine on my
laptop, broken in the browser" bugs.

If a pre-commit hook is installed (`scripts/install-hooks.sh` or
`pre-commit install`), `cargo fmt --check`, `cargo clippy -D warnings`,
and `cargo test` run automatically before each commit. Trust the hook:
if it rejects a commit, fix the underlying issue rather than bypassing
it. The only legitimate reason to use `git commit --no-verify` is a
genuine WIP push to a personal branch.

## Toolchain pin

`rust-toolchain.toml` at the repo root pins the project to a specific
Rust version (currently `1.95.0`). `rustup` reads this file
automatically: any `cargo` invocation inside the project tree uses
exactly the pinned compiler, regardless of your global default.

This exists because **clippy's lint set changes between Rust releases**.
Without the pin, contributors on different stable versions would see
different warnings, and CI (which always ran the newest stable) would
fail mysteriously on code that was clean on someone's laptop.

When operating in this repo:

- Use the pinned toolchain. `rustup show` should display
  "active toolchain: 1.95.0 (overridden by rust-toolchain.toml)".
- Don't add `#[allow(clippy::lint_name)]` to silence a lint that's
  only triggered because you ran a newer clippy locally. Update the
  pin instead — or, better, leave the version alone and write code
  that passes the pinned clippy.
- To bump the pin (rare), edit `channel = "1.95.0"` to the new
  version, run `cargo clippy --fix --allow-dirty --allow-staged
  --all-targets --all-features`, then `cargo clippy ... -D warnings`
  to catch leftovers. Commit the pin bump and the lint fixes
  together with a clear message.

## Coverage and CI status

The README has four badges at the top — `ci`, `codecov`, the pinned
Rust version, and the license. The first two are live; the second two
are static strings that need a manual bump if their underlying values
change (currently `1.95.0` and `MIT`).

Coverage is generated by `cargo-llvm-cov` in the `coverage` CI job and
uploaded to Codecov. **Read coverage as a per-module signal, not a
single global number.** Expected shape of the report (as of the last
test pass):

- `src/solver/` and `src/app/undo.rs`: ~90-100%. Exhaustively
  unit-tested. Any drop here is a real regression worth investigating.
- `src/mazes/`: ~85-95%. Generators and PBM I/O are easy to test.
- `src/app/state.rs`: ~30-40%. Only the pure mutators (clamp, resize,
  fill, invert, undo, solver factory) are tested. `tick_solver` and
  `update` need an egui Context to exercise.
- `src/app/canvas.rs`, `side_panel.rs`, `top_bar.rs`: ~0-5%. Pure
  rendering code driven by mouse events and `Painter`.
- `src/platform/fileio.rs`: ~0%. Talks to OS pickers or the DOM.
- `src/main.rs`: 0%. Entry point.

Overall coverage will sit around 55-60%. **That's the correct number.**
If you see it lower, something in `solver/` or `mazes/` likely lost
coverage. If you see it higher, someone may have written a bad test
(see below).

**Things NOT to do to "improve" coverage:**

- Don't write `#[test] fn click_solve_button()` style tests that
  instantiate `MazeApp` just to walk its state machine. These end up
  testing the test setup, not the behaviour.
- Don't add tests that call `tick_solver` in a loop with mocked time
  to claim coverage of the timing logic. Real bugs in that code
  surface as user-visible UI glitches, not pure-logic failures, and
  the test won't catch them.
- Don't add `#[cfg(test)]` shims that bypass the real code path just
  to mark a line as "covered".
- Don't drop coverage thresholds when CI catches a regression. Find
  why coverage fell and fix it, or explicitly justify the drop in
  the PR description.

**The legitimate reasons to add a test:**

- A bug was reported and a test reproduces it (regression test).
- A new public function/algorithm was added.
- A previously implicit invariant is being made explicit
  (e.g. "Manhattan must be symmetric in axes" pinned the v0.1 bug).
- A boundary condition isn't currently exercised (empty maze,
  single-cell maze, start == goal).

When in doubt: would this test fail if the function were silently
broken? If yes, write it. If it would only fail if someone deleted the
function entirely, skip it.

**Example of a good test** (from `solver/core.rs`):

```rust
#[test]
fn start_equals_goal_returns_single_cell_path() {
    // User can drag the end onto the start. Expected behaviour:
    // immediately Found, with a path of length 1.
    let maze = maze_open(3, 3);
    for &algo in &[Algorithm::AStar, Algorithm::BFS, Algorithm::DFS] {
        let mut s = Solver::new(algo, Heuristic::Manhattan, (1, 1), (1, 1));
        solve_to_completion(&mut s, &maze);
        assert_eq!(s.status(), SolverStatus::Found, "{algo:?}");
        assert_eq!(s.path(), &[(1, 1)], "{algo:?}");
    }
}
```

Notice the shape: it pins a real user-reachable scenario, the comment
explains *why* that scenario matters (the UI lets users do it), and it
loops over multiple algorithms so a regression in any single one is
caught. That's what an earned test looks like.

**Clippy gotcha when writing tests over a grid.** Clippy's
`needless_range_loop` doesn't like `for y in 0..rows { maze[y][...] }`
in test bodies, even when the index is genuinely used. Use these
idioms instead:

| You want                                  | Write                                       |
| ----------------------------------------- | ------------------------------------------- |
| Iterate rows with index                   | `for (y, row) in maze.iter().enumerate()`   |
| Iterate rows without index                | `for row in &maze`                          |
| Compare every distinct pair               | `for (i, a) in xs.iter().enumerate() { for b in xs.iter().skip(i + 1) {...} }` |
| Just count something N times              | `for _ in 0..N` *(clippy permits this)*    |
| Need the top *and* bottom row by index    | `maze.first()` and `maze.last()` directly  |

## egui version pin

The crate uses `eframe = "0.33"` / `egui_extras = "0.33"`. Two API
points moved recently — be careful:

- **Menu bar.** Use `egui::MenuBar::new().ui(ui, |ui| { ... })`.
  The old `egui::menu::bar(ui, ...)` is deprecated.
- **ComboBox id.** Use `ComboBox::from_id_salt("…")`. The old
  `from_id_source` is deprecated.
- **`ui.close()`** is the new way to dismiss a menu after a click.
  The old `ui.close_menu()` is deprecated.
- **`Painter::rect_stroke`** takes `(rect, corner_radius, stroke, stroke_kind)`
  — four arguments. Pass `StrokeKind::Outside` for cell grid lines,
  `StrokeKind::Inside` for borders that shouldn't extend the rect.

If you see deprecation warnings, update to these forms rather than
silencing them.

## Design notes — things people ask for that we're declining

These are not blanket "no"s, but the bar is high. If you're tempted
to add any of these, **stop and ask first.**

- **A third `Cell` variant for "endpoint marker"** — declined because
  PBM only has two bit values, so a third variant breaks round-trip.
  The existing behaviour (`NoPath` when the endpoint is unreachable)
  is the correct answer. The right place to help users is a UI hint,
  not a new cell type.
- **Diagonal moves** — declined for now because the existing
  heuristics are tuned for 4-connected grids (Manhattan is admissible
  there; Chebyshev would become the right default for 8-connected).
  Adding it means revisiting every heuristic *and* the unit tests
  that compare BFS/A* path lengths.
- **Persisting preferences across sessions** — not in scope. The web
  build has no persistent storage hooked up. If you need this, write
  a design note in `docs/` first.
- **`Cargo.lock` in `.gitignore`** — this is a binary crate.
  `Cargo.lock` is checked in on purpose so CI and developer builds
  reproduce.

## When something is unclear

Prefer reading over guessing:

1. The relevant `//!` module comment.
2. The doc in `docs/` matching the area you're editing
   (`architecture.md`, `algorithms.md`, `adding-a-maze.md`,
   `adding-an-algorithm.md`, `maze-format.md`, `web-build.md`).
3. The unit tests — they document expected behaviour better than
   most prose.
4. The git log for the file you're touching.

If none of those answer the question, ask the human in the loop.
Don't invent project conventions on the fly — write the question
into your PR description instead.

## Style

- Run `cargo fmt` before committing. CI fails `--check` otherwise.
- Keep modules small. If a file goes past ~400 lines, look for a
  split. `src/app/` is already split into four focused files for
  this reason.
- Comments explain *why*, not *what*. The code says what.
- Doc comments (`///`) on every public item. Top-level (`//!`) on
  every module file.
- Tests next to the code they cover, inside `#[cfg(test)] mod tests`.

## What done looks like

A change is done when:

1. `cargo fmt --all -- --check` passes.
2. `cargo clippy --all-targets --all-features -- -D warnings` passes.
3. `cargo test` passes.
4. `cargo check --target wasm32-unknown-unknown` passes.
5. New pure-logic code is covered by a test. UI/platform code is
   exempt; see the "Coverage and CI status" section.
6. Anything user-visible is reflected in `README.md`.
7. Anything architecturally non-obvious is reflected in `docs/`.
8. New public items have doc comments.

Do all eight. CI runs the first four (and reports coverage as a
signal, not a gate); the rest are on you.
