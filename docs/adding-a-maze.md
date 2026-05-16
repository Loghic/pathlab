# Adding a built-in maze

The four built-in mazes (Starting, Open, Wall Split, Boxed) all live in
`src/mazes/generators.rs` and are dispatched through a single
[`BuiltinMaze`] enum. Adding a new preset is a three-step change.

## 1 — Write the generator

A generator is just `fn(rows, cols) -> MazeGrid`. If your preset has a
fixed size, ignore the arguments.

```rust
// src/mazes/generators.rs
pub fn maze_zigzag(rows: usize, cols: usize) -> MazeGrid {
    let mut maze = vec![vec![Cell::Empty; cols]; rows];
    for (y, row) in maze.iter_mut().enumerate() {
        let len = (cols as f32 * 0.7) as usize;
        let start = if y % 2 == 0 { 0 } else { cols - len };
        for x in start..start + len {
            row[x] = Cell::Wall;
        }
    }
    maze
}
```

## 2 — Add a `BuiltinMaze` variant

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinMaze {
    Starting,
    Open,
    WallSplit,
    Boxed,
    Zigzag,           // <-- new
}
```

## 3 — Wire it into `label`, `generate`, and `list_builtin`

```rust
impl BuiltinMaze {
    pub fn label(self) -> &'static str {
        match self {
            // ...
            BuiltinMaze::Zigzag => "Zig-zag",
        }
    }

    pub fn generate(self, rows: usize, cols: usize) -> MazeGrid {
        match self {
            // ...
            BuiltinMaze::Zigzag => maze_zigzag(rows, cols),
        }
    }
}

pub fn list_builtin() -> &'static [BuiltinMaze] {
    &[
        BuiltinMaze::Starting,
        BuiltinMaze::Open,
        BuiltinMaze::WallSplit,
        BuiltinMaze::Boxed,
        BuiltinMaze::Zigzag,           // <-- new
    ]
}
```

That's it. The side panel iterates `list_builtin()` to populate the
dropdown, so the new preset appears the next time you `cargo run`.

## Optional: ship it as a PBM

If your maze is hand-drawn rather than parametric, you don't need any
code at all — just drop a `.pbm` into `assets/` and open it with
**File ▸ Open PBM**. See [`maze-format.md`](maze-format.md).

## Optional: add a unit test

```rust
#[test]
fn zigzag_has_expected_walls() {
    let m = maze_zigzag(4, 10);
    assert_eq!(m.len(), 4);
    assert_eq!(m[0].len(), 10);
}
```

Run with `cargo test`.
