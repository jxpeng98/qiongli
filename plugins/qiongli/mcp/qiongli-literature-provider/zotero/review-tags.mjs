import { crossrefStatusTag } from "./crossref-verifier.mjs";

export const DEFAULT_REVIEW_TAGS = ["qiongli:imported", "qiongli:needs-review"];

export function resolveDefaultReviewTags(config = {}, input = {}) {
  const fromInput = normalizeStringList(input.review_tags);
  if (fromInput.length > 0) {
    return fromInput;
  }

  const fromConfig = normalizeStringList(config.default_review_tags);
  return fromConfig.length > 0 ? fromConfig : DEFAULT_REVIEW_TAGS;
}

export function mergeReviewTags({
  baseTags = [],
  provider = "",
  crossrefStatus = "skipped",
  defaultReviewTags = DEFAULT_REVIEW_TAGS
} = {}) {
  return normalizeStringList([
    ...normalizeStringList(baseTags),
    ...normalizeStringList(defaultReviewTags),
    provider ? `qiongli:source:${provider}` : "",
    crossrefStatusTag(crossrefStatus)
  ]);
}

export function reviewStatusForVerification({ writeStatus = "", crossrefStatus = "" } = {}) {
  if (writeStatus === "unchanged") {
    return "unchanged";
  }
  if (writeStatus === "skipped") {
    return "skipped";
  }
  return "needs_review";
}

export function normalizeStringList(value) {
  const values = Array.isArray(value) ? value : typeof value === "string" ? value.split(",") : [];
  const output = [];
  const seen = new Set();

  for (const item of values) {
    const cleaned = String(item ?? "").trim();
    if (!cleaned) {
      continue;
    }

    const key = cleaned.toLowerCase();
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    output.push(cleaned);
  }

  return output;
}
