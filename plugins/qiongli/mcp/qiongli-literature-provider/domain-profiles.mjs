const FINANCE_ECONOMICS_PROFILE = {
  id: "finance_economics",
  label: "Finance and Economics",
  generic_terms: ["finance", "economics"],
  terms: [
    "finance",
    "financial economics",
    "economics",
    "asset pricing",
    "corporate finance",
    "economic policy",
    "monetary policy",
    "fiscal policy",
    "macroeconomic",
    "macroeconomics",
    "microeconomic",
    "microeconomics",
    "econometrics",
    "financial market",
    "stock market",
    "banking",
    "central bank",
    "inflation",
    "exchange rate",
    "accounting",
    "earnings",
    "jel",
    "nber",
    "repec",
    "ssrn"
  ],
  variant_source: "domain_variant",
  variant_rationale: "finance/economics domain query variant"
};

const GENERAL_DOMAIN = {
  id: "general",
  label: "General Academic Search",
  matched_terms: [],
  profile: null
};

const DOMAIN_PROFILES = [
  FINANCE_ECONOMICS_PROFILE
];

function cleanText(value) {
  return String(value ?? "").trim();
}

function comparableQuery(value) {
  return cleanText(value)
    .normalize("NFKD")
    .toLowerCase()
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function termPattern(term) {
  const words = comparableQuery(term).split(" ").map(escapeRegExp);
  return new RegExp(`(?:^|\\s)${words.join("\\s+")}(?:\\s|$)`, "u");
}

function termMatches(query, term) {
  return termPattern(term).test(query);
}

function matchedProfileTerms(profile, query) {
  let matchedTerms = profile.terms.filter((term) => termMatches(query, term));
  const genericTerms = new Set(profile.generic_terms ?? []);
  const specificTerms = matchedTerms.filter((term) => !genericTerms.has(term));
  if (specificTerms.length > 0) {
    matchedTerms = specificTerms;
  }

  return matchedTerms;
}

function financeEconomicsJelVariant(query, matchedTerms) {
  if (matchedTerms.some((term) => ["asset pricing", "financial market", "stock market"].includes(term))) {
    return `${query} JEL G12`;
  }

  if (matchedTerms.some((term) => ["corporate finance", "earnings"].includes(term))) {
    return `${query} JEL G30`;
  }

  if (matchedTerms.some((term) => ["banking", "central bank"].includes(term))) {
    return `${query} JEL G21`;
  }

  if (matchedTerms.some((term) => ["monetary policy", "inflation", "exchange rate"].includes(term))) {
    return `${query} JEL E52`;
  }

  if (matchedTerms.some((term) => ["accounting"].includes(term))) {
    return `${query} JEL M41`;
  }

  return `${query} JEL G00`;
}

function financeEconomicsDeepVariants(query, domain) {
  return [
    `${query} working paper`,
    financeEconomicsJelVariant(query, domain.matched_terms),
    `${query} review`
  ];
}

export function detectSearchDomain(query) {
  const comparable = comparableQuery(query);
  for (const profile of DOMAIN_PROFILES) {
    const matchedTerms = matchedProfileTerms(profile, comparable);
    if (matchedTerms.length > 0) {
      return {
        id: profile.id,
        label: profile.label,
        matched_terms: matchedTerms,
        profile
      };
    }
  }

  return { ...GENERAL_DOMAIN };
}

export function domainProfilePayload(domain) {
  return {
    id: domain?.id ?? GENERAL_DOMAIN.id,
    label: domain?.label ?? GENERAL_DOMAIN.label
  };
}

export function domainAutomaticVariants(query, domain) {
  if (domain?.id === FINANCE_ECONOMICS_PROFILE.id) {
    return financeEconomicsDeepVariants(query, domain);
  }

  return [];
}

export function domainVariantSource(domain) {
  return domain?.profile?.variant_source ?? "auto_variant";
}

export function domainVariantRationale(domain) {
  return domain?.profile?.variant_rationale ?? "automatic deep-search query variant";
}
