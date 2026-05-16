# Maze file format

Mazes are stored as **P1 PBM** files — the plain-text "portable bitmap"
format from the Netpbm family. The full Netpbm spec is intentionally
broad; this section documents the subset Maze Solver reads and writes.

## Layout

```
P1            # magic
<W> <H>       # width then height, in cells
b b b ...     # W * H bits, '0' or '1', whitespace-separated
```

- `1` = wall (impassable).
- `0` = empty (walkable).
- The first row in the file is the top row of the maze.

## What the reader accepts

| Element | Behaviour |
| --- | --- |
| `# ...` comments | Stripped wherever they appear |
| Blank lines | Ignored |
| Whitespace between bits | Any amount (spaces, tabs, newlines) |
| Bits packed without whitespace | OK — `"1100"` parses as four bits |
| Trailing extra data | Rejected (size mismatch) |
| `P4` (binary PBM) | **Not** supported — convert to P1 first |

## What the writer produces

The output is deterministic and human-readable:

```
P1
<cols> <rows>
1 0 0 1 ...
0 1 1 0 ...
...
```

Each row of the maze is one line of the file, with bits separated by
single spaces. No comments are emitted (round-tripping with external
tools stays clean).

## Example

A 4×3 maze with a one-cell gap in a vertical wall:

```
P1
4 3
1 1 0 1
0 0 0 0
1 1 1 0
```

## Why PBM?

- **Trivial to author by hand or in any editor.** The first checked-in
  asset, `assets/sample.pbm`, is literally just typed out.
- **Visualisable** with Netpbm tools, GIMP, ImageMagick.
- **Round-trippable** — converting a maze to a PNG and back loses no
  information.
- **Cross-platform shared parser** — the same `maze_from_pbm_str`
  function powers both the native file dialog and the browser
  `FileReader`, so the two platforms can't drift apart.

## Conversion examples

To create a PBM from a PNG with ImageMagick:

```bash
magick maze.png -threshold 50% -compress none maze.pbm
```

To inspect a PBM emitted by this app:

```bash
cat maze.pbm
```
