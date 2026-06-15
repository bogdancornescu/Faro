<script lang="ts">
  import { Settings, Plus, Clock, Tag as TagIcon, Search } from 'lucide-svelte';
  import type { Tag, TimePeriod } from '$lib/types';

  const TIMELINE_ITEMS: { label: string; value: TimePeriod }[] = [
    { label: 'Today', value: 'today' },
    { label: 'Yesterday', value: 'yesterday' },
    { label: 'This week', value: 'this-week' },
    { label: 'Last week', value: 'last-week' },
    { label: 'Older', value: 'older' },
  ];

  let {
    tags,
    activeFilter = null,
    onFilterChange,
    onNewSnippet,
    onOpenSettings = () => {},
    timePeriodFilter = null,
    onTimePeriodChange = () => {},
  }: {
    tags: Tag[];
    activeFilter?: string | null;
    onFilterChange: (tag: string | null) => void;
    onNewSnippet: () => void;
    onOpenSettings?: () => void;
    timePeriodFilter?: TimePeriod | null;
    onTimePeriodChange?: (period: TimePeriod | null) => void;
  } = $props();

  let tagSearch = $state('');
  const filteredTags = $derived(
    tagSearch.trim()
      ? tags.filter(t => t.name.toLowerCase().includes(tagSearch.toLowerCase()))
      : tags
  );
</script>

<aside class="left-panel">
  <!-- <div class="panel-header">
    <img src="/logo.svg" alt="Faro Logo" width="64" height="64" />
    <span class="brand">Faro</span>
  </div> -->

  <div class="panel-body">
    <!-- Timeline section -->
    <div class="section">
      <div class="section-label"><Clock size={10} strokeWidth={2.5} />Timeline</div>
      {#each TIMELINE_ITEMS as item (item.value)}
        <button
          class="nav-item"
          class:active={timePeriodFilter === item.value}
          onclick={() => onTimePeriodChange(item.value)}
        >
          {item.label}
        </button>
      {/each}
    </div>

    <div class="divider"></div>

    <!-- Tags section -->
    <div class="section tags-section">
      <div class="section-label"><TagIcon size={10} strokeWidth={2.5} />Tags</div>
      <div class="tag-search-wrap">
        <span class="tag-search-icon"><Search size={11} strokeWidth={2} /></span>
        <input
          class="tag-search"
          type="text"
          placeholder="Filter tags…"
          bind:value={tagSearch}
          onkeydown={(e) => { if (e.key === 'Escape') tagSearch = ''; }}
        />
      </div>
      {#if !tagSearch.trim()}
        <button
          class="nav-item"
          class:active={activeFilter === null && timePeriodFilter === null}
          onclick={() => { onFilterChange(null); onTimePeriodChange(null); }}
        >
          All snippets
        </button>
      {/if}
      {#each filteredTags as tag (tag.id)}
        <button
          class="nav-item"
          class:active={activeFilter === tag.name}
          onclick={() => onFilterChange(tag.name)}
        >
          <span class="tag-name">{tag.name}</span>
          {#if tag.count}
            <span class="tag-count">{tag.count}</span>
          {/if}
        </button>
      {/each}
      {#if tagSearch.trim() && filteredTags.length === 0}
        <p class="tag-empty">No tags match</p>
      {/if}
    </div>
  </div>

  <div class="panel-footer">
    <button class="new-btn" onclick={onNewSnippet}>
      <Plus size={14} strokeWidth={2.5} />New Snippet
    </button>
    <button class="settings-btn" onclick={onOpenSettings} title="Settings">
      <Settings size={14} strokeWidth={1.75} />
    </button>
  </div>
</aside>

<style>
  .left-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-right: 1px solid var(--border);
    background: var(--bg-sidebar);
  }
  /* .panel-header {
    padding: 1rem 1rem 0.75rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .brand {
    font-size: 28px;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 0.06em;
  } */
  .panel-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.5rem 0;
    display: flex;
    flex-direction: column;
  }
  .section {
    padding: 0.25rem 0;
  }
  .tags-section {
    flex: 1;
    overflow-y: auto;
  }
  .section-label {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
    padding: 0.4rem 1rem 0.25rem;
  }
  .divider {
    height: 1px;
    background: var(--border);
    margin: 0.25rem 0;
  }
  .tag-search-wrap {
    position: relative;
    display: flex;
    align-items: center;
    margin: 0.2rem 0.75rem 0.25rem;
  }
  .tag-search-icon {
    position: absolute;
    left: 0.5rem;
    display: flex;
    align-items: center;
    color: var(--text-muted);
    pointer-events: none;
  }
  .tag-search {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-chip);
    color: var(--text-secondary);
    font-family: var(--font-body);
    font-size: 12px;
    padding: 0.3rem 0.5rem 0.3rem 1.7rem;
    outline: none;
    transition: border-color 0.15s, background 0.15s;
  }
  .tag-search:focus {
    border-color: var(--accent-border);
    background: var(--bg-surface-hover);
  }
  .tag-search::placeholder { color: var(--text-faint); }
  .tag-empty {
    font-size: 12px;
    color: var(--text-muted);
    padding: 0.5rem 1rem;
    margin: 0;
  }
  .nav-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.35rem 1rem;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    font-family: var(--font-body);
    text-align: left;
    cursor: pointer;
    border-radius: 0;
    transition: background 0.1s, color 0.1s;
  }
  .nav-item:hover {
    background: var(--bg-surface-hover);
    color: var(--text);
  }
  .nav-item.active {
    background: var(--accent-tint);
    color: var(--accent);
  }
  .tag-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tag-count {
    font-size: 11px;
    color: var(--text-muted);
    margin-left: 0.4rem;
    flex-shrink: 0;
  }
  .nav-item.active .tag-count {
    color: var(--accent);
  }
  .panel-footer {
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .new-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    padding: 0.5rem;
    background: var(--accent-tint);
    border: 1px solid var(--accent-border);
    border-radius: var(--radius-chip);
    color: var(--accent);
    font-size: 13px;
    font-weight: 500;
    font-family: var(--font-body);
    cursor: pointer;
    transition: background 0.15s;
  }
  .new-btn:hover { background: var(--accent-tint-strong); }
  .settings-btn {
    flex-shrink: 0;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-chip);
    color: var(--text-muted);
    font-size: 15px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }
  .settings-btn:hover { border-color: var(--border-strong); color: var(--text); }
</style>
