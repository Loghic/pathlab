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
maze-solver/
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
on every push. If you change `Cargo.toml` or `src/platform/`, run
the wasm check locally before pushing — it's the cheapest way to
catch "compiled fine on my laptop, broken in the browser" bugs.

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
5. Anything user-visible is reflected in `README.md`.
6. Anything architecturally non-obvious is reflected in `docs/`.
7. New public items have doc comments.

Do all six. The CI runs the first four; the last three are on you.
