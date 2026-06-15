<script lang="ts">
  import { settings } from '$lib/stores/settings.svelte';

  let { open, onclose }: { open: boolean; onclose: () => void } = $props();

  let draftHotkey = $state(settings.hotkey);
  let draftFontSize = $state(settings.fontSize);
  let draftStayOpen = $state(settings.quickNoteStayOpen);
  let draftTheme = $state(settings.theme);
  let saving = $state(false);
  let error = $state<string | null>(null);

  let overlayEl = $state<HTMLDivElement | null>(null);

  // Sync draft from store when modal opens; focus overlay for keyboard handling
  $effect(() => {
    if (open) {
      draftHotkey = settings.hotkey;
      draftFontSize = settings.fontSize;
      draftStayOpen = settings.quickNoteStayOpen;
      draftTheme = settings.theme;
      error = null;
      setTimeout(() => overlayEl?.focus(), 0);
    }
  });

  // Live font-size preview
  $effect(() => {
    if (open) {
      document.documentElement.style.setProperty('--font-size-editor', `${draftFontSize}px`);
    }
  });

  // Live theme preview
  $effect(() => {
    if (open) {
      document.documentElement.setAttribute('data-theme', draftTheme);
    }
  });

  function handleCancel() {
    document.documentElement.style.setProperty('--font-size-editor', `${settings.fontSize}px`);
    document.documentElement.setAttribute('data-theme', settings.theme);
    onclose();
  }

  async function handleSave() {
    saving = true;
    error = null;
    try {
      await settings.save({
        hotkey: draftHotkey,
        fontSize: draftFontSize,
        quickNoteStayOpen: draftStayOpen,
        theme: draftTheme,
      });
      onclose();
    } catch (e) {
      error = String(e);
      document.documentElement.style.setProperty('--font-size-editor', `${settings.fontSize}px`);
      document.documentElement.setAttribute('data-theme', settings.theme);
      draftFontSize = settings.fontSize;
      draftTheme = settings.theme;
    } finally {
      saving = false;
    }
  }
</script>

{#if open}
  <div
    class="overlay"
    role="dialog"
    aria-modal="true"
    aria-label="Settings"
    tabindex="-1"
    bind:this={overlayEl}
    onkeydown={(e) => { if (e.key === 'Escape') handleCancel(); }}
  >
    <div class="modal">
      <div class="modal-header">
        <h2 class="modal-title">Settings</h2>
      </div>

      <div class="modal-body">
        <div class="setting-row">
          <label for="hotkey-input" class="setting-label">Global hotkey</label>
          <input
            id="hotkey-input"
            type="text"
            bind:value={draftHotkey}
            class="setting-input"
            placeholder="e.g. Ctrl+Shift+Space"
          />
          {#if error}
            <span class="field-error">{error}</span>
          {/if}
        </div>

        <div class="setting-row">
          <label for="font-size-input" class="setting-label">Editor font size</label>
          <div class="font-size-control">
            <input
              id="font-size-input"
              type="number"
              bind:value={draftFontSize}
              min="10"
              max="24"
              step="1"
              class="setting-input number-input"
            />
            <span class="font-size-unit">px</span>
          </div>
        </div>

        <div class="setting-row">
          <label for="stay-open-toggle" class="setting-label">Quick Note stay open after save</label>
          <input
            id="stay-open-toggle"
            type="checkbox"
            bind:checked={draftStayOpen}
            class="setting-checkbox"
          />
        </div>

        <div class="setting-row">
          <span class="setting-label">Theme</span>
          <div class="theme-buttons" role="group" aria-label="Theme">
            {#each (['dark', 'light', 'nord'] as const) as t (t)}
              <button
                type="button"
                class="theme-btn"
                class:active={draftTheme === t}
                onclick={() => (draftTheme = t)}
              >{t}</button>
            {/each}
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" onclick={handleCancel} disabled={saving}>Cancel</button>
        <button class="btn-primary" onclick={handleSave} disabled={saving}>
          {saving ? 'Saving…' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .modal {
    background: var(--bg-modal);
    border: 1px solid var(--border);
    border-radius: 8px;
    width: 420px;
    max-width: calc(100vw - 2rem);
    display: flex;
    flex-direction: column;
  }
  .modal-header {
    padding: 1rem 1.25rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }
  .modal-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }
  .modal-body {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .setting-row {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .setting-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .setting-input {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
    font-family: var(--font-body);
    font-size: 13px;
    padding: 0.4rem 0.6rem;
    outline: none;
    width: 100%;
  }
  .setting-input:focus { border-color: var(--accent-border); }
  .number-input { width: 80px; }
  .font-size-control { display: flex; align-items: center; gap: 0.4rem; }
  .font-size-unit { font-size: 12px; color: var(--text-muted); }
  .setting-checkbox { width: 16px; height: 16px; accent-color: var(--accent); cursor: pointer; }
  .theme-buttons { display: flex; gap: 0.35rem; }
  .theme-btn {
    flex: 1;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-chip);
    color: var(--text-muted);
    font-size: 12px;
    font-family: var(--font-body);
    text-transform: capitalize;
    padding: 0.3rem 0.5rem;
    cursor: pointer;
  }
  .theme-btn.active {
    background: var(--accent-tint);
    border-color: var(--accent-border);
    color: var(--accent);
  }
  .field-error { font-size: 11px; color: var(--danger); }
  .modal-footer {
    padding: 0.75rem 1.25rem;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .btn-primary {
    background: var(--accent-strong);
    border: none;
    border-radius: 4px;
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
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 13px;
    font-family: var(--font-body);
    padding: 0.4rem 0.8rem;
    cursor: pointer;
  }
  .btn-secondary:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text); }
  .btn-secondary:disabled { opacity: 0.35; cursor: default; }
</style>
