<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { emit } from '@tauri-apps/api/event';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { createSnippet } from '$lib/api';
  import { detectSnippet } from '$lib/highlight';
  import { settings } from '$lib/stores/settings.svelte';
  import type { ContentType } from '$lib/types';

  const TITLE_MAX = 60;

  let title = $state('');
  let content = $state('');
  let contentType = $state<ContentType>('text');
  let tags = $state('');
  let systemTags = $state<string[]>([]);
  let saved = $state(false);
  let saving = $state(false);
  let saveError = $state<string | null>(null);

  let contentPrefilled = $state(false);
  let titleEdited = $state(false);

  const suggestedTitle = $derived.by(() => {
    const firstLine = content.split('\n').find((l) => l.trim().length > 0)?.trim() ?? '';
    return firstLine.length > TITLE_MAX
      ? `${firstLine.slice(0, TITLE_MAX).trimEnd()}…`
      : firstLine;
  });

  const canSave = $derived(
    !saving && ((titleEdited ? title.trim() : suggestedTitle) !== '' || content.trim() !== ''),
  );

  function onTitleInput(e: Event) {
    titleEdited = true;
    title = (e.currentTarget as HTMLInputElement).value;
  }

  function clearPrefill() {
    content = '';
    contentPrefilled = false;
    titleEdited = false;
    title = '';
    systemTags = [];
    contentType = 'text';
    document.getElementById('content-area')?.focus();
  }

  async function prefillFromClipboard() {
    if (!settings.loaded) await settings.load();
    if (!settings.clipboardPrefill) return;
    try {
      const text = await readText();
      if (text && text.trim()) {
        content = text;
        contentPrefilled = true;
        try { runDetection(); } catch { /* detection is best-effort */ }
      }
    } catch {
      /* clipboard empty or non-text — nothing to suggest */
    }
  }

  function runDetection() {
    if (!content.trim()) { systemTags = []; return; }
    const { content_type, language } = detectSnippet(content);
    contentType = content_type;
    systemTags = language ? [language] : [];
  }

  async function handleSave() {
    if (saved || !canSave) return;
    saveError = null;
    try { runDetection(); } catch { /* detection is best-effort */ }
    saving = true;
    try {
      if (!settings.loaded) await settings.load();
      const userTags = tags
        .split(',')
        .map(t => t.trim())
        .filter(Boolean)
        .map(name => ({ name, source: 'user' }));
      const sysTags = systemTags.map(name => ({ name, source: 'system' }));
      await createSnippet({
        title: (titleEdited ? title.trim() : '') || suggestedTitle || 'Untitled',
        content,
        content_type: contentType,
        tags: [...userTags, ...sysTags],
      });
      try { await emit('snippet-created', {}); } catch { /* main window may be gone */ }
      saved = true;
      if (!settings.quickNoteStayOpen) {
        await getCurrentWindow().close();
      } else {
        setTimeout(() => {
          title = '';
          content = '';
          tags = '';
          systemTags = [];
          contentType = 'text';
          contentPrefilled = false;
          titleEdited = false;
          saved = false;
          saving = false;
        }, 800);
      }
    } catch (err) {
      console.error('[quicknote] save failed:', err);
      saveError = String(err);
      saved = false;
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') getCurrentWindow().close();
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') handleSave();
    // Ctrl+Shift+X clears the clipboard suggestion; only active while one is shown.
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'x' || e.key === 'X') && contentPrefilled) {
      e.preventDefault();
      clearPrefill();
    }
  }

  onMount(() => {
    settings.load().then(() => {
      document.documentElement.style.setProperty('--font-size-editor', `${settings.fontSize}px`);
    });
    prefillFromClipboard();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if saved}
  <div class="saved-flash">
    <span>Saved</span>
  </div>
{:else}
  <div class="page">
    <input
      id="title-input"
      type="text"
      value={titleEdited ? title : suggestedTitle}
      oninput={onTitleInput}
      placeholder="Untitled snippet"
      class="title-input"
      class:suggested={!titleEdited && suggestedTitle !== ''}
    />

    <div class="field">
      <label for="type-buttons">Type</label>
      <div class="type-buttons" id="type-buttons" role="group">
        {#each (['code', 'cli', 'text', 'url'] as ContentType[]) as t (t)}
          <button
            class="type-btn"
            class:active={contentType === t}
            onclick={() => { contentType = t; }}
          >{t}</button>
        {/each}
      </div>
    </div>

    <div class="field grow">
      {#if contentPrefilled}
        <div class="prefill-chip">
          <span>Prefilled from clipboard</span>
          <button type="button" class="clear-btn" onclick={clearPrefill}>Clear (Ctrl+Shift+X)</button>
        </div>
      {/if}
      <!-- svelte-ignore a11y_autofocus -->
      <textarea
        id="content-area"
        bind:value={content}
        oninput={() => { contentPrefilled = false; }}
        onblur={runDetection}
        placeholder="Paste your snippet here…"
        class="content-area"
        class:suggested={contentPrefilled}
        spellcheck="false"
        autofocus
      ></textarea>
    </div>

    <div class="field">
      <label for="tags-input">Tags</label>
      <input
        id="tags-input"
        type="text"
        bind:value={tags}
        placeholder="rust, async, cli  (comma-separated)"
        class="tags-input"
      />
    </div>

    {#if saveError}
      <div class="save-error">{saveError}</div>
    {/if}

    <div class="actions">
      <span class="hint">Esc to discard · Ctrl+Enter to save</span>
      <button
        class="btn-primary"
        onclick={handleSave}
        disabled={!canSave}
      >
        {saving ? 'Saving…' : 'Save'}
      </button>
    </div>
  </div>
{/if}

<style>
  :global(html, body) {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .saved-flash {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
  }

  .saved-flash span {
    font-size: 2rem;
    font-weight: 600;
    color: var(--accent);
    font-family: var(--font-body);
  }

  .page {
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 1.25rem;
    gap: 0.75rem;
    overflow: hidden;
    background: var(--bg);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .field.grow {
    flex: 1;
    min-height: 0;
  }

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
  
  .title-input.suggested:not(:focus) { color: var(--text-muted); }

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

  .tags-input:focus {
    border-color: var(--accent-border);
  }

  .type-buttons {
    display: flex;
    gap: 0.35rem;
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

  .content-area {
    flex: 1;
    background: var(--bg-editor);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: var(--font-size-editor, 13px);
    line-height: 1.65;
    padding: 0.6rem;
    resize: none;
    outline: none;
    width: 100%;
    height: 100%;
  }

  .content-area:focus {
    border-color: var(--accent-border);
  }

  /* Clipboard-seeded content: subtle accent edge until the user edits it. */
  .content-area.suggested {
    border-color: var(--accent-border);
    border-left-width: 3px;
    background: var(--accent-tint);
  }

  .prefill-chip {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-body);
    font-size: 11px;
    color: var(--accent);
    margin-bottom: -0.05rem;
  }

  .clear-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--font-body);
    font-size: 11px;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }

  .clear-btn:hover { color: var(--text); }

  .save-error {
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid var(--danger);
    border-radius: var(--radius-control);
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 0.4rem 0.6rem;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .hint {
    font-size: 11px;
    color: var(--text-faint);
    font-family: var(--font-body);
    margin-right: auto;
  }

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

  .btn-primary:hover:not(:disabled) {
    opacity: 0.85;
  }

  .btn-primary:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
