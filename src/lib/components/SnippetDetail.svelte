<script lang="ts">
  import type { Snippet, ContentType, CreateSnippetInput, UpdateSnippetInput } from '$lib/types';
  import { detectSnippet } from '$lib/highlight';
  import CodeEditor from './CodeEditor.svelte';
  import { LANGUAGE_OPTIONS } from '$lib/languages';
  import { Copy, Check } from 'lucide-svelte';
  import { copyToClipboard } from '$lib/api';
  import { snippets as store } from '$lib/stores/snippets.svelte';

  let { snippet = null, mode, onSave, onCancel, onDelete, dirty = $bindable(false) }: {
    snippet?: Snippet | null;
    mode: 'idle' | 'creating' | 'editing';
    onSave: (input: CreateSnippetInput | UpdateSnippetInput) => void;
    onCancel?: () => void;
    onDelete?: () => void;
    dirty?: boolean;
  } = $props();

  let draftTitle = $state('');
  let draftContent = $state('');
  let draftType = $state<ContentType>('text');
  let draftTags = $state('');
  let draftSystemTags = $state<string[]>([]);
  let draftSuppressedTags = $state<string[]>([]);
  let typeManuallySet = $state(false);
  let systemTagsManuallySet = $state(false);
  let detailCopied = $state(false);

  async function handleDetailCopy() {
    try {
      await copyToClipboard(draftContent);
      detailCopied = true;
      setTimeout(() => { detailCopied = false; }, 1500);
      if (snippet) store.recordCopy(snippet.id);
    } catch { /* clipboard unavailable */ }
  }

  // Re-initialize draft whenever mode or snippet changes.
  $effect(() => {
    if (mode === 'editing' && snippet) {
      draftTitle = snippet.title;
      draftContent = snippet.content;
      draftType = snippet.content_type;
      draftTags = snippet.tags
        .filter(t => t.source === 'user')
        .map(t => t.name)
        .join(', ');
      draftSystemTags = snippet.tags
        .filter(t => t.source === 'system')
        .map(t => t.name);
      draftSuppressedTags = snippet.tags
        .filter(t => t.source === 'suppressed')
        .map(t => t.name);
      typeManuallySet = false;
      systemTagsManuallySet = false;
      dirty = false;
    } else if (mode === 'creating') {
      draftTitle = '';
      draftContent = '';
      draftType = 'text';
      draftTags = '';
      draftSystemTags = [];
      draftSuppressedTags = [];
      typeManuallySet = false;
      systemTagsManuallySet = false;
      dirty = false;
    }
  });

  function removeSystemTag(tag: string) {
    draftSystemTags = draftSystemTags.filter(t => t !== tag);
    if (!draftSuppressedTags.includes(tag)) {
      draftSuppressedTags = [...draftSuppressedTags, tag];
    }
    systemTagsManuallySet = true;
    dirty = true;
  }

  const currentLanguage = $derived(draftSystemTags[0] ?? null);

  function setLanguage(lang: string) {
    draftSystemTags = lang ? [lang] : [];
    systemTagsManuallySet = true;
    dirty = true;
  }

  $effect(() => {
    const content = draftContent;
    if (mode === 'editing' && snippet && content !== snippet.content) dirty = true;
    if (mode === 'creating' && content.trim()) dirty = true;
  });

  function runDetection() {
    if (!draftContent.trim()) {
      if (!systemTagsManuallySet && draftSystemTags.length > 0) dirty = true;
      if (!systemTagsManuallySet) draftSystemTags = [];
      return;
    }
    const { content_type, language } = detectSnippet(draftContent);
    const newSystemTags = (language ? [language] : [])
      .filter(t => !draftSuppressedTags.includes(t));
    if (!typeManuallySet) {
      if (content_type !== draftType) dirty = true;
      draftType = content_type;
    }
    if (!systemTagsManuallySet) {
      if (newSystemTags.join() !== draftSystemTags.join()) dirty = true;
      draftSystemTags = newSystemTags;
    }
  }

  function handleSave() {
    runDetection();
    const userTags = draftTags
      .split(',')
      .map(t => t.trim())
      .filter(Boolean)
      .map(name => ({ name, source: 'user' }));
    const sysTags = draftSystemTags.map(name => ({ name, source: 'system' }));
    const suppressedTags = draftSuppressedTags.map(name => ({ name, source: 'suppressed' }));
    const tags = [...userTags, ...sysTags, ...suppressedTags];

    if (mode === 'editing' && snippet) {
      onSave({
        id: snippet.id,
        title: draftTitle,
        content: draftContent,
        content_type: draftType,
        tags,
      } as UpdateSnippetInput);
    } else {
      onSave({
        title: draftTitle,
        content: draftContent,
        content_type: draftType,
        tags,
      } as CreateSnippetInput);
    }
  }
</script>

{#if mode === 'idle'}
  <div class="placeholder">
    <span>Select a snippet or create a new one</span>
  </div>
{:else}
  <div class="detail" onkeydown={(e) => { if (e.ctrlKey && e.key === 'Enter') { e.preventDefault(); handleSave(); } }}>
    <input
      id="title-input"
      type="text"
      bind:value={draftTitle}
      oninput={() => (dirty = true)}
      placeholder="Untitled snippet"
      class="title-input"
    />

    <div class="field">
      <label for="type-buttons">Type</label>
      <div class="type-row">
        <div class="type-buttons" id="type-buttons" role="group">
          {#each (['code', 'cli', 'text', 'url'] as ContentType[]) as t (t)}
            <button
              class="type-btn"
              class:active={draftType === t}
              onclick={() => { draftType = t; typeManuallySet = true; dirty = true; }}
            >{t}</button>
          {/each}
        </div>
        {#if draftType === 'code'}
          <select
            class="lang-select"
            value={currentLanguage ?? ''}
            onchange={(e) => setLanguage((e.currentTarget as HTMLSelectElement).value)}
            aria-label="Language"
          >
            <option value="">auto</option>
            {#each LANGUAGE_OPTIONS as opt (opt.value)}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        {/if}
      </div>
    </div>

    <div class="field grow">
      <div class="content-label-row">
        <label>Content</label>
        {#if mode === 'editing' && snippet}
          <button class="content-copy-btn" onclick={handleDetailCopy}>
            {#if detailCopied}
              <Check size={12} strokeWidth={2.5} />
              <span>Copied</span>
            {:else}
              <Copy size={12} strokeWidth={2} />
              <span>Copy</span>
            {/if}
          </button>
        {/if}
      </div>
      <CodeEditor
        bind:value={draftContent}
        contentType={draftType}
        language={currentLanguage}
        onblur={runDetection}
      />
    </div>

    <div class="field">
      <label for="tags-input">Tags</label>
      {#if draftSystemTags.length > 0}
        <div class="system-tags">
          {#each draftSystemTags as tag (tag)}
            <span class="system-tag">
              {tag}
              <button class="system-tag-remove" onclick={() => removeSystemTag(tag)} title="Remove">×</button>
            </span>
          {/each}
        </div>
      {/if}
      <input
        id="tags-input"
        type="text"
        bind:value={draftTags}
        oninput={() => (dirty = true)}
        placeholder="rust, async, cli  (comma-separated)"
        class="tags-input"
      />
    </div>

    <div class="actions">
      <div class="actions-left">
        {#if mode === 'editing' && onDelete}
          <button class="btn-danger" onclick={onDelete}>Delete</button>
        {/if}
      </div>
      <div class="actions-right">
        {#if onCancel}
          <button class="btn-secondary" onclick={onCancel}>Cancel</button>
        {/if}
        <button
          class="btn-primary"
          onclick={handleSave}
          disabled={mode === 'editing' ? !dirty : !draftTitle.trim() && !draftContent.trim()}
        >
          {mode === 'creating' ? 'Create' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .placeholder {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
    font-size: 13px;
  }
  .detail {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: var(--panel-pad);
    gap: 0.75rem;
    overflow: hidden;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .field.grow { flex: 1; min-height: 0; }
  label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
  }
  .title-input {
    background: none;
    border: none;
    border-bottom: 1px solid transparent;
    border-radius: 0;
    color: var(--text);
    font-family: var(--font-body);
    font-size: 18px;
    font-weight: 600;
    padding: 0.1rem 0;
    outline: none;
    width: 100%;
    transition: border-color 0.15s;
    margin-bottom: 0.25rem;
  }
  .title-input:hover:not(:focus) { border-bottom-color: var(--border); }
  .title-input:focus { border-bottom-color: var(--accent-border); }
  .title-input::placeholder { color: var(--text-faint); font-weight: 400; }
  .content-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .content-copy-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-chip);
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-body);
    padding: 0.15rem 0.5rem;
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .content-copy-btn:hover {
    background: var(--bg-surface-hover);
    color: var(--text);
    border-color: var(--border-strong);
  }
  .system-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 0.3rem;
  }
  .system-tag {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    background: var(--accent-tint);
    border: 1px solid var(--accent-border);
    border-radius: var(--radius-chip);
    color: var(--accent);
    font-size: 11px;
    padding: 0.1rem 0.35rem 0.1rem 0.45rem;
  }
  .system-tag-remove {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 13px;
    line-height: 1;
    opacity: 0.6;
    padding: 0;
  }
  .system-tag-remove:hover { opacity: 1; }
  .tags-input {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 14px;
    padding: 0.4rem 0.6rem;
    outline: none;
    width: 100%;
  }
  .tags-input:focus { border-color: var(--accent-border); }
  .type-row { display: flex; align-items: center; gap: 0.5rem; }
  .type-buttons { display: flex; gap: 0.35rem; }
  .lang-select {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-chip);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 12px;
    padding: 0.2rem 0.4rem;
    outline: none;
    cursor: pointer;
  }
  .lang-select:hover { border-color: var(--border-strong); }
  .lang-select:focus { border-color: var(--accent-border); }
  .lang-select option {
    background-color: var(--bg-surface);
    color: var(--text);
  }
  .type-btn {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-chip);
    color: var(--text-muted);
    font-size: 12px;
    font-family: var(--font-body);
    padding: 0.2rem 0.75rem;
    cursor: pointer;
  }
  .type-btn.active {
    background: var(--accent-tint);
    border-color: var(--accent-border);
    color: var(--accent);
  }
  .actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .actions-left { display: flex; }
  .actions-right { display: flex; gap: 0.5rem; }
  .btn-danger {
    background: none;
    border: 1px solid var(--danger);
    border-radius: var(--radius-control);
    color: var(--danger);
    font-size: 13px;
    font-family: var(--font-body);
    padding: 0.4rem 0.8rem;
    cursor: pointer;
  }
  .btn-danger:hover { border-color: var(--danger); color: var(--danger); }
  .btn-primary {
    background: var(--accent-strong);
    border: none;
    border-radius: var(--radius-control);
    color: var(--accent-contrast);
    font-size: 13px;
    font-weight: 500;
    font-family: var(--font-body);
    padding: 0.4rem 1.1rem;
    cursor: pointer;
    transition: opacity 0.1s;
  }
  .btn-primary:hover:not(:disabled) { opacity: 0.85; }
  .btn-primary:disabled { opacity: 0.35; cursor: default; }
  .btn-secondary {
    background: none;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    color: var(--text-muted);
    font-size: 13px;
    font-family: var(--font-body);
    padding: 0.4rem 0.8rem;
    cursor: pointer;
  }
  .btn-secondary:hover { border-color: var(--border-strong); color: var(--text); }
</style>
