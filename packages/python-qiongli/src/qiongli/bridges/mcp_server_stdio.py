from __future__ import annotations

import json
import sys
from typing import Any

from bridges.mcp_tool_handlers import MCP_TOOL_DEFINITIONS, SERVER_NAME, call_qiongli_tool
from qiongli import __version__


PROTOCOL_VERSION = "2024-11-05"


def handle_message(message: dict[str, Any]) -> dict[str, Any] | None:
    if message.get("jsonrpc") != "2.0":
        return _error(message.get("id"), -32600, "Invalid Request")

    method = str(message.get("method", "") or "")
    if method.startswith("notifications/"):
        return None

    request_id = message.get("id")
    params = message.get("params", {})
    if not isinstance(params, dict):
        return _error(request_id, -32602, "Invalid params")

    if method == "initialize":
        return _result(
            request_id,
            {
                "protocolVersion": str(params.get("protocolVersion") or PROTOCOL_VERSION),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": SERVER_NAME, "version": __version__},
            },
        )
    if method == "ping":
        return _result(request_id, {})
    if method == "tools/list":
        return _result(request_id, {"tools": MCP_TOOL_DEFINITIONS})
    if method == "tools/call":
        name = str(params.get("name", "") or "")
        arguments = params.get("arguments", {})
        if not isinstance(arguments, dict):
            return _error(request_id, -32602, "arguments must be an object")
        return _result(request_id, call_qiongli_tool(name, arguments))

    return _error(request_id, -32601, f"Method not found: {method}")


def run_stdio() -> int:
    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            response = _error(None, -32700, "Parse error")
        else:
            response = handle_message(message) if isinstance(message, dict) else _error(None, -32600, "Invalid Request")
        if response is None:
            continue
        sys.stdout.write(json.dumps(response, ensure_ascii=False, separators=(",", ":")) + "\n")
        sys.stdout.flush()
    return 0


def _result(request_id: Any, result: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def _error(request_id: Any, code: int, message: str) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {"code": code, "message": message},
    }


def main() -> int:
    return run_stdio()


if __name__ == "__main__":
    raise SystemExit(main())
