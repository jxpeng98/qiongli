import { beforeAll, beforeEach, describe, expect, it } from 'vitest';

import { i18n, translationCatalogKeys } from './i18n.svelte';

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
    expect(i18n.t('nav.timeline')).toBe('时间线');
    expect(i18n.label('delivery-transitioned-at')).toBe('交付转换记录');
    expect(i18n.t('dialog.consolidationReview')).toBe('学术整合审阅');
    expect(i18n.t('library.summaryAria')).toBe('研究库摘要');
    expect(i18n.t('orchestrator.hostTitle')).toContain('穷理');
  });

  it('keeps local Full, manual Desktop, and remote-only boundaries distinct', () => {
    expect(i18n.t('integrations.fullLocal')).toBe('Full local');
    expect(i18n.t('integrations.manualMcpb')).toBe('Manual Full MCPB');
    expect(i18n.t('integrations.remoteOnly')).toBe('Remote-only');

    i18n.setLocale('zh-CN');
    expect(i18n.t('integrations.claudeDesktopDescription')).toContain('文献 MCPB');
    expect(i18n.t('integrations.remoteDescription')).toContain('远程 Worker');
  });

  it('keeps the English and Chinese message catalogs structurally complete', () => {
    expect(translationCatalogKeys('zh-CN')).toEqual(translationCatalogKeys('en'));
  });
});
