import { render, screen } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeAll, beforeEach, describe, expect, it } from 'vitest';

import StatusBadge from './components/app/StatusBadge.svelte';
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

  beforeEach(async () => {
    window.localStorage.clear();
    await i18n.setLocale('en');
  });

  it('switches and persists Simplified Chinese', async () => {
    await i18n.setLocale('zh-CN');

    expect(i18n.t('nav.library')).toBe('研究库');
    expect(i18n.label('ready')).toBe('就绪');
    expect(document.documentElement.lang).toBe('zh-CN');
    expect(window.localStorage.getItem('qiongli.locale')).toBe('zh-CN');
  });

  it('loads the persisted catalog before initialization resolves', async () => {
    window.localStorage.setItem('qiongli.locale', 'zh-CN');

    await i18n.initialize();

    expect(i18n.locale).toBe('zh-CN');
    expect(i18n.t('nav.library')).toBe('研究库');
    expect(document.documentElement.lang).toBe('zh-CN');
  });

  it('updates already-mounted translated surfaces after a lazy catalog switch', async () => {
    render(StatusBadge, { status: 'ready' });
    expect(screen.getByText('Ready')).toBeInTheDocument();

    await i18n.setLocale('zh-CN');
    await tick();

    expect(screen.getByText('就绪')).toBeInTheDocument();
    expect(screen.queryByText('Ready')).not.toBeInTheDocument();
  });

  it('interpolates messages and localizes project selection errors', async () => {
    await i18n.setLocale('zh-CN');

    expect(i18n.t('library.risks', { count: 2 })).toBe('2 个未解决风险');
    expect(i18n.reason('multiple-article-projects-found-select-topic'))
      .toContain('多个文章主题');
    expect(i18n.reason('project-skills-project-archived'))
      .toContain('恢复项目');
    expect(i18n.reason('project-skills-library-revision-conflict'))
      .toContain('重新审阅');
    expect(i18n.t('nav.timeline')).toBe('时间线');
    expect(i18n.label('delivery-transitioned-at')).toBe('交付转换记录');
    expect(i18n.t('dialog.consolidationReview')).toBe('学术整合审阅');
    expect(i18n.t('library.summaryAria')).toBe('研究库摘要');
    expect(i18n.t('orchestrator.hostTitle')).toContain('穷理');
  });

  it('keeps local Full, manual Desktop, and remote-only boundaries distinct', async () => {
    expect(i18n.t('integrations.fullLocal')).toBe('Full local');
    expect(i18n.t('integrations.manualMcpb')).toBe('Manual Full MCPB');
    expect(i18n.t('integrations.remoteOnly')).toBe('Remote-only');

    await i18n.setLocale('zh-CN');
    expect(i18n.t('integrations.claudeDesktopDescription')).toContain('文献 MCPB');
    expect(i18n.t('integrations.remoteDescription')).toContain('远程 Worker');
  });

  it('does not retain retired direct-model or duplicate host-content messages', async () => {
    const keys = await translationCatalogKeys('en');

    expect(keys).toContain('orchestrator.hostTitle');
    expect(keys).toContain('backend.legacyCredentialTitle');
    expect(keys).not.toContain('backend.title');
    expect(keys).not.toContain('backend.test');
    expect(keys).not.toContain('backend.previewRun');
    expect(keys).not.toContain('orchestrator.runtimeTitle');
    expect(keys).not.toContain('orchestrator.previewTest');
    expect(keys).not.toContain('content.preset.codex');
    expect(keys).not.toContain('content.preset.claude');
    expect(keys).not.toContain('nav.backend');
    expect(keys).not.toContain('nav.content');
  });

  it('keeps the English and Chinese message catalogs structurally complete', async () => {
    expect(await translationCatalogKeys('zh-CN'))
      .toEqual(await translationCatalogKeys('en'));
  });
});
