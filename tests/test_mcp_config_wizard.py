from __future__ import annotations

import os
import tempfile
import unittest
import urllib.parse
import urllib.request
from pathlib import Path
from unittest import mock

from bridges.mcp_config_wizard import start_config_wizard
from bridges.provider_config import resolve_provider_config


class MCPConfigWizardTests(unittest.TestCase):
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
