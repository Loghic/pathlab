//! Central panel: draws the maze grid and the solver's overlay.

use eframe::egui;

use crate::mazes::Cell;

use super::state::{InteractionMode, MazeApp};

// ---------------------------------------------------------------------
// Visual constants. Tweak here, not in the drawing code.
// ---------------------------------------------------------------------

/// Fill for the start cell.
const START_FILL: egui::Color32 = egui::Color32::from_rgb(50, 200, 90);
/// Fill for the end cell.
const END_FILL: egui::Color32 = egui::Color32::from_rgb(230, 60, 90);
/// Border drawn around start/end cells for contrast on both light and
/// dark themes.
const ENDPOINT_BORDER: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);
/// Inner highlight ring on start/end cells.
const ENDPOINT_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);

/// Path color — chosen to stand out on both white empties and black walls.
const PATH_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 215, 0);
/// Dark outline drawn underneath the path for readability.
const PATH_OUTLINE: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);

/// Palette used when the active algorithm returns multiple paths
/// (currently just [`crate::solver::Algorithm::KShortest`]). The
/// shortest path always renders in entry 0 (gold, matching the single-
/// path PATH_COLOR) so the eye still anchors there. Subsequent paths
/// get progressively cooler hues. If `k` exceeds the palette length the
/// colours wrap.
const MULTI_PATH_PALETTE: [egui::Color32; 8] = [
    egui::Color32::from_rgb(255, 215, 0),   // gold     - shortest
    egui::Color32::from_rgb(255, 110, 60),  // orange
    egui::Color32::from_rgb(230, 70, 130),  // pink
    egui::Color32::from_rgb(160, 90, 220),  // purple
    egui::Color32::from_rgb(80, 140, 255),  // blue
    egui::Color32::from_rgb(40, 200, 200),  // teal
    egui::Color32::from_rgb(80, 220, 110),  // green
    egui::Color32::from_rgb(200, 200, 110), // khaki
];

const CLOSED_COLOR: egui::Color32 = egui::Color32::from_rgb(120, 120, 130);
const OPEN_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 140, 0);

pub(super) fn show(app: &mut MazeApp, ui: &mut egui::Ui) {
    let available = ui.available_size();

    if app.auto_fit && available != app.last_canvas_size {
        let nx = available.x / app.cols.max(1) as f32;
        let ny = available.y / app.rows.max(1) as f32;
        app.target_cell_size = nx.min(ny).clamp(5.0, 100.0);
    }
    app.last_canvas_size = available;

    // Smoothly chase the target size for a nicer feel during resize.
    app.cell_size = egui::lerp(app.cell_size..=app.target_cell_size, 0.3);

    let maze_w = app.cols as f32 * app.cell_size;
    let maze_h = app.rows as f32 * app.cell_size;
    let offset = egui::vec2(
        (available.x - maze_w).max(0.0) * 0.5,
        (available.y - maze_h).max(0.0) * 0.5,
    );
    let origin = ui.min_rect().min + offset;
    let maze_rect = egui::Rect::from_min_size(origin, egui::vec2(maze_w, maze_h));

    let response = ui.allocate_rect(maze_rect, egui::Sense::click_and_drag());
    let painter = ui.painter_at(maze_rect);

    handle_input(app, &response, origin, maze_rect);
    draw_grid(app, &painter, origin);
    draw_solver_overlay(app, &painter, origin);
    draw_endpoints(app, &painter, origin);
}

/// Handle clicks and drag strokes on the maze canvas.
///
/// Drag-strokes push exactly one undo snapshot per stroke, taken at
/// `drag_started`, regardless of how many cells the user paints over.
fn handle_input(
    app: &mut MazeApp,
    response: &egui::Response,
    origin: egui::Pos2,
    maze_rect: egui::Rect,
) {
    let is_editing = matches!(
        app.interaction,
        InteractionMode::DrawWall | InteractionMode::EraseWall
    );

    // Snapshot at the start of an editing stroke or on a single click.
    // drag_started() fires once per drag; clicked() fires once per click.
    if is_editing && (response.drag_started() || response.clicked()) {
        app.snapshot_for_undo();
    }

    let Some(pos) = response.interact_pointer_pos() else {
        return;
    };
    if !maze_rect.contains(pos) {
        return;
    }
    let col = ((pos.x - origin.x) / app.cell_size).floor() as isize;
    let row = ((pos.y - origin.y) / app.cell_size).floor() as isize;
    if col < 0 || row < 0 {
        return;
    }
    let (col, row) = (col as usize, row as usize);
    if row >= app.rows || col >= app.cols {
        return;
    }

    match app.interaction {
        InteractionMode::DrawWall => {
            app.maze[row][col] = Cell::Wall;
        }
        InteractionMode::EraseWall => {
            app.maze[row][col] = Cell::Empty;
        }
        InteractionMode::PickStart if response.clicked() => {
            app.start.x = col;
            app.start.y = row;
            app.interaction = InteractionMode::None;
        }
        InteractionMode::PickEnd if response.clicked() => {
            app.end.x = col;
            app.end.y = row;
            app.interaction = InteractionMode::None;
        }
        _ => {}
    }
}

fn draw_grid(app: &MazeApp, painter: &egui::Painter, origin: egui::Pos2) {
    for row in 0..app.rows {
        for col in 0..app.cols {
            // Endpoints are drawn last (on top of overlays) — here we
            // only draw the base wall/empty fill.
            let is_start = row == app.start.y && col == app.start.x;
            let is_end = row == app.end.y && col == app.end.x;
            if is_start || is_end {
                continue;
            }

            let rect = egui::Rect::from_min_size(
                egui::pos2(
                    origin.x + col as f32 * app.cell_size,
                    origin.y + row as f32 * app.cell_size,
                ),
                egui::vec2(app.cell_size, app.cell_size),
            );

            let color = match app.maze[row][col] {
                Cell::Empty => egui::Color32::WHITE,
                Cell::Wall => egui::Color32::BLACK,
            };

            painter.rect_filled(rect, 0.0, color);
            painter.rect_stroke(
                rect,
                0.0,
                (1.0, egui::Color32::GRAY),
                egui::StrokeKind::Outside,
            );
        }
    }
}

fn draw_solver_overlay(app: &MazeApp, painter: &egui::Painter, origin: egui::Pos2) {
    let Some(solver) = &app.solver else { return };
    let radius = (app.cell_size * 0.2).max(2.0);

    let cell_center = |col: usize, row: usize| {
        egui::pos2(
            origin.x + (col as f32 + 0.5) * app.cell_size,
            origin.y + (row as f32 + 0.5) * app.cell_size,
        )
    };

    // Explored cells.
    for &(col, row) in solver.closed_cells() {
        painter.circle_filled(cell_center(col, row), radius, CLOSED_COLOR);
    }

    // Frontier.
    for &(col, row) in solver.open_cells() {
        painter.circle_filled(cell_center(col, row), radius, OPEN_COLOR);
    }

    // Paths. For single-path algorithms `paths()` has exactly one entry
    // and the rendering is identical to before. For K-Shortest, every
    // path gets a distinct colour from MULTI_PATH_PALETTE. We draw
    // longest first so the shortest ends up on top of the stack.
    let core_width = (app.cell_size * 0.28).max(3.0);
    let outline_width = core_width + 2.5;

    let mut ordered: Vec<(usize, &Vec<(usize, usize)>)> =
        solver.paths().iter().enumerate().collect();
    // Longest first; ties keep their original order.
    ordered.sort_by_key(|entry| std::cmp::Reverse(entry.1.len()));

    for (idx, path) in ordered {
        if path.len() < 2 {
            continue;
        }
        let points: Vec<egui::Pos2> = path.iter().map(|&(c, r)| cell_center(c, r)).collect();

        let colour = if solver.paths().len() <= 1 {
            PATH_COLOR
        } else {
            MULTI_PATH_PALETTE[idx % MULTI_PATH_PALETTE.len()]
        };

        painter.add(egui::Shape::line(
            points.clone(),
            egui::Stroke::new(outline_width, PATH_OUTLINE),
        ));
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(core_width, colour),
        ));
    }
}

/// Draw start and end on top of everything else so they're always
/// visible, even when the path or open-set markers overlap them.
fn draw_endpoints(app: &MazeApp, painter: &egui::Painter, origin: egui::Pos2) {
    for (point, fill, label) in [(app.start, START_FILL, "S"), (app.end, END_FILL, "E")] {
        if point.x >= app.cols || point.y >= app.rows {
            continue;
        }
        let rect = egui::Rect::from_min_size(
            egui::pos2(
                origin.x + point.x as f32 * app.cell_size,
                origin.y + point.y as f32 * app.cell_size,
            ),
            egui::vec2(app.cell_size, app.cell_size),
        );

        // Filled background.
        painter.rect_filled(rect, 0.0, fill);

        // Thick dark border for contrast against neighbouring cells.
        let border = (app.cell_size * 0.08).max(2.0);
        painter.rect_stroke(
            rect.shrink(border * 0.5),
            0.0,
            egui::Stroke::new(border, ENDPOINT_BORDER),
            egui::StrokeKind::Inside,
        );

        // Inner highlight ring so the cell reads as a "marker" not just
        // a coloured square.
        let inner = rect.shrink(border + (app.cell_size * 0.12).max(2.0));
        if inner.width() > 0.0 && inner.height() > 0.0 {
            painter.rect_stroke(
                inner,
                0.0,
                egui::Stroke::new((app.cell_size * 0.05).max(1.0), ENDPOINT_HIGHLIGHT),
                egui::StrokeKind::Inside,
            );
        }

        // S / E letter, scaled to the cell. Skip if the cell is too tiny.
        if app.cell_size >= 14.0 {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional((app.cell_size * 0.55).max(10.0)),
                ENDPOINT_BORDER,
            );
        }
    }
}
