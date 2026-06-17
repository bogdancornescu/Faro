<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { snippets as store } from '$lib/stores/snippets.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import LeftPanel from '$lib/components/layout/LeftPanel.svelte';
  import CenterPanel from '$lib/components/layout/CenterPanel.svelte';
  import RightPanel from '$lib/components/layout/RightPanel.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import type { CreateSnippetInput, UpdateSnippetInput, TimePeriod, ContentType } from '$lib/types';

  let mode = $state<'idle' | 'creating' | 'editing'>('idle');
  let formDirty = $state(false);
  let settingsOpen = $state(false);
  let searchInputEl = $state<HTMLInputElement | undefined>();

  // Derive the currently selected Snippet object from the store
  const selectedSnippet = $derived(
    store.snippets.find(s => s.id === store.selectedId) ?? null
  );

  function isEditableTarget(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    return t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable;
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;

    if (ctrl && e.key === 'f') {
      e.preventDefault();
      searchInputEl?.focus();
      searchInputEl?.select();
      return;
    }

    if (ctrl && e.key === 'n') {
      if (isEditableTarget(e)) return;
      e.preventDefault();
      handleNewSnippet();
      return;
    }

    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      if (isEditableTarget(e)) return;
      const snippets = store.snippets;
      if (snippets.length === 0) return;
      e.preventDefault();
      const idx = snippets.findIndex(s => s.id === store.selectedId);
      const next = e.key === 'ArrowDown'
        ? Math.min(snippets.length - 1, idx + 1)
        : Math.max(0, idx - 1);
      const target = snippets[next];
      if (!target || target.id === store.selectedId) return;
      if (formDirty && !confirm('You have unsaved changes. Discard them?')) return;
      store.select(target.id);
      mode = 'editing';
      requestAnimationFrame(() => {
        document.querySelector(`[data-snippet-id="${target.id}"]`)?.scrollIntoView({ block: 'nearest' });
      });
    }
  }

  onMount(() => {
    let unlisten: (() => void) | null = null;

    listen('snippet-created', () => { store.load(); }).then(fn => {
      unlisten = fn;
    });

    store.load();
    settings.load();

    window.addEventListener('keydown', handleGlobalKeydown);

    return () => {
      if (unlisten) unlisten();
      window.removeEventListener('keydown', handleGlobalKeydown);
    };
  });

  function handleNewSnippet() {
    store.select(null);
    mode = 'creating';
  }

  function handleSelectSnippet(id: number) {
    if (formDirty && !confirm('You have unsaved changes. Discard them?')) return;
    store.select(id);
    mode = 'editing';
  }

  async function handleSave(input: CreateSnippetInput | UpdateSnippetInput) {
    if (mode === 'creating') {
      const s = await store.create(input as CreateSnippetInput);
      if (s) mode = 'editing';
    } else {
      await store.update(input as UpdateSnippetInput);
    }
  }

  function handleCancel() {
    formDirty = false;
    store.select(null);
    mode = 'idle';
  }

  async function handleDelete() {
    if (!store.selectedId) return;
    if (!confirm('Delete this snippet? This cannot be undone.')) return;
    await store.remove(store.selectedId);
    if (!store.error) {
      formDirty = false;
      mode = 'idle';
    }
  }
</script>

<div class="app-layout">
  <LeftPanel
    tags={store.visibleTags}
    activeFilter={store.tagFilter}
    contentTypeFilter={store.contentTypeFilter}
    timePeriodFilter={store.timePeriodFilter}
    onFilterChange={(tag) => {
      if (formDirty && !confirm('You have unsaved changes. Discard them?')) return;
      store.setTagFilter(tag);
      store.select(null);
      mode = 'idle';
    }}
    onContentTypeChange={(ct: ContentType | null) => {
      if (formDirty && !confirm('You have unsaved changes. Discard them?')) return;
      store.setContentTypeFilter(ct);
      store.select(null);
      mode = 'idle';
    }}
    onTimePeriodChange={(period: TimePeriod | null) => {
      if (formDirty && !confirm('You have unsaved changes. Discard them?')) return;
      store.setTimePeriodFilter(period);
      store.select(null);
      mode = 'idle';
    }}
    onNewSnippet={handleNewSnippet}
    onOpenSettings={() => { settingsOpen = true; }}
  />
  <CenterPanel
    snippets={store.snippets}
    selectedId={store.selectedId}
    loading={store.loading}
    searchQuery={store.searchQuery}
    tagFilter={store.tagFilter}
    contentTypeFilter={store.contentTypeFilter}
    timePeriodFilter={store.timePeriodFilter}
    onSelect={handleSelectSnippet}
    onSearch={(q) => {
      store.setSearch(q);
      store.select(null);
      mode = 'idle';
    }}
    bind:searchInputEl
  />
  <RightPanel
    snippet={selectedSnippet}
    {mode}
    onSave={handleSave}
    onCancel={handleCancel}
    onDelete={handleDelete}
    bind:dirty={formDirty}
  />
</div>

<SettingsModal open={settingsOpen} onclose={() => { settingsOpen = false; }} />

{#if store.error}
  <div class="error-bar">{store.error}</div>
{/if}

<style>
  .app-layout {
    display: grid;
    grid-template-columns: 270px 1fr 360px;
    grid-template-rows: 100vh;
    height: 100vh;
    overflow: hidden;
    background: var(--bg);
  }
  .error-bar {
    position: fixed;
    bottom: 1rem;
    left: 50%;
    transform: translateX(-50%);
    background: var(--danger);
    color: var(--accent-contrast);
    font-size: 12px;
    padding: 0.4rem 1rem;
    border-radius: var(--radius-control);
    max-width: 80vw;
    z-index: 100;
  }
</style>
