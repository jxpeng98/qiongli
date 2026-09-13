export interface TranslationCatalog {
  messages: Record<string, string>;
  labels: Record<string, string>;
  dynamicLabels: Record<string, string>;
  reasons: Record<string, string>;
}
