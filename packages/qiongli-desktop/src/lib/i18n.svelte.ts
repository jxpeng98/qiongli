import type { TranslationCatalog } from './i18n/catalog';

export type Locale = 'en' | 'zh-CN';

const STORAGE_KEY = 'qiongli.locale';
const bootstrapCatalog: TranslationCatalog = {
  messages: {
    'app.subtitle': 'Research system',
    'nav.global': 'Qiongli',
    'nav.primary': 'Primary navigation',
    'nav.skip': 'Skip to main content',
    'nav.overview': 'Overview',
    'nav.library': 'Research Library',
    'nav.portfolio': 'Portfolio',
    'nav.integrations': 'Client Integrations',
    'nav.about': 'About',
    'sidebar.native': 'Native service',
    'sidebar.unavailable': 'Bridge unavailable',
    'sidebar.connecting': 'Connecting…',
    'sidebar.refresh': 'Refresh status',
    'theme.light': 'Light',
    'theme.dark': 'Dark',
    'theme.useLight': 'Use light mode',
    'theme.useDark': 'Use dark mode',
    'language.label': 'Language',
    'language.en': 'English',
    'language.zh-CN': '简体中文',
    'language.loading': 'Loading language…',
    'language.loadFailed': 'The selected language could not be loaded. Try again.',
    'common.loading': 'Loading…',
    'common.close': 'Close',
    'common.moreInformation': 'More information',
    'notice.actionFailed': 'Qiongli could not complete this action',
    'notice.dismiss': 'Dismiss message'
  },
  labels: {
    ready: 'Ready',
    attention: 'Attention',
    missing: 'Missing',
    blocked: 'Blocked',
    unknown: 'Unknown'
  },
  dynamicLabels: {},
  reasons: {}
};
const loadedCatalogs = new Map<Locale, TranslationCatalog>();
const pendingCatalogs = new Map<Locale, Promise<TranslationCatalog>>();

async function loadCatalog(locale: Locale): Promise<TranslationCatalog> {
  const loaded = loadedCatalogs.get(locale);
  if (loaded) return loaded;

  let pending = pendingCatalogs.get(locale);
  if (!pending) {
    pending = (locale === 'en'
      ? import('./i18n/locales/en')
      : import('./i18n/locales/zh-CN'))
      .then(({ default: catalog }) => {
        loadedCatalogs.set(locale, catalog);
        pendingCatalogs.delete(locale);
        return catalog;
      })
      .catch((error: unknown) => {
        pendingCatalogs.delete(locale);
        throw error;
      });
    pendingCatalogs.set(locale, pending);
  }
  return pending;
}

function preferredLocale(): Locale {
  if (typeof window === 'undefined') return 'en';
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (saved === 'en' || saved === 'zh-CN') return saved;
  } catch {
    // A restricted WebView can deny storage while still allowing localization.
  }
  return window.navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en';
}

function persistLocale(locale: Locale): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // The active in-memory locale remains valid when persistence is unavailable.
  }
}

class I18nState {
  private currentLocale = $state<Locale>('en');
  private catalog = $state.raw<TranslationCatalog>(bootstrapCatalog);
  private requestSequence = 0;

  loading = $state(false);
  loadFailed = $state(false);

  get locale(): Locale {
    return this.currentLocale;
  }

  get isChinese(): boolean {
    return this.currentLocale === 'zh-CN';
  }

  async initialize(): Promise<void> {
    const preferred = preferredLocale();
    try {
      await this.setLocale(preferred);
    } catch {
      if (preferred !== 'en') {
        try {
          this.catalog = await loadCatalog('en');
        } catch {
          this.catalog = bootstrapCatalog;
        }
      } else {
        this.catalog = bootstrapCatalog;
      }
      this.currentLocale = 'en';
      this.loading = false;
      this.loadFailed = true;
      if (typeof document !== 'undefined') document.documentElement.lang = 'en';
    }
  }

  async setLocale(locale: Locale): Promise<void> {
    const request = ++this.requestSequence;
    this.loading = true;
    this.loadFailed = false;
    try {
      const catalog = await loadCatalog(locale);
      if (request !== this.requestSequence) return;
      this.catalog = catalog;
      this.currentLocale = locale;
      if (typeof document !== 'undefined') document.documentElement.lang = locale;
      persistLocale(locale);
    } catch (error) {
      if (request === this.requestSequence) this.loadFailed = true;
      throw error;
    } finally {
      if (request === this.requestSequence) this.loading = false;
    }
  }

  t(key: string, values: Record<string, string | number> = {}): string {
    const template = this.catalog.messages[key] ?? bootstrapCatalog.messages[key] ?? key;
    return Object.entries(values).reduce(
      (text, [name, value]) => text.replaceAll(`{${name}}`, String(value)),
      template
    );
  }

  label(value: string): string {
    return this.catalog.labels[value]
      ?? value.replaceAll('-', ' ').replaceAll('_', ' ').replace(/^./, (letter) => letter.toUpperCase());
  }

  reason(code: string): string {
    return this.catalog.reasons[code] ?? code;
  }

  dynamic(value: string): string {
    return this.catalog.dynamicLabels[value] ?? value;
  }

  date(unixSeconds: number, includeTime = false): string {
    return new Intl.DateTimeFormat(this.currentLocale === 'zh-CN' ? 'zh-CN' : 'en-GB', {
      dateStyle: 'medium',
      ...(includeTime ? { timeStyle: 'short' as const } : {})
    }).format(new Date(unixSeconds * 1_000));
  }
}

export async function translationCatalogKeys(locale: Locale): Promise<string[]> {
  return Object.keys((await loadCatalog(locale)).messages).sort();
}

export const i18n = new I18nState();
