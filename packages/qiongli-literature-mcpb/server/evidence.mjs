function uniqueList(values) {
  return [...new Set((values ?? []).filter((value) => typeof value === "string" && value.trim() !== ""))];
}

function readResultCount(value) {
  return Number.isInteger(value) && value >= 0 ? value : 0;
}

export function buildEvidence(input = {}) {
  const attempted = uniqueList(input.attemptedProviders);
  const successful = uniqueList(input.successfulProviders);
  const failed = uniqueList(input.failedProviders);
  const warnings = [];

  if (attempted.length > 0 && successful.length === 0 && failed.length > 0) {
    warnings.push("all_providers_failed");
  }

  if (successful.length === 1) {
    warnings.push("single_successful_provider");
  }

  if (successful.length > 0 && failed.length > 0) {
    warnings.push("partial_provider_failure");
  }

  return {
    capability_mode: successful.length > 0 ? "provider_connected" : "strategy_only",
    providers: {
      attempted,
      successful,
      failed
    },
    result_count: readResultCount(input.resultCount),
    warnings
  };
}
