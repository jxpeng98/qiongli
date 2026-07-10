from __future__ import annotations

import json
import unittest
from unittest import mock

from bridges.providers import crossref_client, openalex_client, pubmed_client


class ProviderClientTests(unittest.TestCase):
    def test_openalex_search_uses_query_filters_and_normalizes_results(self) -> None:
        captured: dict[str, str] = {}

        def fake_urlopen(request, timeout=0):
            captured["url"] = request.full_url
            return _FakeResponse(
                {
                    "results": [
                        {
                            "id": "https://openalex.org/W1",
                            "display_name": "OpenAlex Paper",
                            "publication_year": 2024,
                        }
                    ]
                }
            )

        with mock.patch("urllib.request.urlopen", fake_urlopen):
            result = openalex_client.search(
                {"translated_query": "AI education", "filters": {"year_start": 2020}},
                3,
                api_key="openalex-test-key",
                email="maintainer@example.com",
            )

        self.assertIn("search=AI%20education", captured["url"])
        self.assertIn("per-page=3", captured["url"])
        self.assertIn("api_key=openalex-test-key", captured["url"])
        self.assertIn("mailto=maintainer%40example.com", captured["url"])
        self.assertEqual(result["data"][0]["title"], "OpenAlex Paper")
        self.assertEqual(result["data"][0]["provider"], "openalex")

    def test_crossref_search_normalizes_items(self) -> None:
        def fake_urlopen(request, timeout=0):
            return _FakeResponse(
                {
                    "message": {
                        "items": [
                            {
                                "DOI": "10.1/demo",
                                "title": ["Crossref Paper"],
                                "issued": {"date-parts": [[2023]]},
                            }
                        ]
                    }
                }
            )

        with mock.patch("urllib.request.urlopen", fake_urlopen):
            result = crossref_client.search({"translated_query": "AI education", "filters": {}}, 2)

        self.assertEqual(result["data"][0]["title"], "Crossref Paper")
        self.assertEqual(result["data"][0]["doi"], "10.1/demo")
        self.assertEqual(result["data"][0]["provider"], "crossref")

    def test_pubmed_search_normalizes_esearch_and_esummary(self) -> None:
        responses = [
            {"esearchresult": {"idlist": ["123"]}},
            {"result": {"123": {"title": "PubMed Paper", "pubdate": "2022 Jan", "source": "Demo Journal"}}},
        ]

        def fake_urlopen(request, timeout=0):
            return _FakeResponse(responses.pop(0))

        with mock.patch("urllib.request.urlopen", fake_urlopen):
            result = pubmed_client.search({"translated_query": "AI education", "filters": {}}, 5)

        self.assertEqual(result["data"][0]["title"], "PubMed Paper")
        self.assertEqual(result["data"][0]["year"], 2022)
        self.assertEqual(result["data"][0]["provider"], "pubmed")


class _FakeResponse:
    def __init__(self, payload: dict):
        self.payload = payload

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        return False

    def read(self) -> bytes:
        return json.dumps(self.payload).encode("utf-8")


if __name__ == "__main__":
    unittest.main()
