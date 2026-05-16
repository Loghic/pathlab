//! Open / save dialogs for maze files.
//!
//! The web variant is asynchronous (the browser only exposes file pickers
//! via DOM events), so both implementations expose the same fire-and-forget
//! API: the result is delivered to the app through a shared cell that the
//! app polls each frame.

use std::sync::{Arc, Mutex};

use crate::mazes::MazeGrid;

/// Shared inbox: the file-picker writes here when the user picks a file,
/// the app reads and clears it each frame.
#[derive(Clone, Default)]
pub struct FileInbox {
    inner: Arc<Mutex<Option<Result<MazeGrid, String>>>>,
}

impl FileInbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&self, result: Result<MazeGrid, String>) {
        if let Ok(mut slot) = self.inner.lock() {
            *slot = Some(result);
        }
    }

    /// Take any pending result. Returns `None` if no file has been picked
    /// since the last call.
    pub fn take(&self) -> Option<Result<MazeGrid, String>> {
        self.inner.lock().ok().and_then(|mut s| s.take())
    }
}

// =====================================================================
// Native implementation
// =====================================================================
#[cfg(not(target_arch = "wasm32"))]
pub fn open_maze(inbox: &FileInbox) {
    use rfd::FileDialog;

    let Some(path) = FileDialog::new()
        .add_filter("PBM image", &["pbm"])
        .set_directory("assets")
        .pick_file()
    else {
        return;
    };

    let result = crate::mazes::maze_from_pbm_path(&path);
    inbox.put(result);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_maze(maze: &MazeGrid) {
    use rfd::FileDialog;
    use std::io::Write;

    let Some(path) = FileDialog::new()
        .add_filter("PBM image", &["pbm"])
        .set_directory("assets")
        .save_file()
    else {
        return;
    };

    let pbm = crate::mazes::maze_to_pbm_str(maze);
    match std::fs::File::create(&path).and_then(|mut f| f.write_all(pbm.as_bytes())) {
        Ok(()) => {}
        Err(e) => eprintln!("Failed to save {}: {e}", path.display()),
    }
}

// =====================================================================
// Web (wasm32) implementation
// =====================================================================
#[cfg(target_arch = "wasm32")]
pub fn open_maze(inbox: &FileInbox) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::{FileReader, HtmlInputElement};

    let document = web_sys::window().unwrap().document().unwrap();
    let input: HtmlInputElement = document
        .create_element("input")
        .unwrap()
        .dyn_into()
        .unwrap();
    input.set_type("file");
    input.set_accept(".pbm");

    let inbox_for_change = inbox.clone();

    let onchange = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        let Some(files) = input.files() else { return };
        let Some(file) = files.get(0) else { return };

        let reader = FileReader::new().unwrap();
        let reader_for_load = reader.clone();
        let inbox_for_load = inbox_for_change.clone();

        let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let Ok(value) = reader_for_load.result() else {
                inbox_for_load.put(Err("FileReader returned no value".into()));
                return;
            };
            let Some(text) = value.as_string() else {
                inbox_for_load.put(Err("FileReader produced a non-string result".into()));
                return;
            };
            inbox_for_load.put(crate::mazes::maze_from_pbm_str(&text));
        }) as Box<dyn FnMut(_)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        if let Err(_) = reader.read_as_text(&file) {
            inbox_for_change.put(Err("FileReader could not read the file".into()));
        }
        onload.forget();
    }) as Box<dyn FnMut(_)>);

    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    input.click();
    onchange.forget();
}

#[cfg(target_arch = "wasm32")]
pub fn save_maze(maze: &MazeGrid) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, HtmlAnchorElement, Url};

    let pbm = crate::mazes::maze_to_pbm_str(maze);

    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(&pbm));

    let Ok(blob) = Blob::new_with_str_sequence(&array) else {
        return;
    };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else {
        return;
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let anchor: HtmlAnchorElement = document.create_element("a").unwrap().dyn_into().unwrap();
    anchor.set_href(&url);
    anchor.set_download("maze.pbm");
    anchor.click();

    let _ = Url::revoke_object_url(&url);
}
