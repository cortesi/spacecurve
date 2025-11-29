# Developer Guide

## Quick Setup
- Install Rust toolchain.
- For web work: `uv run ./scripts/setup_web.py` (installs `wasm32-unknown-unknown`, `wasm-server-runner`, `wasm-bindgen-cli`).

## Core Commands
- Native dev: `cargo run`
- Native release: `cargo build --release`

### Web Dev
- Live dev server: `uv run ./scripts/serve_web.py` (uses `wasm-server-runner`).
- Alt: `cargo run --target wasm32-unknown-unknown --bin scurve-web`.
- Build prod bundle: `uv run ./scripts/build_web.py`.
- Serve bundle: `uv run ./scripts/serve_dist.py 8000` (any HTTP server works; don’t use `file://`).
- Optional local aliases (add to `.cargo/config.toml`):
  - `serve-web = "run --target wasm32-unknown-unknown --bin scurve-web"`
  - `build-web = "build --target wasm32-unknown-unknown --bin scurve-web --profile wasm-release"`

Prod output: `dist/` with `index.html`, `scurve-web.js`, `scurve-web_bg.wasm` (auto-optimized with `wasm-opt` if available).

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
1) `uv run ./scripts/build_web.py`
2) Serve `dist/` via HTTP (`uv run ./scripts/serve_dist.py 8000` or any static server).
3) Files: `index.html`, `scurve-web.js`, `scurve-web_bg.wasm`.
