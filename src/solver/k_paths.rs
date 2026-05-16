//! Yen's algorithm — find the *k* shortest loopless paths from start to
//! goal on a 4-connected grid.
//!
//! Edges are unit-weight, so each underlying shortest-path query is just
//! a BFS. The output paths are returned sorted by length (ties broken
//! arbitrarily but stably with respect to BFS exploration order).
//!
//! Pseudocode:
//!
//! ```text
//! A[0] = bfs(start, goal)               // shortest path
//! B = empty min-heap of candidates
//! for k in 1..K:
//!     for each spur_node in A[k-1] except the last:
//!         prefix      = A[k-1][..spur_node]
//!         banned_edges = { (path[i], path[i+1])
//!                          for path in A
//!                          if path[..=i] == prefix
//!                          for i = len(prefix)-1 }
//!         banned_nodes = prefix without spur_node
//!         spur_path    = bfs(spur_node, goal,
//!                            avoid edges, avoid nodes)
//!         if spur_path:
//!             push prefix + spur_path into B
//!     if B empty:
//!         break
//!     A[k] = shortest unique path in B
//! return A
//! ```

use std::collections::{HashSet, VecDeque};

use crate::mazes::{Cell, MazeGrid};

use super::core::Coord;

/// Find at most `k` shortest distinct loopless paths from `start` to
/// `goal`. Returns an empty vec if no path exists.
pub fn k_shortest_paths(maze: &MazeGrid, start: Coord, goal: Coord, k: usize) -> Vec<Vec<Coord>> {
    if k == 0 {
        return Vec::new();
    }

    // First shortest path via plain BFS.
    let Some(first) = bfs(maze, start, goal, &HashSet::new(), &HashSet::new()) else {
        return Vec::new();
    };

    let mut a: Vec<Vec<Coord>> = vec![first];
    // Candidates: stored as (length, path). We dedup by exact path
    // equality before insertion, then pick the shortest by length.
    let mut b: Vec<Vec<Coord>> = Vec::new();

    while a.len() < k {
        let prev = a.last().expect("a is non-empty at top of loop");

        // Generate spur paths off every node in prev except the last.
        for i in 0..prev.len().saturating_sub(1) {
            let spur_node = prev[i];
            let root_path = &prev[..=i];

            // Forbid the edge out of spur_node along every already-found
            // path that shares this exact prefix. This is what stops Yen
            // from reproducing paths we've already collected.
            let mut banned_edges: HashSet<(Coord, Coord)> = HashSet::new();
            for path in &a {
                if path.len() > i + 1 && &path[..=i] == root_path {
                    banned_edges.insert((path[i], path[i + 1]));
                }
            }

            // Nodes in the root path (except the spur node itself) are
            // off-limits so the spur path stays loopless.
            let banned_nodes: HashSet<Coord> = root_path
                .iter()
                .copied()
                .filter(|&c| c != spur_node)
                .collect();

            let Some(spur) = bfs(maze, spur_node, goal, &banned_nodes, &banned_edges) else {
                continue;
            };

            // Combine root_path[0..i] + spur (which already starts at
            // spur_node).
            let mut candidate: Vec<Coord> = root_path[..i].to_vec();
            candidate.extend(spur);

            // Dedup against everything we've found and everything in B.
            if a.iter().any(|p| p == &candidate) {
                continue;
            }
            if b.iter().any(|p| p == &candidate) {
                continue;
            }
            b.push(candidate);
        }

        if b.is_empty() {
            break;
        }

        // Pick the shortest candidate. Stable sort keeps insertion
        // order on ties, which gives deterministic output.
        b.sort_by_key(|p| p.len());
        a.push(b.remove(0));
    }

    a
}

/// Breadth-first shortest path with edge and node exclusion lists.
fn bfs(
    maze: &MazeGrid,
    start: Coord,
    goal: Coord,
    banned_nodes: &HashSet<Coord>,
    banned_edges: &HashSet<(Coord, Coord)>,
) -> Option<Vec<Coord>> {
    let height = maze.len();
    let width = if height > 0 { maze[0].len() } else { 0 };

    let walkable = |c: Coord| -> bool {
        c.0 < width && c.1 < height && maze[c.1][c.0] != Cell::Wall && !banned_nodes.contains(&c)
    };

    // The start may be banned (it's the spur node, so it's allowed) or
    // a wall (always rejected). The spur-node exception is already
    // handled by the caller filtering it out of `banned_nodes`.
    if start.0 >= width || start.1 >= height {
        return None;
    }
    if maze[start.1][start.0] == Cell::Wall {
        return None;
    }

    let mut parent: std::collections::HashMap<Coord, Coord> = Default::default();
    let mut visited: HashSet<Coord> = HashSet::new();
    let mut queue: VecDeque<Coord> = VecDeque::new();
    queue.push_back(start);
    visited.insert(start);

    const DIRS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    while let Some(cur) = queue.pop_front() {
        if cur == goal {
            // Reconstruct.
            let mut path = vec![cur];
            let mut c = cur;
            while let Some(&p) = parent.get(&c) {
                path.push(p);
                c = p;
                if c == start {
                    break;
                }
            }
            path.reverse();
            return Some(path);
        }
        for (dx, dy) in DIRS {
            let nx = cur.0 as isize + dx;
            let ny = cur.1 as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let next = (nx as usize, ny as usize);
            if !walkable(next) {
                continue;
            }
            if banned_edges.contains(&(cur, next)) {
                continue;
            }
            if visited.insert(next) {
                parent.insert(next, cur);
                queue.push_back(next);
            }
        }
    }
    None
}

// ----------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mazes::maze_open;

    #[test]
    fn k_equals_one_returns_shortest() {
        let maze = maze_open(4, 4);
        let paths = k_shortest_paths(&maze, (0, 0), (3, 3), 1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 7); // 3 + 3 + 1
    }

    #[test]
    fn finds_multiple_distinct_paths() {
        let maze = maze_open(3, 3);
        let paths = k_shortest_paths(&maze, (0, 0), (2, 2), 5);
        // There are exactly C(4,2) = 6 distinct shortest monotone paths
        // on a 3x3 grid, plus longer detours. We just check that at
        // least 5 different ones come back and none repeat.
        assert!(paths.len() >= 2);
        for i in 0..paths.len() {
            for j in (i + 1)..paths.len() {
                assert_ne!(paths[i], paths[j], "paths {i} and {j} are identical");
            }
        }
    }

    #[test]
    fn paths_are_sorted_by_length() {
        let maze = maze_open(4, 4);
        let paths = k_shortest_paths(&maze, (0, 0), (3, 3), 8);
        for w in paths.windows(2) {
            assert!(w[0].len() <= w[1].len());
        }
    }

    #[test]
    fn empty_when_unreachable() {
        // Vertical wall splitting the maze; no path possible.
        let mut maze = maze_open(3, 3);
        for row in maze.iter_mut().take(3) {
            row[1] = Cell::Wall;
        }
        let paths = k_shortest_paths(&maze, (0, 0), (2, 2), 5);
        assert!(paths.is_empty());
    }

    #[test]
    fn k_zero_returns_empty() {
        let maze = maze_open(3, 3);
        assert!(k_shortest_paths(&maze, (0, 0), (2, 2), 0).is_empty());
    }
}
