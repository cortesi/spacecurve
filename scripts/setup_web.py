#!/usr/bin/env python3
"""Install toolchain requirements for building and serving scurve-web."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

REQUIRED_COMMANDS = ["rustup", "cargo"]


def ensure_prereqs() -> None:
    missing = [cmd for cmd in REQUIRED_COMMANDS if shutil.which(cmd) is None]
    if missing:
        joined = ", ".join(missing)
        raise SystemExit(
            f"Missing required command(s): {joined}. Install Rust (https://rustup.rs) first."
        )


def run_command(command: list[str]) -> None:
    print(f"$ {' '.join(command)}")
    subprocess.run(command, check=True, cwd=ROOT)


def install_tools() -> None:
    print("Adding wasm32-unknown-unknown target...")
    run_command(["rustup", "target", "add", "wasm32-unknown-unknown"])

    print("Installing wasm-server-runner...")
    run_command(["cargo", "install", "wasm-server-runner"])

    print("Installing wasm-bindgen-cli...")
    run_command(["cargo", "install", "wasm-bindgen-cli"])


def main() -> None:
    print("Setting up web compilation tools for Scurve...")
    ensure_prereqs()
    install_tools()

    print()
    print("Setup complete!")
    print()
    print("Next steps:")
    print("  uv run ./scripts/serve_web.py   # Start development server")
    print("  uv run ./scripts/build_web.py   # Build optimized version")
    print()
    print("See DEV.md for detailed development guide.")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode) from exc
    except SystemExit:
        raise
    except Exception as exc:  # noqa: BLE001
        raise SystemExit(f"setup failed: {exc}") from exc
