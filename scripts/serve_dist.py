#!/usr/bin/env python3
"""Serve the built web bundle from ./dist using Python's HTTP server."""

from __future__ import annotations

import argparse
import sys
from contextlib import suppress
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIST_DIR = ROOT / "dist"
HOST = "127.0.0.1"
DEFAULT_PORT = 8000


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Serve the ./dist directory on http://127.0.0.1:<port>."
    )
    parser.add_argument(
        "port",
        nargs="?",
        type=int,
        default=DEFAULT_PORT,
        help=f"Port to bind (default: {DEFAULT_PORT})",
    )
    return parser.parse_args()


class DistRequestHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(DIST_DIR), **kwargs)

    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        sys.stdout.write(f"[{self.address_string()}] {format % args}\n")
        sys.stdout.flush()


def ensure_dist_ready() -> None:
    index_html = DIST_DIR / "index.html"
    if not index_html.is_file():
        message = "dist/index.html not found. Run uv run ./scripts/build_web.py first."
        print(message, file=sys.stderr)
        raise SystemExit(1)


def open_server(port: int) -> ThreadingHTTPServer:
    class ReusableTCPServer(ThreadingHTTPServer):
        allow_reuse_address = True

    try:
        server = ReusableTCPServer((HOST, port), DistRequestHandler)
    except OSError as exc:
        raise SystemExit(f"Failed to bind to {HOST}:{port}: {exc}") from exc
    return server


def main() -> None:
    args = parse_args()
    ensure_dist_ready()

    server = open_server(args.port)
    address = server.server_address
    url = f"http://{address[0]}:{address[1]}"
    print(f"Serving dist/ on {url} (Ctrl+C to stop)")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping server...")
    finally:
        with suppress(Exception):
            server.server_close()


if __name__ == "__main__":
    main()
