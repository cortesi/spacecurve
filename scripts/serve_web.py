#!/usr/bin/env python3
"""Launch the wasm dev server for scurve-web using wasm-server-runner."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX_HTML = ROOT / "crates" / "scurve-gui" / "assets" / "index.html"


def run_command(command: list[str], env: dict[str, str]) -> None:
    print(f"$ {' '.join(command)}")
    subprocess.run(command, check=True, cwd=ROOT, env=env)


def main() -> None:
    env = os.environ.copy()
    env["WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML"] = str(INDEX_HTML)

    run_command(
        [
            "cargo",
            "run",
            "--target",
            "wasm32-unknown-unknown",
            "--bin",
            "scurve-web",
        ],
        env,
    )


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode) from exc
    except SystemExit:
        raise
    except Exception as exc:  # noqa: BLE001
        raise SystemExit(f"serve failed: {exc}") from exc
