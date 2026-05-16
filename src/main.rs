//! Entry point for the maze solver.
//!
//! The same source compiles to both a native desktop binary (via `eframe`'s
//! native runner) and a WebAssembly module (via `eframe::WebRunner`). All
//! UI logic lives in [`pathlab::app`]; this file only handles boot.

use pathlab::app::MazeApp;

// -------------------------------------------------------------------------
// Native entry point
// -------------------------------------------------------------------------
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Pathlab"),
        ..Default::default()
    };

    eframe::run_native(
        "Pathlab",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MazeApp>::default())
        }),
    )
}

// -------------------------------------------------------------------------
// Web (wasm32) entry point
// -------------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    // Forward Rust panics to the browser dev console.
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("no global `window`");
        let document = window.document().expect("no document on window");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("canvas with id 'the_canvas_id' not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element is not a HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Ok(Box::<MazeApp>::default())
                }),
            )
            .await
            .expect("failed to start eframe web app");
    });
}
