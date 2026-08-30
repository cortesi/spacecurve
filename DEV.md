# Developer Guide

## Web Dev
- One-time setup: `cargo xtask web setup`
- Re-run setup after `wasm-bindgen` dependency bumps. It force-refreshes both
  `wasm-server-runner` and `wasm-bindgen-cli`.
- Live dev server: `cargo xtask web serve` (uses `wasm-server-runner`, not the standalone
  `wasm-bindgen` binary).
- Build prod bundle: `cargo xtask web build`.
- Serve bundle: `cargo xtask web serve-dist 8000` (any HTTP server works; don’t use `file://`).

Prod output: `dist/` with `index.html`, `scurve-web.js`, `scurve-web_bg.wasm` (auto-optimized with `wasm-opt` if available).

## Tidy
- Format + clippy (with fixes): `nanocode tidy`

## Experimental curves
- Experimental patterns (currently Hairy Onion) are hidden in the GUI by default.
- Native GUI: run `cargo run -- scurve gui --dev` to expose experimental curves.
- Web GUI: append `?dev=1` (or `?experimental=1`) to the served page URL to show them.

## GUI Screenshots
- Build with feature: `cargo build --package scurve --features screenshot`
- Panes: `2d`, `3d`, `about`, `settings`, `settings-3d` (3D settings shows spin speed).
- Capture: `cargo run --package scurve --features screenshot -- screenshot -p <pane> /tmp/out.png`
- Behavior: waits one extra frame so overlays (About, settings) render; single-frame capture then exit.

Handy for styling checks: run the command above and view the PNG (e.g., with the Read tool).

## Debugging the egui image viewer
- Quick capture for centering/layout: `cargo run -p egui-img --example debug_viewer assets/hilbert.png --screenshot /tmp/view.png`
- The helper `egui_img::view_image_with_screenshot` renders one frame, saves the PNG, then closes.

## Deployment (Web)
1) `cargo xtask web build`
2) Serve `dist/` via HTTP (`cargo xtask web serve-dist 8000` or any static server).
3) Files: `index.html`, `scurve-web.js`, `scurve-web_bg.wasm`.

## README snippets

Managed with snips: 

```sh
cargo install snips
```

Then run:

```sh
snips ./README.md
```
