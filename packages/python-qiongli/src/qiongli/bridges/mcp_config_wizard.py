from __future__ import annotations

import html
import secrets
import sys
import threading
import webbrowser
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, TextIO
from urllib.parse import parse_qs, urlparse

from bridges.provider_config import PROVIDER_FIELDS, global_provider_config_path, set_provider_value


DEFAULT_HOST = "127.0.0.1"
ALLOWED_HOSTS = {"127.0.0.1", "localhost"}


PROVIDER_ACCESS_GUIDANCE: dict[str, dict[str, object]] = {
    "openalex": {
        "title": "OpenAlex API key",
        "config_field": "openalex.api_key",
        "apply_url": "https://openalex.org/settings/api",
        "docs_url": "https://developers.openalex.org/api-reference/authentication",
        "summary": (
            "OpenAlex requires a free API key for API calls at scale. Email is optional contact metadata."
        ),
        "steps": (
            "Sign in to OpenAlex and open the API settings page.",
            "Copy the free API key from the OpenAlex settings page.",
            "Paste the key into openalex.api_key; optionally add openalex.email.",
        ),
    },
    "semantic_scholar": {
        "title": "Semantic Scholar API key",
        "config_field": "semantic_scholar.api_key",
        "apply_url": "https://www.semanticscholar.org/product/api",
        "docs_url": "https://api.semanticscholar.org/api-docs/",
        "summary": "Semantic Scholar sends private API keys by email and recommends using a key.",
        "steps": (
            "Open the Semantic Scholar API page.",
            "Use the Request an API key section.",
            "Paste only the private key you receive by email into this local setup page.",
        ),
    },
    "crossref": {
        "title": "Crossref polite access",
        "config_field": "crossref.email",
        "apply_url": "",
        "docs_url": "https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication/",
        "summary": "Crossref public REST API access does not require signup; provide an email for polite access.",
        "steps": (
            "No public API key application is required for polite access.",
            "Use an email address you monitor.",
            "Use Metadata Plus only if you separately subscribe to Crossref's premium API-key service.",
        ),
    },
    "pubmed": {
        "title": "NCBI API key",
        "config_field": "pubmed.api_key",
        "apply_url": "https://support.nlm.nih.gov/kbArticle/?pn=KA-05317",
        "docs_url": "https://www.ncbi.nlm.nih.gov/books/NBK25501/",
        "summary": "NCBI API keys are generated from an NCBI account and can increase E-Utilities limits.",
        "steps": (
            "Sign in to NCBI.",
            "Open Account settings from your username menu.",
            "Create a key in the API Key Management section and paste it here.",
        ),
    },
    "arxiv": {
        "title": "arXiv",
        "config_field": "arxiv",
        "apply_url": "",
        "docs_url": "https://info.arxiv.org/help/api/index.html",
        "summary": "arXiv does not require an API key for the public API used by Qiongli.",
        "steps": (
            "No credential setup is needed.",
            "Qiongli can use arXiv search after the literature MCP runtime is installed.",
        ),
    },
}


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


@dataclass(frozen=True)
class ConfigWizardRunResult:
    status: str
    url: str
    config_path: str


def run_config_wizard(
    *,
    host: str = DEFAULT_HOST,
    port: int = 0,
    provider: str | None = None,
    output: TextIO = sys.stdout,
    open_browser: bool = True,
) -> ConfigWizardRunResult:
    wizard = start_config_wizard(host=host, port=port, provider=provider)
    print("Qiongli Literature Provider Setup", file=output)
    print(f"  url: {wizard.url}", file=output)
    print(f"  config: {wizard.config_path}", file=output)
    print("  waiting: save the page to continue", file=output)
    if open_browser:
        try:
            webbrowser.open(wizard.url)
        except Exception:  # noqa: BLE001 - browser launch is best-effort.
            print("  browser: unable to open automatically; use the URL above", file=output)
    try:
        wizard.completed.wait()
    except KeyboardInterrupt:
        wizard.stop()
        raise
    return ConfigWizardRunResult(status="saved", url=wizard.url, config_path=wizard.config_path)


def start_config_wizard(
    host: str = DEFAULT_HOST,
    port: int = 0,
    provider: str | None = None,
) -> ConfigWizard:
    host = _normalize_host(host)
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


def _normalize_host(host: str) -> str:
    normalized = str(host or DEFAULT_HOST).strip().lower()
    if normalized not in ALLOWED_HOSTS:
        raise ValueError("host must be 127.0.0.1 or localhost")
    return DEFAULT_HOST if normalized == "localhost" else normalized


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
            input_id = f"field-{provider}-{field}"
            preview_id = f"preview-{provider}-{field}"
            field_rows.append(
                "<div class=\"field-row\">"
                "<div>"
                f"<label for=\"{html.escape(input_id)}\">{html.escape(provider)} / {html.escape(field)}</label>"
                f"<p class=\"hint\">{html.escape(_field_help(provider, field))}</p>"
                "</div>"
                "<div>"
                "<div class=\"input-row\">"
                f"<input id=\"{html.escape(input_id)}\" name=\"{html.escape(input_name)}\" "
                f"type=\"password\" autocomplete=\"off\" spellcheck=\"false\" "
                f"aria-describedby=\"{html.escape(preview_id)}\" data-secret-input=\"{html.escape(input_name)}\" />"
                f"<button type=\"button\" data-toggle-for=\"{html.escape(input_name)}\" "
                f"aria-controls=\"{html.escape(input_id)}\" aria-pressed=\"false\">Show</button>"
                "</div>"
                f"<p class=\"preview\" id=\"{html.escape(preview_id)}\">Preview: "
                f"<code data-preview-for=\"{html.escape(input_name)}\" class=\"is-empty\">empty</code></p>"
                "</div>"
                "</div>"
            )
        if not field_rows:
            field_rows.append("<p class=\"hint\">No API key is required for this provider.</p>")
        fields.append(
            "<fieldset>"
            f"<legend>{html.escape(provider)}</legend>"
            + "".join(field_rows)
            + "</fieldset>"
        )
    saved_html = "<p class=\"saved\">Saved.</p>" if saved else ""
    guidance = _render_access_guidance()
    return (
        "<!doctype html>"
        "<html><head><meta charset=\"utf-8\" />"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />"
        "<title>Qiongli MCP Provider Configuration</title>"
        "<style>"
        "body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:32px;max-width:900px;color:#24292f}"
        "h1{font-size:28px;margin:0 0 8px}h2{font-size:16px;margin:0 0 8px}p{line-height:1.45}"
        ".intro{color:#57606a;margin:0 0 20px}"
        ".notice{background:#f6f8fa;border:1px solid #d0d7de;border-radius:8px;margin:20px 0;padding:16px}"
        ".notice ul{margin:8px 0 0;padding-left:22px}.notice li{margin:6px 0}"
        ".guidance-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin:20px 0}"
        ".guidance-card{border:1px solid #d0d7de;border-radius:8px;padding:14px}"
        ".guidance-card h3{font-size:15px;margin:0 0 6px}.guidance-card p,.guidance-card ol{font-size:13px}"
        ".guidance-card ol{margin:8px 0 0;padding-left:20px}.guidance-card li{margin:5px 0}"
        ".resource-links{display:flex;flex-wrap:wrap;gap:8px}"
        "fieldset{border:1px solid #d0d7de;border-radius:8px;margin:0 0 16px;padding:16px}"
        "legend{font-weight:700;padding:0 6px}"
        ".field-row{display:grid;grid-template-columns:240px 1fr;gap:16px;align-items:start;margin:14px 0}"
        "label{display:block;font-weight:600;margin:2px 0 4px}.hint{color:#57606a;font-size:13px;margin:0}"
        ".input-row{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:8px}"
        "input{font:inherit;padding:8px;border:1px solid #8c959f;border-radius:6px}"
        "input:focus{border-color:#0969da;box-shadow:0 0 0 3px rgba(9,105,218,.15);outline:none}"
        "button{font:inherit;padding:8px 12px;border:1px solid #57606a;border-radius:6px;background:#f6f8fa}"
        "button[disabled]{color:#6e7781;border-color:#d0d7de}.saved{color:#116329;font-weight:600}"
        ".path{color:#57606a;font-size:13px}"
        ".preview{color:#57606a;font-size:13px;margin:6px 0 0}"
        ".preview code,code{background:#f6f8fa;border:1px solid #d0d7de;border-radius:6px;color:#24292f;padding:2px 6px;word-break:break-all}"
        ".preview code.is-empty{color:#6e7781}"
        "@media(max-width:760px){body{margin:20px}.guidance-grid{grid-template-columns:1fr}.field-row{grid-template-columns:1fr}.input-row{grid-template-columns:1fr}}"
        "</style></head><body>"
        "<h1>Qiongli MCP Provider Configuration</h1>"
        "<p class=\"intro\">Use this local setup page for Qiongli provider credentials across Codex, Claude Code, Antigravity, Claude Desktop, and other MCP clients.</p>"
        "<section class=\"notice\" aria-label=\"Security notes\">"
        "<h2>Before you save</h2>"
        "<ul>"
        "<li><strong>Keys stay on this machine.</strong> Qiongli writes only to the local shared provider config.</li>"
        "<li><strong>Do not paste API keys into chat.</strong> Configure them here so secrets do not enter conversation history.</li>"
        "<li>After saving, run <code>qiongli provider doctor</code> or <code>qiongli_config_status</code> to confirm redacted status.</li>"
        "</ul>"
        "</section>"
        f"{guidance}"
        f"<p class=\"path\">Config path: {html.escape(str(Path(config_path)))}</p>"
        f"{saved_html}"
        f"<form method=\"post\" action=\"/save?token={html.escape(token)}\">"
        + "".join(fields)
        + "<button type=\"submit\">Save</button>"
        "</form>"
        "<script>"
        "(() => {"
        "const find=(s,a,v)=>Array.from(document.querySelectorAll(s)).find(n=>n.getAttribute(a)===v);"
        "const mask=(v)=>!v?'empty':(v.length<=8?'*'.repeat(v.length):`${v.slice(0,4)}...${v.slice(-4)}`);"
        "document.querySelectorAll('[data-secret-input]').forEach((input)=>{"
        "const name=input.getAttribute('data-secret-input');"
        "const preview=find('[data-preview-for]','data-preview-for',name);"
        "const toggle=find('[data-toggle-for]','data-toggle-for',name);"
        "let revealed=false;"
        "const sync=()=>{if(!preview)return;preview.textContent=revealed?(input.value||'empty'):mask(input.value);preview.classList.toggle('is-empty',!input.value);};"
        "input.addEventListener('input',sync);"
        "toggle?.addEventListener('click',()=>{revealed=!revealed;input.type=revealed?'text':'password';toggle.textContent=revealed?'Hide':'Show';toggle.setAttribute('aria-pressed',revealed?'true':'false');sync();input.focus();});"
        "sync();"
        "});"
        "document.querySelector('form')?.addEventListener('submit',(event)=>{const form=event.currentTarget;const submit=form.querySelector('button[type=\"submit\"]');form.setAttribute('aria-busy','true');if(submit){submit.textContent='Saving...';submit.disabled=true;}});"
        "})();"
        "</script>"
        "</body></html>"
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


def _render_access_guidance() -> str:
    cards = []
    for entry in PROVIDER_ACCESS_GUIDANCE.values():
        apply_url = str(entry.get("apply_url") or "")
        docs_url = str(entry.get("docs_url") or "")
        links = []
        if apply_url:
            links.append(f"<a href=\"{html.escape(apply_url)}\" target=\"_blank\" rel=\"noreferrer\">Apply or configure</a>")
        if docs_url:
            links.append(f"<a href=\"{html.escape(docs_url)}\" target=\"_blank\" rel=\"noreferrer\">Official docs</a>")
        steps = "".join(f"<li>{html.escape(str(step))}</li>" for step in entry.get("steps", ()))
        cards.append(
            "<article class=\"guidance-card\">"
            f"<h3>{html.escape(str(entry['title']))}</h3>"
            f"<p>{html.escape(str(entry['summary']))}</p>"
            f"<p>Config field: <code>{html.escape(str(entry['config_field']))}</code></p>"
            f"<ol>{steps}</ol>"
            + (f"<p class=\"resource-links\">{''.join(links)}</p>" if links else "")
            + "</article>"
        )
    return (
        "<section aria-label=\"Provider access guidance\">"
        "<h2>How to get provider access</h2>"
        "<p class=\"intro\">Use the links below to request or prepare provider credentials before saving them here.</p>"
        f"<div class=\"guidance-grid\">{''.join(cards)}</div>"
        "</section>"
    )


def _field_help(provider: str, field: str) -> str:
    if provider == "openalex" and field == "api_key":
        return "OpenAlex API key from openalex.org/settings/api. Stored only in local Qiongli provider config."
    if provider == "openalex" and field == "email":
        return "Optional contact email included as mailto for OpenAlex requests."
    if provider == "semantic_scholar" and field == "api_key":
        return "Semantic Scholar API key received by email and used only by the local MCP server."
    if provider == "crossref" and field == "email":
        return "Email for Crossref polite access; no public API key is required."
    if provider == "pubmed" and field == "api_key":
        return "NCBI API key generated from your NCBI account."
    if field == "email":
        return "Provider contact email for rate limits or attribution."
    return "Provider credential used only by the local MCP server."


def _shutdown_server_once(server: ThreadingHTTPServer, completed: threading.Event) -> None:
    if completed.is_set():
        return
    try:
        server.shutdown()
    finally:
        server.server_close()
        completed.set()
