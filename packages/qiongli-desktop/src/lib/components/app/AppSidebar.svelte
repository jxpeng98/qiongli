<script lang="ts">
  import {
    BookOpenText,
    Cable,
    Database,
    Info,
    Languages,
    LayoutDashboard,
    Moon,
    Network,
    RefreshCw,
    Sun
  } from '@lucide/svelte';

  import { Button } from '$lib/components/ui/button';
  import { NativeSelect } from '$lib/components/ui/native-select';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import { useAppState } from '$lib/context';
  import { i18n, type Locale } from '$lib/i18n.svelte';

  let {
    currentPath,
    theme,
    onToggleTheme
  }: {
    currentPath: string;
    theme: 'light' | 'dark';
    onToggleTheme: () => void;
  } = $props();

  const app = useAppState();
  const sidebar = Sidebar.useSidebar();
  const navigation = [
    { href: '/overview', label: 'nav.overview', icon: LayoutDashboard },
    { href: '/research-library', label: 'nav.library', icon: BookOpenText },
    { href: '/portfolio', label: 'nav.portfolio', icon: Database },
    { href: '/client-integrations', label: 'nav.integrations', icon: Cable },
    { href: '/about', label: 'nav.about', icon: Info }
  ];

  function changeLanguage(event: Event): void {
    const locale = (event.currentTarget as HTMLSelectElement).value as Locale;
    if (locale === i18n.locale) return;
    app.dismissNotice();
    i18n.setLocale(locale);
  }

  function closeMobileNavigation(): void {
    if (sidebar.isMobile) sidebar.setOpenMobile(false);
  }
</script>

<Sidebar.Root class="app-sidebar" collapsible="offcanvas">
  <Sidebar.Header class="app-sidebar-header">
    <a class="brand" href="/overview" onclick={closeMobileNavigation}>
      <span class="mark" aria-hidden="true"><Network size={21} strokeWidth={1.9} /></span>
      <span class="brand-copy">
        <strong>Qiongli</strong>
        <small>{i18n.t('app.subtitle')}</small>
      </span>
    </a>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>{i18n.t('nav.global')}</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each navigation as item (item.href)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                isActive={currentPath === item.href}
                tooltipContent={i18n.t(item.label)}
              >
                {#snippet child({ props })}
                  <a
                    {...props}
                    href={item.href}
                    aria-current={currentPath === item.href ? 'page' : undefined}
                    onclick={closeMobileNavigation}
                  >
                    <item.icon size={18} strokeWidth={1.9} aria-hidden="true" />
                    <span>{i18n.t(item.label)}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer class="app-sidebar-footer">
    <Sidebar.Separator />
    <label class="language-control">
      <span><Languages size={15} aria-hidden="true" />{i18n.t('language.label')}</span>
      <NativeSelect
        class="language-select"
        size="sm"
        value={i18n.locale}
        aria-label={i18n.t('language.label')}
        onchange={changeLanguage}
      >
        <option value="en">{i18n.t('language.en')}</option>
        <option value="zh-CN">{i18n.t('language.zh-CN')}</option>
      </NativeSelect>
    </label>

    <div class="runtime" role="status">
      <span class:online={app.bridgeReady} class="runtime-dot" aria-hidden="true"></span>
      <span class="runtime-copy">
        <strong>{app.bridgeReady ? i18n.t('sidebar.native') : i18n.t('sidebar.unavailable')}</strong>
        <small>{app.snapshot?.product.version ?? i18n.t('sidebar.connecting')}</small>
      </span>
    </div>

    <div class="utility-controls">
      <Button
        variant="outline"
        size="sm"
        aria-label={i18n.t(theme === 'dark' ? 'theme.useLight' : 'theme.useDark')}
        aria-pressed={theme === 'dark'}
        title={i18n.t(theme === 'dark' ? 'theme.useLight' : 'theme.useDark')}
        onclick={onToggleTheme}
      >
        {#if theme === 'dark'}<Sun size={16} aria-hidden="true" />{:else}<Moon size={16} aria-hidden="true" />{/if}
        <span>{i18n.t(theme === 'dark' ? 'theme.light' : 'theme.dark')}</span>
      </Button>
      <Button
        variant="outline"
        size="sm"
        aria-label={i18n.t('sidebar.refresh')}
        disabled={app.loading}
        onclick={() => app.refresh()}
      >
        <RefreshCw size={16} class={app.loading ? 'spin' : undefined} aria-hidden="true" />
        <span>{i18n.t('sidebar.refresh')}</span>
      </Button>
    </div>
  </Sidebar.Footer>
</Sidebar.Root>

<style>
  :global(.app-sidebar) {
    background: var(--color-sidebar);
  }

  :global(.app-sidebar [data-slot='sidebar-inner']) {
    border-color: var(--color-border);
    background: var(--color-sidebar);
    box-shadow: none;
  }

  .app-sidebar-header { padding: 8px 8px 3px; }

  .brand {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 8px;
    border-radius: var(--radius-md);
    padding: 3px;
    color: inherit;
    text-decoration: none;
  }

  .mark {
    display: grid;
    width: 28px;
    height: 28px;
    flex: none;
    place-items: center;
    border-radius: 50%;
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }

  .brand-copy,
  .brand-copy strong,
  .brand-copy small,
  .runtime-copy,
  .runtime-copy strong,
  .runtime-copy small { display: block; min-width: 0; }

  .brand-copy strong { color: var(--color-ink-strong); font-size: 15px; font-weight: 600; letter-spacing: -0.015em; }
  .brand-copy small,
  .runtime-copy small { color: var(--color-muted); font-size: 11px; }

  .app-sidebar-footer { padding: 5px 8px 8px; }

  .language-control { display: grid; min-width: 0; gap: 4px; }
  .language-control > span {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--color-muted);
    font-size: 11px;
    font-weight: 600;
  }
  :global(.language-select) { width: 100%; }

  .runtime {
    display: grid;
    min-width: 0;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 7px;
    padding: 3px 2px;
  }

  .runtime-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--color-danger); }
  .runtime-dot.online { background: var(--color-success); }
  .runtime-copy strong { color: var(--color-ink); font-size: 12px; line-height: 1.35; overflow-wrap: anywhere; }
  .runtime-copy small { line-height: 1.35; overflow-wrap: anywhere; }

  .utility-controls { display: grid; grid-template-columns: 1fr; gap: 4px; }
  .utility-controls :global(button) { min-width: 0; }

  @media (max-width: 767px) {
    :global(.app-sidebar) { background: var(--color-sidebar-strong); }
  }

</style>
