#!/usr/bin/env python3
"""
Test server for curl-gui. Zero dependencies — uses only stdlib.
Echoes back the exact request so you can verify headers, body, and method.

Usage:
    python test_server.py
    # Or in background:
    python test_server.py &

Then in curl-gUI, hit:
    http://localhost:8765/echo
"""

import json
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler


class EchoHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Quieter logs
        print(f"  → {self.command} {self.path}")

    def _send_json(self, data, status=200):
        body = json.dumps(data, indent=2).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("X-Echo-Server", "curl-gui-test")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _echo(self):
        # Read body
        content_length = self.headers.get("Content-Length")
        raw_body = b""
        if content_length:
            try:
                raw_body = self.rfile.read(int(content_length))
            except Exception:
                pass

        body_str = raw_body.decode("utf-8", errors="replace")

        # Try JSON parse
        body_parsed = body_str
        body_parsed_as = "raw"
        try:
            body_parsed = json.loads(body_str)
            body_parsed_as = "json"
        except Exception:
            pass

        # Parse query string
        from urllib.parse import parse_qs, urlparse
        parsed = urlparse(self.path)
        query_params = {k: v[0] if len(v) == 1 else v for k, v in parse_qs(parsed.query).items()}

        response = {
            "method": self.command,
            "path": parsed.path,
            "query_params": query_params,
            "headers": dict(self.headers),
            "content_type": self.headers.get("Content-Type"),
            "body_length": len(raw_body),
            "body": body_parsed,
            "body_parsed_as": body_parsed_as,
        }

        self._send_json(response)

    def do_GET(self):
        if self.path.startswith("/health"):
            self._send_json({"status": "ok"})
        else:
            self._echo()

    def do_POST(self):
        self._echo()

    def do_PUT(self):
        self._echo()

    def do_PATCH(self):
        self._echo()

    def do_DELETE(self):
        self._echo()

    def do_HEAD(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("X-Echo-Server", "curl-gui-test")
        self.end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Allow", "GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS")
        self.end_headers()


if __name__ == "__main__":
    port = 8765
    if len(sys.argv) > 1:
        port = int(sys.argv[1])

    server = HTTPServer(("0.0.0.0", port), EchoHandler)
    print("=" * 55)
    print(f"  curl-gui test server running on http://localhost:{port}")
    print(f"  Endpoints:")
    print(f"    GET  /health     → health check")
    print(f"    ANY  /echo       → echo back request")
    print(f"    ANY  /           → echo back request")
    print(f"  Ctrl+C to stop")
    print("=" * 55)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n  Shutting down.")
        server.shutdown()
