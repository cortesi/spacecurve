# Scurve

Command-line tool for space-filling curve visualization and binary file analysis.

This is the main CLI application that provides various commands for generating curve visualizations, analyzing binary files, and launching the interactive GUI.

## Features

- **Binary file visualization**: Convert binary data into visual patterns using space-filling curves
- **Pattern generation**: Create maps and visualizations of different curve types
- **AllRGB generation**: Generate dense color maps using all RGB values
- **Interactive GUI**: Launch the GUI interface for real-time exploration
- **Multiple output formats**: Save results as image files or display interactively

## Installation

```bash
cargo install --path .
```

## Usage

### Commands

#### Visualize a Binary File
```bash
scurve vis -p hilbert -w 512 input.bin
```

#### Generate a Curve Pattern Map  
```bash
scurve map -s 512 -w 2 -d 16 hilbert
```

#### Create AllRGB Visualization
```bash
scurve allrgb -c hilbert zorder
```

#### Launch Interactive GUI
```bash
scurve gui
```

### Options

- `-p, --pattern`: Space-filling curve pattern (hilbert, zorder, etc.)
- `-w, --width` (vis): Output image width/height for `vis`
- `-s, --size` (map): Square output size for `map`
- `-w, --line-width` (map): Line width in pixels for `map`
- `-d, --dimension`: Side length of the curve grid (renders `dimension×dimension` points)
- `--fg, --foreground`: Foreground stroke color for `map` (named colours or hex with optional alpha, `#` optional)
- `--bg, --background`: Background color for `map` (named colours or hex with optional alpha, `#` optional)
- `-c, --colormap`: Color mapping pattern for AllRGB
- Omit the final `output` path on `map`, `vis`, or `allrgb` to open a native egui preview window

Map dimensions are rounded up to the nearest valid size for the selected curve (e.g., a Hilbert
curve requested with `-d 3` renders using `-d 4` and prints a warning).

### Available Curve Types

- `hilbert` - Hilbert curve
- `zorder` - Z-order (Morton) curve  
- `scan` - Linear scan
- And more...

## Architecture

This CLI tool uses the `scurve-gui` crate for the interactive GUI functionality and the `spacecurve` crate for curve generation algorithms. The visualization commands use native libraries for image processing and window management.

## Dependencies

- spacecurve — core curve-generation algorithms
- scurve-gui — interactive GUI (built on egui/eframe)
- clap — command-line argument parsing
- image — image encoding/decoding
- pbr — simple progress bar for long-running ops
- memmap2 — memory-mapped file I/O used by `vis`
- eframe, egui, egui_commonmark, webbrowser — GUI stack used via `scurve-gui`

Note: the project no longer uses piston_window.
