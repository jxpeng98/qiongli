"""
Antigravity CLI Bridge for qiongli.
Wraps Antigravity CLI as a non-interactive runtime collaborator.

Python 3.12+ required.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .base_bridge import BaseBridge, BridgeResponse, ModelType, escape_prompt


class AntigravityBridge(BaseBridge):
    """Bridge to Antigravity CLI for review, verification, and triad audit tasks."""

    model_type = ModelType.ANTIGRAVITY

    def __init__(
        self,
        model: str | None = None,
        sandbox: bool = True,
        dangerously_skip_permissions: bool = False,
        print_timeout: str | None = None,
        log_file: str | None = None,
    ):
        self.model = model
        self.sandbox = sandbox
        self.dangerously_skip_permissions = dangerously_skip_permissions
        self.print_timeout = print_timeout
        self.log_file = log_file

    def build_command(
        self,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        conversation_id: str | None = None,
        continue_last: bool = False,
        add_dirs: list[str] | None = None,
        **kwargs,
    ) -> list[str]:
        model = str(kwargs.get("model") or self.model or "").strip()
        sandbox = bool(kwargs.get("sandbox", self.sandbox))
        dangerously_skip_permissions = bool(
            kwargs.get(
                "dangerously_skip_permissions",
                self.dangerously_skip_permissions,
            )
        )
        print_timeout = str(
            kwargs.get("print_timeout") or self.print_timeout or ""
        ).strip()
        log_file = str(kwargs.get("log_file") or self.log_file or "").strip()
        extra_add_dirs = kwargs.get("add_dirs")
        all_add_dirs = list(add_dirs or [])
        if isinstance(extra_add_dirs, list):
            all_add_dirs.extend(str(item) for item in extra_add_dirs)

        cmd = ["antigravity", "--print"]
        if model:
            cmd.extend(["--model", model])
        if sandbox:
            cmd.append("--sandbox")
        if dangerously_skip_permissions:
            cmd.append("--dangerously-skip-permissions")
        if print_timeout:
            cmd.extend(["--print-timeout", print_timeout])
        if log_file:
            cmd.extend(["--log-file", log_file])
        for path in all_add_dirs:
            if str(path).strip():
                cmd.extend(["--add-dir", str(path)])

        resume_id = conversation_id or session_id
        if continue_last:
            cmd.append("--continue")
        elif resume_id:
            cmd.extend(["--conversation", resume_id])

        cmd.append(escape_prompt(prompt))
        return cmd

    def parse_output(self, lines: list[str]) -> BridgeResponse:
        all_messages: list[dict[str, Any]] = []
        plain_lines: list[str] = []
        assistant_parts: list[str] = []
        session_id: str | None = None
        errors: list[str] = []

        for line in lines:
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                plain_lines.append(line)
                continue
            except Exception as exc:
                errors.append(f"Parse error: {exc}")
                continue

            if not isinstance(data, dict):
                plain_lines.append(line)
                continue

            all_messages.append(data)
            session_id = session_id or self._extract_session_id(data)
            assistant_parts.extend(self._extract_assistant_messages(data))
            error_text = self._extract_error_text(data)
            if error_text:
                errors.append(error_text)

        content = "".join(assistant_parts).strip()
        if not content and plain_lines:
            content = "\n".join(item for item in plain_lines if item.strip()).strip()

        if not content:
            return BridgeResponse(
                success=False,
                model="antigravity",
                session_id=session_id,
                error=(
                    "; ".join(errors)
                    if errors
                    else "No assistant text received from Antigravity CLI."
                ),
                raw_messages=all_messages or None,
            )

        return BridgeResponse(
            success=True,
            model="antigravity",
            session_id=session_id,
            content=content,
            error="; ".join(errors) if errors else None,
            raw_messages=all_messages or None,
        )

    def _extract_session_id(self, data: dict[str, Any]) -> str | None:
        for key in (
            "conversation_id",
            "conversationId",
            "session_id",
            "sessionId",
            "thread_id",
            "threadId",
        ):
            value = data.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
        for key in ("conversation", "session", "thread"):
            nested = data.get(key)
            if isinstance(nested, dict):
                found = self._extract_session_id(nested)
                if found:
                    return found
        return None

    def _extract_assistant_messages(self, data: dict[str, Any]) -> list[str]:
        texts: list[str] = []
        role = str(data.get("role", "")).strip().lower()
        msg_type = str(data.get("type", "")).strip().lower()
        message = data.get("message")
        is_assistant = role == "assistant" or msg_type in {
            "assistant",
            "assistant_message",
            "assistant-response",
        }

        if isinstance(message, dict):
            msg_role = str(message.get("role", "")).strip().lower()
            if msg_role == "assistant":
                is_assistant = True

        if is_assistant:
            for key in ("content", "text", "delta", "output_text"):
                texts.extend(self._extract_text(data.get(key)))
            if isinstance(message, dict):
                for key in ("content", "text", "delta", "output_text"):
                    texts.extend(self._extract_text(message.get(key)))
        return texts

    def _extract_text(self, value: Any) -> list[str]:
        if value is None:
            return []
        if isinstance(value, str):
            return [value]
        if isinstance(value, list):
            out: list[str] = []
            for item in value:
                out.extend(self._extract_text(item))
            return out
        if isinstance(value, dict):
            out: list[str] = []
            if isinstance(value.get("text"), str):
                out.append(value["text"])
            for key in ("content", "delta", "message", "output_text"):
                out.extend(self._extract_text(value.get(key)))
            return out
        return []

    def _extract_error_text(self, data: dict[str, Any]) -> str:
        error = data.get("error")
        if isinstance(error, str) and error.strip():
            return error.strip()
        if isinstance(error, dict):
            message = error.get("message")
            if isinstance(message, str) and message.strip():
                return message.strip()
        msg_type = str(data.get("type", "")).strip().lower()
        if "error" in msg_type:
            message = data.get("message")
            if isinstance(message, str) and message.strip():
                return message.strip()
        return ""
