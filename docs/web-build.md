# Web build

The same `src/main.rs` compiles to a desktop binary *and* a wasm
module. `eframe` provides both runners; `cfg(target_arch = "wasm32")`
picks the right one at compile time.

## Build with Trunk (recommended)

[Trunk](https://trunkrs.dev) bundles wasm-bindgen + asset hashing +
a dev server. The included `index.html` is already trunk-compatible.

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk serve --release
```

Open <http://127.0.0.1:8080>. Saving a `.rs` file rebuilds + hot-reloads.

## Build with wasm-bindgen-cli

If you'd rather not use Trunk:

```bash
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen \
    target/wasm32-unknown-unknown/release/pathlab.wasm \
    --out-dir dist --target web
cp index.html dist/
python3 -m http.server -d dist 8080
```

The `index.html` imports `./pathlab.js`, so the output names must
match — if you rename the binary in `Cargo.toml`, update the import too.

## The timing pitfall this project hit

A `std::time::Instant::now()` call on `wasm32-unknown-unknown` does
this at runtime:

```text
panicked at 'time not implemented on this platform'
```

…because the wasm target has no monotonic clock in `std`. The
workaround used here is the
[`web-time`](https://docs.rs/web-time) crate, re-exported from
`src/platform/time.rs`. It expands to:

- `std::time::Instant` on native
- `performance.now()` on wasm

Every timing-sensitive site in the app imports `Instant` and `Duration`
from `crate::platform::time` rather than `std::time`, so the desktop
and web binaries behave identically. **If you add new timing logic,
import from `crate::platform::time` to keep it that way.**

## CORS / cross-origin isolation

The app currently does not use `SharedArrayBuffer`, so it works from
any plain static-file server. If you later enable threading or
`wgpu`-multithread features, you'll need the COOP/COEP headers:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

`trunk serve` sets these automatically when `--cross-origin-isolation`
is passed.

## Browser file pickers

The native build uses the `rfd` crate. On the web that path is unused;
instead, `platform::fileio` creates a transient `<input type="file">`
element, attaches a `change` listener, calls `.click()`, and pipes the
resulting `FileReader` text into the same `maze_from_pbm_str` parser
the desktop side uses.

The picker is asynchronous (a JS callback delivers the result), so
the result is routed through a `FileInbox` (`Arc<Mutex<...>>`). The
app polls the inbox once per frame inside `tick_solver`.

## Known limitations

- `image` crate-based formats (PNG, JPG) are not enabled to keep the
  wasm payload small. If you want them, add `image` to the dependency
  list and a parser arm to `mazes::pbm` (or split it into a separate
  module).
- The web build doesn't persist preferences between reloads — every
  visit starts on the "Starting" maze.
