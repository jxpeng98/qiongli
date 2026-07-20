import { beforeAll, beforeEach, describe, expect, it } from 'vitest';

import { i18n } from './i18n.svelte';

describe('i18n state', () => {
  beforeAll(() => {
    const values = new Map<string, string>();
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: {
        clear: () => values.clear(),
        getItem: (key: string) => values.get(key) ?? null,
        removeItem: (key: string) => values.delete(key),
        setItem: (key: string, value: string) => values.set(key, value)
      }
    });
  });

  beforeEach(() => {
    window.localStorage.clear();
    i18n.setLocale('en');
  });

  it('switches and persists Simplified Chinese', () => {
    i18n.setLocale('zh-CN');

    expect(i18n.t('nav.library')).toBe('研究库');
    expect(i18n.label('ready')).toBe('就绪');
    expect(document.documentElement.lang).toBe('zh-CN');
    expect(window.localStorage.getItem('qiongli.locale')).toBe('zh-CN');
  });

  it('interpolates messages and localizes project selection errors', () => {
    i18n.setLocale('zh-CN');

    expect(i18n.t('library.risks', { count: 2 })).toBe('2 个未解决风险');
    expect(i18n.reason('multiple-article-projects-found-select-topic'))
      .toContain('多个文章主题');
  });
});
