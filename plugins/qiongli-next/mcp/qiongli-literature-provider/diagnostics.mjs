function numericStat(value, fallback = 0) {
  return Number.isInteger(value) && value >= 0 ? value : fallback;
}

function providerDiagnostic(response) {
  const resultCount = Array.isArray(response?.results) ? response.results.length : 0;
  const requestCount = numericStat(response?.request_count, 1);

  return {
    provider: response?.provider ?? "unknown",
    status: response?.error ? "failed" : "success",
    result_count: resultCount,
    request_count: requestCount,
    attempts: numericStat(response?.attempts, requestCount),
    error: response?.error ?? null
  };
}

function queryDiagnostic(response) {
  return {
    query_id: response?.query_id ?? null,
    query: response?.query ?? null,
    ...providerDiagnostic(response)
  };
}

function aggregateProviderDiagnostics(responses) {
  const summaries = new Map();

  for (const response of responses) {
    const provider = response?.provider ?? "unknown";
    if (!summaries.has(provider)) {
      summaries.set(provider, {
        provider,
        result_count: 0,
        request_count: 0,
        attempts: 0,
        success_count: 0,
        failure_count: 0,
        error: null
      });
    }

    const summary = summaries.get(provider);
    summary.result_count += Array.isArray(response?.results) ? response.results.length : 0;
    const requestCount = numericStat(response?.request_count, 1);
    summary.request_count += requestCount;
    summary.attempts += numericStat(response?.attempts, requestCount);

    if (response?.error) {
      summary.failure_count += 1;
      summary.error ??= response.error;
    } else {
      summary.success_count += 1;
    }
  }

  return Array.from(summaries.values()).map((summary) => ({
    provider: summary.provider,
    status: summary.success_count > 0 ? "success" : "failed",
    result_count: summary.result_count,
    request_count: summary.request_count,
    attempts: summary.attempts,
    error: summary.success_count > 0 ? null : summary.error
  }));
}

function resultSearchText(result) {
  return [
    result?.title,
    result?.venue,
    result?.document_type,
    result?.abstract
  ].map((value) => String(value ?? "").toLowerCase()).join(" ");
}

function isWorkingPaper(result) {
  const text = resultSearchText(result);
  return (
    text.includes("working paper") ||
    text.includes("working-paper") ||
    text.includes("discussion paper") ||
    text.includes("nber") ||
    text.includes("ssrn") ||
    text.includes("repec")
  );
}

function isPublishedVersion(result) {
  if (isWorkingPaper(result)) {
    return false;
  }

  const type = String(result?.document_type ?? "").toLowerCase();
  return Boolean(result?.doi) || type.includes("journal") || type === "article";
}

function domainDiagnostics(queryPlan, results) {
  const domain = queryPlan?.domain ?? "general";
  const matchedTerms = Array.isArray(queryPlan?.domain_terms) ? queryPlan.domain_terms : [];
  const workingPaperCount = results.filter(isWorkingPaper).length;
  const publishedVersionCount = results.filter(isPublishedVersion).length;

  return {
    domain,
    field_term_coverage: {
      covered: domain === "finance_economics" && matchedTerms.length > 0,
      matched_terms: matchedTerms
    },
    working_paper_coverage: {
      covered: workingPaperCount > 0,
      result_count: workingPaperCount
    },
    published_version_coverage: {
      covered: publishedVersionCount > 0,
      result_count: publishedVersionCount
    }
  };
}

export function searchDiagnostics({ responses, rawResults, dedupedResults, filteredResults, outputResults, queryPlan }) {
  const coverageResults = filteredResults;

  return {
    raw_result_count: rawResults.length,
    deduped_result_count: dedupedResults.length,
    filtered_result_count: filteredResults.length,
    returned_result_count: outputResults.length,
    coverage_result_count: coverageResults.length,
    ...domainDiagnostics(queryPlan, coverageResults),
    providers: aggregateProviderDiagnostics(responses),
    queries: responses.map(queryDiagnostic)
  };
}

export function appendSearchWarnings(warnings, outputResults, options, diagnostics) {
  const merged = [...warnings];

  if (
    options.minimumResultThreshold > 0 &&
    outputResults.length < options.minimumResultThreshold
  ) {
    merged.push("insufficient_review_results");
  }

  if (options.includeCitations) {
    merged.push("citation_expansion_limited");
  }

  if (options.includeReferences) {
    merged.push("reference_expansion_limited");
  }

  if (diagnostics?.domain === "finance_economics" && options.searchDepth === "deep") {
    if (diagnostics.working_paper_coverage?.covered === false) {
      merged.push("missing_working_paper_coverage");
    }
    if (diagnostics.published_version_coverage?.covered === false) {
      merged.push("missing_published_version_coverage");
    }
  }

  return [...new Set(merged)];
}
