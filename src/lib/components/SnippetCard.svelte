<script lang="ts">
  import { Code2, Terminal, FileText, Link, Copy, Check } from 'lucide-svelte';
  import type { Snippet } from '$lib/types';
  import { copyToClipboard } from '$lib/api';
  import { snippets as store } from '$lib/stores/snippets.svelte';
  import { highlightPreviewHtml } from '$lib/highlight';

  let { snippet, selected = false, onselect }: {
    snippet: Snippet;
    selected?: boolean;
    onselect?: (id: number) => void;
  } = $props();

  const previewLines = $derived(snippet.content.split('\n').slice(0, 2).join('\n'));

  const previewLang = $derived(
    snippet.content_type === 'code'
      ? (snippet.tags.find(t => t.source === 'system')?.name ?? null)
      : snippet.content_type === 'cli'
        ? 'bash'
        : null,
  );
  const previewHtml = $derived(highlightPreviewHtml(previewLines, previewLang));
  const previewFont = $derived(
    snippet.content_type === 'text' ? 'var(--font-body)' : 'var(--font-mono)',
  );

  let copied = $state(false);

  async function copyContent(e: MouseEvent) {
    e.stopPropagation();
    try {
      await copyToClipboard(snippet.content);
      copied = true;
      setTimeout(() => { copied = false; }, 1500);
      store.recordCopy(snippet.id);
    } catch { /* clipboard unavailable */ }
  }
</script>

<div
  class="card"
  class:selected
  role="button"
  tabindex="0"
  data-snippet-id={snippet.id}
  onclick={() => onselect?.(snippet.id)}
  onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), onselect?.(snippet.id))}
>
  <div class="header">
    <span class="type-icon {snippet.content_type}">
      {#if snippet.content_type === 'code'}
        <Code2 size={13} strokeWidth={2} />
      {:else if snippet.content_type === 'cli'}
        <Terminal size={13} strokeWidth={2} />
      {:else if snippet.content_type === 'url'}
        <Link size={13} strokeWidth={2} />
      {:else}
        <FileText size={13} strokeWidth={2} />
      {/if}
    </span>
    <span class="title">{snippet.title || 'Untitled'}</span>
    {#if snippet.copy_count > 0}
      <span class="copy-count">{snippet.copy_count}</span>
    {/if}
    <button class="copy-btn" onclick={copyContent}>
      {#if copied}
        <Check size={12} strokeWidth={2.5} />
      {:else}
        <Copy size={12} strokeWidth={2} />
      {/if}
    </button>
  </div>
  <pre class="preview" style:font-family={previewFont}>{@html previewHtml}</pre>
  {#if snippet.tags.some(t => t.source !== 'suppressed')}
    <div class="tags">
      {#each snippet.tags.filter(t => t.source !== 'suppressed') as tag (tag.id)}
        <span class="tag" class:system={tag.source === 'system'}>{tag.name}</span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .card {
    --icon-col: calc(20px + 0.5rem);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: var(--card-pad);
    cursor: pointer;
    user-select: none;
    transition: background 0.15s, border-color 0.15s, box-shadow 0.15s;
  }
  .card:hover {
    background: var(--bg-card-hover);
    border-color: var(--border-strong);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  }
  .card.selected {
    background: var(--accent-tint);
    border-color: var(--accent-border);
    box-shadow: 0 2px 8px var(--accent-tint);
  }

  .header {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    margin-bottom: 0.3rem;
  }
  .type-icon {
    display: flex;
    align-items: center;
    width: 20px;
    flex-shrink: 0;
    justify-content: center;
  }
  .type-icon.code { color: var(--accent); }
  .type-icon.cli { color: var(--success); }
  .type-icon.text { color: var(--text-muted); }
  .type-icon.url { color: var(--link); }
  .title {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .copy-count {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }
  .copy-btn {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--border);
    color: var(--text-muted);
    width: 22px;
    height: 22px;
    border-radius: var(--radius-chip);
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.1s, background 0.1s, color 0.1s;
  }
  .card:hover .copy-btn,
  .card.selected .copy-btn,
  .copy-btn:focus { opacity: 1; }
  .copy-btn:hover { background: var(--bg-surface-hover); color: var(--text); }

  .preview {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0 0 0.35rem var(--icon-col);
    background: var(--bg-code);
    border: 1px solid var(--preview-border);
    border-radius: var(--radius-control);
    padding: var(--preview-pad);
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    margin-left: var(--icon-col);
  }
  .tag {
    font-size: 11px;
    padding: 1px 6px;
    border-radius: var(--radius-chip);
    background: var(--accent-tint);
    color: var(--accent);
  }
  .tag.system {
    background: var(--bg-surface-hover);
    color: var(--text-faint);
    font-style: italic;
  }
</style>
