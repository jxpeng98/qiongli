from __future__ import annotations

import unittest
from unittest import mock

from bridges.providers import arxiv_client


ATOM_FEED = """<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
  <entry>
    <id>http://arxiv.org/abs/2401.01234v2</id>
    <updated>2024-01-04T00:00:00Z</updated>
    <published>2024-01-02T00:00:00Z</published>
    <title>  arXiv Test   Paper </title>
    <summary>
      Results are normalized
      from Atom XML.
    </summary>
    <author><name>Ada Lovelace</name></author>
    <author><name>Grace Hopper</name></author>
    <link href="http://arxiv.org/abs/2401.01234v2" rel="alternate" type="text/html" />
    <link title="pdf" href="http://arxiv.org/pdf/2401.01234v2" rel="related" type="application/pdf" />
    <arxiv:doi>10.48550/arXiv.2401.01234</arxiv:doi>
    <category term="cs.AI" />
  </entry>
</feed>
"""


class ArxivClientTests(unittest.TestCase):
    def test_search_uses_payload_query_and_normalizes_atom_entries(self) -> None:
        calls: list[tuple[str, dict[str, str | int]]] = []

        def fake_get_text(url: str, params: dict[str, str | int]) -> str:
            calls.append((url, params))
            return ATOM_FEED

        with mock.patch("bridges.providers.arxiv_client._get_text", fake_get_text):
            response = arxiv_client.search(
                {
                    "translated_query": "fallback query",
                    "payload": {
                        "search_query": 'all:"machine learning" AND cat:cs.*',
                    },
                },
                3,
            )

        self.assertEqual(
            response,
            {
                "data": [
                    {
                        "paperId": "2401.01234v2",
                        "title": "arXiv Test Paper",
                        "authors": [{"name": "Ada Lovelace"}, {"name": "Grace Hopper"}],
                        "year": 2024,
                        "abstract": "Results are normalized from Atom XML.",
                        "url": "http://arxiv.org/abs/2401.01234v2",
                        "venue": "arXiv",
                        "externalIds": {
                            "ArXiv": "2401.01234v2",
                            "DOI": "10.48550/arXiv.2401.01234",
                        },
                        "citationCount": None,
                        "openAccessPdf": {"url": "http://arxiv.org/pdf/2401.01234v2"},
                        "provider": "arxiv",
                    }
                ]
            },
        )
        self.assertEqual(
            calls,
            [
                (
                    arxiv_client.ARXIV_QUERY_URL,
                    {
                        "search_query": 'all:"machine learning" AND cat:cs.*',
                        "start": 0,
                        "max_results": 3,
                        "sortBy": "relevance",
                        "sortOrder": "descending",
                    },
                )
            ],
        )


if __name__ == "__main__":
    unittest.main()
