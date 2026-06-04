from __future__ import annotations

import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

from bridges.mcp_server_stdio import handle_message
from bridges.mcp_tool_handlers import SERVER_NAME


class MCPHTTPHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        if self.path.rstrip("/") == "/health":
            self._send_json({"ok": True, "server": SERVER_NAME})
            return
        self._send_json({"error": "not found"}, status=404)

    def do_POST(self) -> None:
        if self.path.rstrip("/") not in {"", "/mcp"}:
            self._send_json({"error": "not found"}, status=404)
            return
        length = int(self.headers.get("Content-Length", "0") or "0")
        raw_body = self.rfile.read(length).decode("utf-8")
        try:
            message = json.loads(raw_body)
        except json.JSONDecodeError:
            self._send_json(
                {"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": "Parse error"}},
                status=400,
            )
            return
        if not isinstance(message, dict):
            self._send_json(
                {"jsonrpc": "2.0", "id": None, "error": {"code": -32600, "message": "Invalid Request"}},
                status=400,
            )
            return
        response = handle_message(message)
        if response is None:
            self._send_json({}, status=202)
            return
        self._send_json(response)

    def log_message(self, _format: str, *_args: Any) -> None:
        return

    def _send_json(self, payload: dict[str, Any], *, status: int = 200) -> None:
        body = json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def run_http_server(host: str = "127.0.0.1", port: int = 8765) -> int:
    server = ThreadingHTTPServer((host, port), MCPHTTPHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        return 0
    finally:
        server.server_close()
    return 0
