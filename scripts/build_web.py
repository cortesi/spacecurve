#!/usr/bin/env python3
"""Build the production web bundle for spacecurve."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET_DIR = ROOT / "target" / "wasm32-unknown-unknown" / "wasm-release"
RAW_WASM = TARGET_DIR / "scurve-web.wasm"
DIST_DIR = ROOT / "dist"

INDEX_HTML = """<!DOCTYPE html>
<html>
<head>
  <meta charset=\"utf-8\">
  <title>spacecurve — Web</title>
  <link rel=\"icon\" href=\"data:,\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
  <style>
    html, body { margin: 0; padding: 0; height: 100%; overflow: hidden; background: #2c3e50; font-family: Arial, sans-serif; }
    canvas { display: block; width: 100vw; height: 100vh; border: 2px solid #34495e; border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,.3); }
    .loading { position: absolute; bottom: 16px; left: 50%; transform: translateX(-50%); color: white; font-size: 1.1em; }
  </style>
</head>
<body>
  <canvas id=\"bevy\"></canvas>
  <div class=\"loading\" id=\"loading\">Loading...</div>

  <script type=\"module\">
    import init from './scurve-web.js';
    init().then(() => {
      const loading = document.getElementById('loading');
      if (loading) loading.style.display = 'none';
    }).catch(console.error);
  </script>
</body>
</html>
"""

FALLBACK_HTML = """<!DOCTYPE html>
<html>
<head>
  <meta charset=\"utf-8\">
  <title>spacecurve — Web (fallback)</title>
  <link rel=\"icon\" href=\"data:,\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
</head>
<body>
  <p>Fallback build created. To produce a working web bundle, install wasm-bindgen CLI:</p>
  <pre>cargo install wasm-bindgen-cli</pre>
  <p>Then re-run <code>uv run ./scripts/build_web.py</code>.</p>
  <p>Raw wasm artifact is at <code>./dist/scurve-web.wasm</code>.</p>
</body>
</html>
"""


def run_command(command: list[str]) -> None:
    """Run a subprocess while streaming output."""
    print(f"$ {' '.join(command)}")
    subprocess.run(command, check=True, cwd=ROOT)


def ensure_raw_wasm_exists() -> None:
    if not RAW_WASM.exists():
        raise SystemExit(
            f"Expected wasm artifact at {RAW_WASM}, but it does not exist. "
            "Ensure the build completed successfully."
        )


def human_size(num_bytes: int) -> str:
    units = ["B", "KB", "MB", "GB", "TB"]
    amount = float(num_bytes)
    for unit in units:
        if amount < 1024 or unit == units[-1]:
            return f"{amount:.1f} {unit}"
        amount /= 1024
    return f"{amount:.1f} PB"


def describe_file(path: Path) -> str:
    size = human_size(path.stat().st_size)
    return f"{size} \t{path.name}"


def prepare_dist() -> None:
    if DIST_DIR.exists():
        print(f"Removing existing {DIST_DIR} ...")
        shutil.rmtree(DIST_DIR)
    DIST_DIR.mkdir(parents=True, exist_ok=True)


def run_wasm_bindgen() -> bool:
    if shutil.which("wasm-bindgen") is None:
        print("wasm-bindgen not found; creating fallback bundle.")
        return False

    print("Running wasm-bindgen (target=web, no TS) ...")
    run_command(
        [
            "wasm-bindgen",
            "--target",
            "web",
            "--no-typescript",
            "--out-dir",
            str(DIST_DIR),
            str(RAW_WASM),
        ]
    )
    return True


def maybe_optimize_wasm(bg_wasm: Path) -> None:
    if not bg_wasm.exists():
        print("wasm-bindgen output missing; skipping wasm-opt.")
        return

    if shutil.which("wasm-opt") is None:
        print("wasm-opt not found; skipping additional optimization.")
        return

    print("Optimizing wasm via wasm-opt -Oz ...")
    run_command(["wasm-opt", "-Oz", "-o", str(bg_wasm), str(bg_wasm)])


def write_index(html: str) -> None:
    DIST_DIR.joinpath("index.html").write_text(html, encoding="utf-8")


def emit_fallback_bundle() -> None:
    shutil.copy2(RAW_WASM, DIST_DIR / "scurve-web.wasm")
    write_index(FALLBACK_HTML)


def emit_production_bundle() -> None:
    bg_wasm = DIST_DIR / "scurve_web_bg.wasm"
    maybe_optimize_wasm(bg_wasm)
    write_index(INDEX_HTML)


def main() -> None:
    print("Building scurve-web (wasm-release)...")
    run_command(
        [
            "cargo",
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--bin",
            "scurve-web",
            "--profile",
            "wasm-release",
        ]
    )

    ensure_raw_wasm_exists()
    size_info = describe_file(RAW_WASM)
    print(f"WASM raw size:\n{size_info}")

    print("Preparing dist/ directory...")
    prepare_dist()

    if run_wasm_bindgen():
        emit_production_bundle()
    else:
        emit_fallback_bundle()

    print()
    print("Build complete. Deploy the contents of dist/ via any static web server.")
    print("Included artifacts:")
    for path in sorted(DIST_DIR.iterdir()):
        if path.is_file():
            print(describe_file(path))
    print()
    print("Next steps:")
    print(
        "  uv run ./scripts/serve_dist.py 8000    # Serve dist/ locally and open in a browser"
    )


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode) from exc
    except SystemExit:
        raise
    except Exception as exc:  # noqa: BLE001
        raise SystemExit(f"build failed: {exc}") from exc
