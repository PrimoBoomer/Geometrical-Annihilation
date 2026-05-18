"""Tiny static HTTP server for the Godot Web export.

Serves client/export/web/ on 127.0.0.1:8081 with the cross-origin isolation
headers required by Godot 4.x threaded web builds (SharedArrayBuffer).
"""

import http.server
import os
import socketserver
import sys

HOST = "127.0.0.1"
PORT = 8081
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SERVE_DIR = os.path.join(REPO_ROOT, "client", "export", "web")


class CrossOriginIsolatedHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=SERVE_DIR, **kwargs)

    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Resource-Policy", "same-origin")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


def main() -> int:
    if not os.path.isdir(SERVE_DIR):
        print(f"[serve-web] missing directory: {SERVE_DIR}", file=sys.stderr)
        print("[serve-web] run scripts/run-all.ps1 first to export the client.", file=sys.stderr)
        return 1
    with socketserver.TCPServer((HOST, PORT), CrossOriginIsolatedHandler) as httpd:
        print(f"[serve-web] serving {SERVE_DIR} on http://{HOST}:{PORT}/", flush=True)
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
