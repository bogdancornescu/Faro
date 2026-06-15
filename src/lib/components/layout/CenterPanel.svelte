<script lang="ts">
  import SnippetCard from '$lib/components/SnippetCard.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import type { Snippet, TimePeriod } from '$lib/types';

  let { snippets, selectedId, loading = false, searchQuery, tagFilter = null, timePeriodFilter = null, onSelect, onSearch, searchInputEl = $bindable() }: {
    snippets: Snippet[];
    selectedId: number | null;
    loading?: boolean;
    searchQuery: string;
    tagFilter?: string | null;
    timePeriodFilter?: TimePeriod | null;
    onSelect: (id: number) => void;
    onSearch: (q: string) => void;
    searchInputEl?: HTMLInputElement;
  } = $props();

  const PERIOD_LABELS: Record<TimePeriod, string> = {
    'today': 'today',
    'yesterday': 'yesterday',
    'this-week': 'this week',
    'last-week': 'last week',
    'older': 'older than two weeks',
  };

  const emptyMessage = $derived(
    searchQuery
      ? 'No snippets match your search.'
      : timePeriodFilter
        ? `No snippets from ${PERIOD_LABELS[timePeriodFilter]}.`
        : tagFilter
          ? `No snippets tagged "${tagFilter}".`
          : 'No snippets yet — create one with the button on the left.'
  );
</script>

<div class="center-panel">
  <div class="search-bar-wrap">
    <SearchBar value={searchQuery} onchange={onSearch} bind:searchInput={searchInputEl} />
  </div>
  <div class="card-list">
    {#if loading}
      <p class="empty">Loading…</p>
    {:else if snippets.length === 0}
      <p class="empty">{emptyMessage}</p>
    {:else}
      {#each snippets as snippet (snippet.id)}
        <SnippetCard
          {snippet}
          selected={selectedId === snippet.id}
          onselect={onSelect}
        />
      {/each}
    {/if}
  </div>
</div>

<style>
  .center-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-right: 1px solid var(--border);
  }
  .search-bar-wrap {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .card-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--list-pad);
    display: flex;
    flex-direction: column;
    gap: var(--list-gap);
  }
  .empty {
    padding: 2.5rem 1.25rem;
    color: var(--text-muted);
    font-size: 13px;
    text-align: center;
    margin: 0;
  }
</style>
