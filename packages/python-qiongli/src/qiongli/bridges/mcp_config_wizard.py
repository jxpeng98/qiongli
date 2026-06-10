from __future__ import annotations

import html
import secrets
import threading
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse

from bridges.provider_config import PROVIDER_FIELDS, global_provider_config_path, set_provider_value


@dataclass
class ConfigWizard:
    host: str
    port: int
    token: str
    config_path: str
    server: ThreadingHTTPServer
    thread: threading.Thread
    completed: threading.Event

    @property
    def url(self) -> str:
        return f"http://{self.host}:{self.port}/?token={self.token}"

    def stop(self) -> None:
        _shutdown_server_once(self.server, self.completed)


def start_config_wizard(
    host: str = "127.0.0.1",
    port: int = 0,
    provider: str | None = None,
) -> ConfigWizard:
    provider_id = _normalize_provider(provider)
    token = secrets.token_urlsafe(18)
    config_path = str(global_provider_config_path())
    completed = threading.Event()
    close_lock = threading.Lock()
    close_started = False
    close_timer: threading.Timer | None = None

    def schedule_shutdown(delay: float) -> None:
        nonlocal close_started, close_timer
        with close_lock:
            if close_started:
                return
            if close_timer is not None:
                close_timer.cancel()

        def shutdown() -> None:
            nonlocal close_started
            with close_lock:
                if close_started:
                    return
                close_started = True
            _shutdown_server_once(server, completed)

        timer = threading.Timer(delay, shutdown)
        timer.daemon = True
        close_timer = timer
        timer.start()

    class WizardHandler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            if not self._authorized():
                self._send_text("Forbidden", status=403)
                return
            if urlparse(self.path).path == "/saved":
                self._send_html(_render_saved_page(config_path=config_path))
                schedule_shutdown(0.1)
                return
            saved = "saved" in parse_qs(urlparse(self.path).query)
            self._send_html(
                _render_form(
                    token=token,
                    config_path=config_path,
                    saved=saved,
                    provider=provider_id,
                )
            )

        def do_POST(self) -> None:
            if not self._authorized():
                self._send_text("Forbidden", status=403)
                return
            length = int(self.headers.get("Content-Length", "0") or "0")
            body = self.rfile.read(length).decode("utf-8")
            values = parse_qs(body)
            for provider, fields in _provider_entries(provider_id):
                for field in fields:
                    value = values.get(f"{provider}.{field}", [""])[0].strip()
                    if value:
                        set_provider_value(provider, field, value)
            self.send_response(303)
            self.send_header("Location", f"/saved?token={token}")
            self.end_headers()
            schedule_shutdown(1.5)

        def log_message(self, _format: str, *_args: Any) -> None:
            return

        def _authorized(self) -> bool:
            query = parse_qs(urlparse(self.path).query)
            return query.get("token", [""])[0] == token

        def _send_html(self, body: str) -> None:
            payload = body.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def _send_text(self, body: str, *, status: int = 200) -> None:
            payload = body.encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

    server = ThreadingHTTPServer((host, port), WizardHandler)
    actual_host, actual_port = server.server_address
    thread = threading.Thread(target=server.serve_forever, name="qiongli-mcp-config", daemon=True)
    thread.start()
    return ConfigWizard(
        host=str(actual_host),
        port=int(actual_port),
        token=token,
        config_path=config_path,
        server=server,
        thread=thread,
        completed=completed,
    )


def _normalize_provider(provider: str | None) -> str | None:
    if not provider:
        return None
    normalized = provider.strip().lower().replace("-", "_")
    aliases = {"s2": "semantic_scholar", "semanticscholar": "semantic_scholar"}
    provider_id = aliases.get(normalized, normalized)
    if provider_id not in PROVIDER_FIELDS:
        raise ValueError(f"unsupported provider: {provider}")
    return provider_id


def _provider_entries(provider: str | None) -> list[tuple[str, object]]:
    if provider:
        return [(provider, PROVIDER_FIELDS[provider])]
    return list(PROVIDER_FIELDS.items())


def _render_form(*, token: str, config_path: str, saved: bool, provider: str | None = None) -> str:
    fields = []
    for provider, provider_fields in _provider_entries(provider):
        field_rows = []
        for field in provider_fields:
            input_name = f"{provider}.{field}"
            field_rows.append(
                "<label>"
                f"<span>{html.escape(provider)} / {html.escape(field)}</span>"
                f"<input name=\"{html.escape(input_name)}\" type=\"password\" autocomplete=\"off\" />"
                "</label>"
            )
        fields.append(
            "<fieldset>"
            f"<legend>{html.escape(provider)}</legend>"
            + "".join(field_rows)
            + "</fieldset>"
        )
    saved_html = "<p class=\"saved\">Saved.</p>" if saved else ""
    return (
        "<!doctype html>"
        "<html><head><meta charset=\"utf-8\" />"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />"
        "<title>Qiongli MCP Provider Configuration</title>"
        "<style>"
        "body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:32px;max-width:760px}"
        "fieldset{border:1px solid #d0d7de;border-radius:8px;margin:0 0 16px;padding:16px}"
        "label{display:grid;grid-template-columns:220px 1fr;gap:12px;align-items:center;margin:10px 0}"
        "input{font:inherit;padding:8px;border:1px solid #8c959f;border-radius:6px}"
        "button{font:inherit;padding:8px 12px;border:1px solid #57606a;border-radius:6px;background:#f6f8fa}"
        ".saved{color:#116329;font-weight:600}"
        ".path{color:#57606a;font-size:13px}"
        "@media(max-width:640px){label{grid-template-columns:1fr}}"
        "</style></head><body>"
        "<h1>Qiongli MCP Provider Configuration</h1>"
        f"<p class=\"path\">Config path: {html.escape(str(Path(config_path)))}</p>"
        f"{saved_html}"
        f"<form method=\"post\" action=\"/save?token={html.escape(token)}\">"
        + "".join(fields)
        + "<button type=\"submit\">Save</button>"
        "</form></body></html>"
    )


def _render_saved_page(*, config_path: str) -> str:
    return (
        "<!doctype html>"
        "<html><head><meta charset=\"utf-8\" />"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />"
        "<title>Qiongli MCP Provider Configuration Saved</title>"
        "<style>"
        "body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:32px;max-width:720px}"
        ".saved{color:#116329;font-weight:700}"
        ".path{color:#57606a;font-size:13px}"
        "code{background:#f6f8fa;border:1px solid #d0d7de;border-radius:6px;padding:2px 6px;word-break:break-all}"
        "</style></head><body>"
        "<h1>Saved</h1>"
        "<p class=\"saved\">Provider configuration was saved locally.</p>"
        "<p>You can close this page. If this was opened from the CLI, the waiting process will continue automatically.</p>"
        f"<p class=\"path\">Config path: <code>{html.escape(str(Path(config_path)))}</code></p>"
        "<script>setTimeout(function(){ try { window.close(); } catch (_) {} }, 1200);</script>"
        "</body></html>"
    )


def _shutdown_server_once(server: ThreadingHTTPServer, completed: threading.Event) -> None:
    if completed.is_set():
        return
    try:
        server.shutdown()
    finally:
        server.server_close()
        completed.set()
