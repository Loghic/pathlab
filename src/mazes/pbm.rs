//! Minimal P1 (ASCII) PBM reader/writer.
//!
//! The format used by this project encodes walls as `1` and walkable cells
//! as `0`. The on-disk format is exactly what the reference Netpbm tools
//! produce, so files can be authored externally.

use super::cell::{Cell, MazeGrid};

/// Parse a P1 PBM from an in-memory string.
///
/// Works on every platform - the desktop and web file pickers both end up
/// calling this. Returns a human-readable error message on malformed input.
pub fn maze_from_pbm_str(contents: &str) -> Result<MazeGrid, String> {
    // Strip blank and comment lines up-front. PBM allows `#` comments
    // anywhere whitespace would be valid.
    let mut tokens = contents
        .lines()
        .map(|l| {
            // Cut a trailing comment.
            match l.find('#') {
                Some(i) => &l[..i],
                None => l,
            }
        })
        .flat_map(str::split_whitespace);

    let header = tokens.next().ok_or("Empty PBM")?;
    if header != "P1" {
        return Err(format!("Unsupported PBM magic: {header} (expected P1)"));
    }

    let width: usize = tokens
        .next()
        .ok_or("Missing width")?
        .parse()
        .map_err(|e| format!("Bad width: {e}"))?;
    let height: usize = tokens
        .next()
        .ok_or("Missing height")?
        .parse()
        .map_err(|e| format!("Bad height: {e}"))?;

    if width == 0 || height == 0 {
        return Err("Zero-sized maze".to_string());
    }

    // Remaining tokens are bits. PBM allows them concatenated without
    // whitespace ("110010"), so flatten by char.
    let bits: Vec<u8> = tokens
        .flat_map(|tok| tok.chars())
        .filter_map(|c| match c {
            '0' => Some(0u8),
            '1' => Some(1u8),
            _ => None,
        })
        .collect();

    let expected = width * height;
    if bits.len() != expected {
        return Err(format!(
            "Pixel count mismatch: expected {expected}, got {}",
            bits.len()
        ));
    }

    let mut maze = vec![vec![Cell::Empty; width]; height];
    for y in 0..height {
        for x in 0..width {
            maze[y][x] = if bits[y * width + x] == 1 {
                Cell::Wall
            } else {
                Cell::Empty
            };
        }
    }
    Ok(maze)
}

/// Serialize a maze to a P1 PBM string. Walls are `1`, empties are `0`.
pub fn maze_to_pbm_str(maze: &MazeGrid) -> String {
    let rows = maze.len();
    let cols = if rows > 0 { maze[0].len() } else { 0 };

    let mut out = String::with_capacity(rows * (cols * 2 + 1) + 16);
    out.push_str("P1\n");
    out.push_str(&format!("{cols} {rows}\n"));

    for row in maze {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push(if *cell == Cell::Wall { '1' } else { '0' });
        }
        out.push('\n');
    }
    out
}

/// Native-only convenience: read a PBM directly from a filesystem path.
#[cfg(not(target_arch = "wasm32"))]
pub fn maze_from_pbm_path(path: &std::path::Path) -> Result<MazeGrid, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    maze_from_pbm_str(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let original = crate::mazes::maze_starting();
        let serialized = maze_to_pbm_str(&original);
        let parsed = maze_from_pbm_str(&serialized).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn rejects_bad_header() {
        assert!(maze_from_pbm_str("P4\n2 2\n0 0 0 0").is_err());
    }

    #[test]
    fn accepts_comments() {
        let m = maze_from_pbm_str("P1\n# created by pathlab\n2 2\n1 0\n0 1\n").unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m[0][0], Cell::Wall);
        assert_eq!(m[0][1], Cell::Empty);
    }
}
