from __future__ import annotations

import os
import tempfile
import unittest
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from unittest import mock

from bridges.mcp_config_wizard import start_config_wizard
from bridges.provider_config import resolve_provider_config


class MCPConfigWizardTests(unittest.TestCase):
    def test_wizard_rejects_invalid_lifetime_and_body_limits(self) -> None:
        with self.assertRaisesRegex(ValueError, "TTL must be greater than zero"):
            start_config_wizard(ttl_seconds=0)
        with self.assertRaisesRegex(ValueError, "body limit must be between"):
            start_config_wizard(max_body_bytes=0)
        with self.assertRaisesRegex(ValueError, "body limit must be between"):
            start_config_wizard(max_body_bytes=64 * 1024 + 1)

    def test_wizard_expires_without_a_submission(self) -> None:
        try:
            wizard = start_config_wizard(port=0, ttl_seconds=0.05)
        except PermissionError as exc:
            self.skipTest(f"local port binding unavailable: {exc}")
        try:
            self.assertTrue(wizard.completed.wait(timeout=2))
        finally:
            wizard.stop()

    def test_wizard_rejects_large_requests_without_echoing_secrets(self) -> None:
        secret = "wizard-secret-value-that-must-not-be-echoed"
        try:
            wizard = start_config_wizard(port=0, ttl_seconds=5, max_body_bytes=24)
        except PermissionError as exc:
            self.skipTest(f"local port binding unavailable: {exc}")
        try:
            body = urllib.parse.urlencode({"openalex.api_key": secret}).encode("utf-8")
            oversized_body = urllib.request.Request(
                f"http://{wizard.host}:{wizard.port}/save?token={wizard.token}",
                data=body,
                method="POST",
            )
            with self.assertRaises(urllib.error.HTTPError) as body_error:
                urllib.request.urlopen(oversized_body, timeout=5)
            body_response = body_error.exception.read().decode("utf-8")
            self.assertEqual(body_error.exception.code, 413)
            self.assertNotIn(secret, body_response)
            self.assertEqual(body_error.exception.headers["Cache-Control"], "no-store")

            header_secret = "oversized-header-secret"
            oversized_headers = urllib.request.Request(
                wizard.url,
                headers={"X-Oversized": header_secret * 400},
            )
            with self.assertRaises(urllib.error.HTTPError) as header_error:
                urllib.request.urlopen(oversized_headers, timeout=5)
            self.assertEqual(header_error.exception.code, 431)
            header_response = header_error.exception.read().decode("utf-8")
            self.assertNotIn(secret, header_response)
            self.assertNotIn(header_secret, header_response)
        finally:
            wizard.stop()

    def test_successful_submission_is_single_use_and_secret_free(self) -> None:
        secret = "single-use-wizard-secret"
        repeated_secret = "repeated-wizard-secret"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                try:
                    wizard = start_config_wizard(port=0, ttl_seconds=5)
                except PermissionError as exc:
                    self.skipTest(f"local port binding unavailable: {exc}")
                try:
                    body = urllib.parse.urlencode({"openalex.api_key": secret}).encode("utf-8")
                    target = f"http://{wizard.host}:{wizard.port}/save?token={wizard.token}"
                    opener = urllib.request.build_opener(NoRedirectHandler)

                    with self.assertRaises(urllib.error.HTTPError) as unauthorized_error:
                        urllib.request.urlopen(
                            f"http://{wizard.host}:{wizard.port}/",
                            timeout=5,
                        )
                    self.assertEqual(unauthorized_error.exception.code, 403)
                    self.assertNotIn(secret, unauthorized_error.exception.read().decode("utf-8"))

                    unsupported_secret = "unsupported-provider-secret"
                    with self.assertRaises(urllib.error.HTTPError) as unsupported_error:
                        opener.open(
                            urllib.request.Request(
                                target,
                                data=urllib.parse.urlencode(
                                    {"unknown.api_key": unsupported_secret}
                                ).encode("utf-8"),
                                method="POST",
                            ),
                            timeout=5,
                        )
                    self.assertEqual(unsupported_error.exception.code, 400)
                    self.assertNotIn(
                        unsupported_secret,
                        unsupported_error.exception.read().decode("utf-8"),
                    )

                    first = opener.open(
                        urllib.request.Request(target, data=body, method="POST"),
                        timeout=5,
                    )
                    first_body = first.read().decode("utf-8")
                    self.assertEqual(first.status, 303)
                    self.assertNotIn(secret, first_body)
                    self.assertEqual(first.headers["Cache-Control"], "no-store")
                    self.assertEqual(first.headers["X-Content-Type-Options"], "nosniff")

                    with self.assertRaises(urllib.error.HTTPError) as repeated_error:
                        opener.open(
                            urllib.request.Request(
                                target,
                                data=urllib.parse.urlencode(
                                    {"openalex.api_key": repeated_secret}
                                ).encode("utf-8"),
                                method="POST",
                            ),
                            timeout=5,
                        )
                    repeated_body = repeated_error.exception.read().decode("utf-8")
                    self.assertEqual(repeated_error.exception.code, 410)
                    self.assertNotIn(secret, repeated_body)
                    self.assertNotIn(repeated_secret, repeated_body)
                    config = resolve_provider_config(cwd=root, env={})
                    self.assertEqual(config["providers"]["openalex"]["api_key"], secret)
                finally:
                    wizard.stop()

    def test_malformed_config_remains_byte_identical_after_failed_submission(self) -> None:
        secret = "malformed-wizard-secret-canary"
        submitted_secret = "submitted-wizard-secret-canary"
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config-path-canary"
            config_home.mkdir()
            config_path = config_home / "providers.json"
            original = f"{{not-json {secret}".encode()
            config_path.write_bytes(original)
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(config_home)},
                clear=False,
            ):
                try:
                    wizard = start_config_wizard(port=0, ttl_seconds=5)
                except PermissionError as exc:
                    self.skipTest(f"local port binding unavailable: {exc}")
                try:
                    target = f"http://{wizard.host}:{wizard.port}/save?token={wizard.token}"
                    request = urllib.request.Request(
                        target,
                        data=urllib.parse.urlencode(
                            {"openalex.api_key": submitted_secret}
                        ).encode("utf-8"),
                        method="POST",
                    )
                    with self.assertRaises(urllib.error.HTTPError) as save_error:
                        urllib.request.urlopen(request, timeout=5)
                    response = save_error.exception.read().decode("utf-8")
                    self.assertEqual(save_error.exception.code, 500)
                    self.assertEqual(response, "Unable to save configuration.")
                    self.assertNotIn(secret, response)
                    self.assertNotIn(submitted_secret, response)
                    self.assertNotIn(str(config_home), response)
                    self.assertEqual(config_path.read_bytes(), original)
                finally:
                    wizard.stop()

    def test_wizard_post_saves_provider_value_to_shared_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                try:
                    wizard = start_config_wizard(port=0)
                except PermissionError as exc:
                    self.skipTest(f"local port binding unavailable: {exc}")
                try:
                    body = urllib.parse.urlencode(
                        {
                            "openalex.api_key": "openalex-secret-key",
                            "openalex.email": "user@example.com",
                        }
                    ).encode("utf-8")
                    request = urllib.request.Request(
                        f"http://{wizard.host}:{wizard.port}/save?token={wizard.token}",
                        data=body,
                        method="POST",
                    )
                    opener = urllib.request.build_opener(NoRedirectHandler)
                    response = opener.open(request, timeout=5)
                    self.assertEqual(response.status, 303)
                    self.assertTrue(wizard.completed.wait(timeout=5))

                    config = resolve_provider_config(cwd=root, env={})
                finally:
                    wizard.stop()

        self.assertEqual(config["providers"]["openalex"]["api_key"], "openalex-secret-key")
        self.assertEqual(config["providers"]["openalex"]["email"], "user@example.com")

    def test_wizard_saved_page_guides_user_to_close_and_completes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                try:
                    wizard = start_config_wizard(port=0)
                except PermissionError as exc:
                    self.skipTest(f"local port binding unavailable: {exc}")
                try:
                    body = urllib.parse.urlencode({"openalex.api_key": "openalex-secret-key"}).encode(
                        "utf-8"
                    )
                    request = urllib.request.Request(
                        f"http://{wizard.host}:{wizard.port}/save?token={wizard.token}",
                        data=body,
                        method="POST",
                    )
                    response = urllib.request.urlopen(request, timeout=5)
                    html = response.read().decode("utf-8")
                    self.assertIn("Saved", html)
                    self.assertIn("You can close this page", html)
                    self.assertTrue(wizard.completed.wait(timeout=5))
                finally:
                    wizard.stop()

    def test_wizard_page_includes_provider_access_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.dict(
                os.environ,
                {"QIONGLI_CONFIG_HOME": str(root / "config")},
                clear=False,
            ):
                try:
                    wizard = start_config_wizard(port=0)
                except PermissionError as exc:
                    self.skipTest(f"local port binding unavailable: {exc}")
                try:
                    response = urllib.request.urlopen(wizard.url, timeout=5)
                    rendered = response.read().decode("utf-8")
                    self.assertEqual(response.headers["Cache-Control"], "no-store")
                    self.assertEqual(response.headers["X-Content-Type-Options"], "nosniff")
                finally:
                    wizard.stop()

        self.assertIn("How to get provider access", rendered)
        self.assertIn("OpenAlex API key", rendered)
        self.assertIn("https://openalex.org/settings/api", rendered)
        self.assertIn("Semantic Scholar API key", rendered)
        self.assertIn("https://www.semanticscholar.org/product/api", rendered)
        self.assertIn("Crossref polite access", rendered)
        self.assertIn("https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication/", rendered)
        self.assertIn("NCBI API key", rendered)
        self.assertIn("https://support.nlm.nih.gov/kbArticle/?pn=KA-05317", rendered)
        self.assertIn("arXiv does not require an API key", rendered)
        self.assertIn("Do not paste API keys into chat", rendered)
        self.assertIn("data-preview-for=\"openalex.api_key\"", rendered)

    def test_wizard_rejects_non_local_hosts(self) -> None:
        with self.assertRaisesRegex(ValueError, "host must be 127.0.0.1 or localhost"):
            start_config_wizard(host="0.0.0.0", port=0)


class NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: ANN001
        return None

    def http_error_303(self, req, fp, code, msg, headers):  # noqa: ANN001
        return fp


if __name__ == "__main__":
    unittest.main()
