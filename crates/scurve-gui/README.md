# Scurve GUI

Interactive space-filling curve visualization GUI library and web application.

This crate provides the GUI components for space-filling curve visualization. It can be used as a library by other applications (like the `scurve` CLI) or run directly as a web application.

## Features

- **Complete GUI functionality**: Interactive 2D and 3D curve visualizations
- **Web-first design**: Optimized for WebGL2 rendering in browsers
- **Library interface**: Can be imported by other Rust applications
- **Multiple curve types**: Support for various space-filling curve algorithms
- **Responsive design**: Adapts to different screen sizes

## Usage

### As a Library
Add to your `Cargo.toml`:
```toml
[dependencies]
scurve-gui = { path = "../scurve-gui" }
```

Then call the GUI function:
```rust
scurve_gui::gui()?;
```

### Web Application
1. **Install required tools (run from the repository root):**
   ```bash
   uv run ./scripts/setup_web.py
   ```

2. **Run development server (run from the repository root):**
   ```bash
   uv run ./scripts/serve_web.py
   ```

   Open `http://127.0.0.1:1334` in your browser (uses wasm-server-runner).

## Dependencies

### Core Dependencies
- **spacecurve** - Space-filling curve generation algorithms
- **anyhow** - Error handling
- **bevy 0.16.1** - Game engine with excellent cross-platform support
- **bevy_egui 0.35.1** - Immediate mode GUI integration
- **getrandom 0.3** - Random number generation with WASM support

### Native-only Dependencies
- **clap** - Command-line argument parsing
- **memmap2** - Memory-mapped file I/O
- **palette** - Color manipulation
- **image** - Image processing
- **piston_window** - Alternative windowing backend
- **rand** - Random number generation
- **pbr** - Progress bar utilities

### Web-specific Dependencies
- **wasm-bindgen** - Rust/JavaScript interop
- **console_error_panic_hook** - Better error reporting in browsers

## Project Structure

```
crates/scurve/
├── src/
│   ├── main.rs          # Native CLI binary
│   ├── web.rs           # Web-only binary (GUI only)
│   ├── cmd.rs           # Command-line interface
│   ├── lib.rs           # Library exports
│   └── gui/             # Shared GUI module
│       ├── mod.rs       # GUI module exports
│       ├── threed.rs    # 3D visualization
│       ├── twod.rs      # 2D visualization
│       └── widgets.rs   # UI widgets
├── assets/              # Web assets
│   └── index.html       # Web page template
├── .cargo/
│   └── config.toml      # WASM build configuration & aliases
├── index.html           # Symlink to assets/index.html
├── scripts/setup_web.py # Tool installation script
├── README.md            # This file
└── DEV.md               # Development guide
```

## Build Targets

The crate supports multiple build targets:

- **Native executable**: `scurve` - Full CLI with GUI support
- **Web application**: `scurve-web` - GUI-only for browser deployment
- **Library**: Available as both cdylib and rlib for integration

## Requirements

### Native
- Rust toolchain
- Graphics drivers supporting OpenGL

### Web
- Rust toolchain with `wasm32-unknown-unknown` target
- wasm-server-runner for development (installed by setup script)
- Modern web browser with WebGL2 support

## Browser Compatibility

Tested and working on:
- Chrome 80+
- Firefox 79+
- Safari 14+
- Edge 80+

Requires WebGL2 support (available in all modern browsers).
