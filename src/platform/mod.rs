//! Platform-abstraction layer.
//!
//! - [`time`] re-exports an `Instant` that works on both native and wasm.
//! - [`fileio`] exposes a uniform "open a maze file" / "save a maze file"
//!   API that picks the right implementation at compile time.

pub mod fileio;
pub mod time;
