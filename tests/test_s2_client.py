from __future__ import annotations

import io
import json
import os
import tempfile
import unittest
import urllib.error
import urllib.parse
import warnings
from unittest import mock

from bridges.provider_config import set_provider_value
from bridges.providers import s2_client


class _FakeResponse:
    def __init__(self, payload: dict[str, object]) -> None:
        self._payload = payload

    def read(self) -> bytes:
        return json.dumps(self._payload).encode("utf-8")

    def __enter__(self) -> "_FakeResponse":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        return None


class SemanticScholarClientTests(unittest.TestCase):
    def _capture_search_url(self, *args: object, **kwargs: object) -> str:
        captured_url = ""

        def fake_make_request(url: str) -> dict[str, object]:
            nonlocal captured_url
            captured_url = url
            return {"data": []}

        with mock.patch.object(s2_client, "_make_request", side_effect=fake_make_request):
            s2_client.search_paper(*args, **kwargs)  # type: ignore[arg-type]

        return captured_url

    def _search_params(self, url: str) -> dict[str, list[str]]:
        parsed = urllib.parse.urlparse(url)
        return urllib.parse.parse_qs(parsed.query)

    def test_search_paper_preserves_existing_request_behavior(self) -> None:
        url = self._capture_search_url("graph embeddings", 3)

        self.assertEqual(url.split("?", 1)[0], f"{s2_client.S2_GRAPH_BASE}/paper/search")
        self.assertEqual(
            url,
            f"{s2_client.S2_GRAPH_BASE}/paper/search?"
            "query=graph%20embeddings&limit=3&"
            "fields=paperId%2Ctitle%2Cauthors%2Cyear%2Cabstract%2Curl%2C"
            "citationCount%2Cvenue%2CexternalIds%2CopenAccessPdf",
        )

    def test_search_paper_url_includes_encoded_query_limit_and_default_fields(self) -> None:
        url = self._capture_search_url(query="retrieval augmented generation", limit=7)
        params = self._search_params(url)

        self.assertEqual(params["query"], ["retrieval augmented generation"])
        self.assertEqual(params["limit"], ["7"])
        self.assertEqual(
            params["fields"],
            ["paperId,title,authors,year,abstract,url,citationCount,venue,externalIds,openAccessPdf"],
        )

    def test_search_paper_adds_year_range_filter_when_start_and_end_are_valid(self) -> None:
        url = self._capture_search_url(query="ranking", limit=5, year_start=2020, year_end="2024")

        self.assertEqual(self._search_params(url)["year"], ["2020-2024"])

    def test_search_paper_adds_start_only_year_filter(self) -> None:
        url = self._capture_search_url(query="ranking", year_start="2021")

        self.assertEqual(self._search_params(url)["year"], ["2021-"])

    def test_search_paper_adds_end_only_year_filter(self) -> None:
        url = self._capture_search_url(query="ranking", year_end=2022)

        self.assertEqual(self._search_params(url)["year"], ["-2022"])

    def test_search_paper_ignores_invalid_year_filters(self) -> None:
        url = self._capture_search_url(query="ranking", year_start="20x1", year_end="")

        self.assertNotIn("year", self._search_params(url))

    def test_search_paper_accepts_field_override_as_list(self) -> None:
        url = self._capture_search_url(query="ranking", fields=["paperId", "title", "year"])

        self.assertEqual(
            self._search_params(url)["fields"],
            ["paperId,title,authors,year,abstract,url,citationCount,venue,externalIds,openAccessPdf"],
        )

    def test_search_paper_accepts_field_override_as_string(self) -> None:
        url = self._capture_search_url(query="ranking", fields="paperId,title,isOpenAccess")

        self.assertEqual(
            self._search_params(url)["fields"],
            [
                "paperId,title,authors,year,abstract,url,citationCount,venue,"
                "externalIds,openAccessPdf,isOpenAccess"
            ],
        )

    def test_search_paper_accepts_field_override_as_tuple(self) -> None:
        url = self._capture_search_url(query="ranking", fields=("paperId", "title", "isOpenAccess"))

        self.assertEqual(
            self._search_params(url)["fields"],
            [
                "paperId,title,authors,year,abstract,url,citationCount,venue,"
                "externalIds,openAccessPdf,isOpenAccess"
            ],
        )

    def test_search_paper_empty_field_override_uses_default_fields(self) -> None:
        url = self._capture_search_url(query="ranking", fields=[])

        self.assertEqual(self._search_params(url)["fields"], [s2_client.DEFAULT_SEARCH_FIELDS])

    def test_search_paper_appends_venue_and_type_to_query_without_unsupported_params(self) -> None:
        url = self._capture_search_url(
            query="ranking",
            venue="ACL Anthology",
            publication_type="Review",
        )
        params = self._search_params(url)

        self.assertEqual(params["query"], ["ranking ACL Anthology Review"])
        self.assertNotIn("venue", params)
        self.assertNotIn("publication_type", params)

    def test_search_paper_blank_query_returns_empty_data_without_network(self) -> None:
        with mock.patch.object(s2_client, "_make_request") as make_request:
            result = s2_client.search_paper("   ", limit=5, year_start=2020)

        self.assertEqual(result, {"data": []})
        make_request.assert_not_called()

    def test_search_paper_ignores_out_of_range_or_reversed_year_filters(self) -> None:
        reversed_range_url = self._capture_search_url(query="ranking", year_start=2025, year_end=2020)
        short_year_url = self._capture_search_url(query="ranking", year_start="1", year_end="99999")
        ancient_year_url = self._capture_search_url(query="ranking", year_start="1600")

        self.assertNotIn("year", self._search_params(reversed_range_url))
        self.assertNotIn("year", self._search_params(short_year_url))
        self.assertNotIn("year", self._search_params(ancient_year_url))

    def test_make_request_adds_api_key_header_when_present(self) -> None:
        captured_headers: dict[str, str] = {}

        def fake_urlopen(request, timeout):  # type: ignore[no-untyped-def]
            del timeout
            captured_headers.update(dict(request.header_items()))
            return _FakeResponse({"data": []})

        with mock.patch.dict(os.environ, {"SEMANTIC_SCHOLAR_API_KEY": "demo-key"}, clear=False):
            with mock.patch("urllib.request.urlopen", side_effect=fake_urlopen):
                result = s2_client._make_request("https://example.com")

        self.assertEqual(result, {"data": []})
        self.assertEqual(captured_headers["X-api-key"], "demo-key")

    def test_make_request_reads_api_key_from_global_provider_config(self) -> None:
        captured_headers: dict[str, str] = {}

        def fake_urlopen(request, timeout):  # type: ignore[no-untyped-def]
            del timeout
            captured_headers.update(dict(request.header_items()))
            return _FakeResponse({"data": []})

        with tempfile.TemporaryDirectory() as config_home, self.subTest("global provider config"):
            with mock.patch.dict(
                os.environ,
                {
                    "QIONGLI_CONFIG_HOME": config_home,
                    "QIONGLI_SEMANTIC_SCHOLAR_API_KEY": "",
                    "SEMANTIC_SCHOLAR_API_KEY": "",
                    "S2_API_KEY": "",
                },
                clear=False,
            ):
                set_provider_value("semantic-scholar", "api-key", "stored-key")
                with mock.patch("urllib.request.urlopen", side_effect=fake_urlopen):
                    result = s2_client._make_request("https://example.com")

        self.assertEqual(result, {"data": []})
        self.assertEqual(captured_headers["X-api-key"], "stored-key")

    def test_make_request_retries_http_429_then_succeeds(self) -> None:
        attempts = {"count": 0}

        def fake_urlopen(request, timeout):  # type: ignore[no-untyped-def]
            del request, timeout
            attempts["count"] += 1
            if attempts["count"] == 1:
                raise urllib.error.HTTPError(
                    "https://example.com",
                    429,
                    "Too Many Requests",
                    {"Retry-After": "1"},
                    None,
                )
            return _FakeResponse({"data": [{"paperId": "123"}]})

        with warnings.catch_warnings():
            warnings.simplefilter("ignore", ResourceWarning)
            with mock.patch("urllib.request.urlopen", side_effect=fake_urlopen):
                with mock.patch("time.sleep") as sleep_mock:
                    result = s2_client._make_request("https://example.com")

        self.assertEqual(result["data"], [{"paperId": "123"}])
        self.assertEqual(attempts["count"], 2)
        sleep_mock.assert_called_once_with(1.0)

    def test_make_request_returns_terminal_error_after_retries(self) -> None:
        http_error = urllib.error.HTTPError(
            "https://example.com",
            429,
            "Too Many Requests",
            {},
            None,
        )

        with mock.patch("urllib.request.urlopen", side_effect=http_error):
            with mock.patch("time.sleep"):
                result = s2_client._make_request("https://example.com")

        self.assertEqual(result["data"], [])
        self.assertIn("HTTP Error 429", result["error"])


if __name__ == "__main__":
    unittest.main()
