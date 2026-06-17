const CAPABILITIES = {
  openalex: {
    status: "implemented",
    max_per_provider_limit: 100,
    capabilities: [
      "topic_search",
      "doi_lookup",
      "year_filter",
      "document_type_filter",
      "venue_metadata",
      "metadata_enrichment"
    ]
  },
  semantic_scholar: {
    status: "implemented",
    max_per_provider_limit: 100,
    capabilities: [
      "topic_search",
      "title_lookup",
      "doi_lookup",
      "year_filter",
      "publication_type_metadata",
      "venue_metadata",
      "metadata_enrichment"
    ]
  },
  crossref: {
    status: "implemented",
    max_per_provider_limit: 100,
    capabilities: [
      "topic_search",
      "doi_lookup",
      "metadata_enrichment",
      "year_filter",
      "document_type_filter",
      "venue_metadata",
      "reference_metadata"
    ]
  },
  pubmed: {
    status: "implemented",
    max_per_provider_limit: 100,
    capabilities: [
      "topic_search",
      "doi_lookup",
      "biomedical_topic_search",
      "year_filter",
      "medical_subject_headings",
      "metadata_enrichment"
    ]
  }
};

export function providerCapabilities() {
  return JSON.parse(JSON.stringify(CAPABILITIES));
}
