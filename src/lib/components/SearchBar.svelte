<script lang="ts">
  import { Search, X } from 'lucide-svelte';

  let { value, onchange, searchInput = $bindable() }: {
    value: string;
    onchange: (q: string) => void;
    searchInput?: HTMLInputElement;
  } = $props();

  let inputValue = $state(value);
  let timer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => { inputValue = value; });
  $effect(() => { return () => { if (timer) clearTimeout(timer); }; });

  function handleInput(e: Event) {
    const v = (e.target as HTMLInputElement).value;
    inputValue = v;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => onchange(v), 250);
  }

  function clear() {
    inputValue = '';
    if (timer) clearTimeout(timer);
    onchange('');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { clear(); (e.target as HTMLInputElement).blur(); }
  }
</script>

<div class="search-bar">
  <span class="icon-search"><Search size={14} strokeWidth={2} /></span>
  <input
    type="search"
    placeholder="Search snippets…"
    value={inputValue}
    oninput={handleInput}
    onkeydown={handleKeydown}
    bind:this={searchInput}
  />
  {#if inputValue}
    <button class="clear-btn" onclick={clear}><X size={14} strokeWidth={2} /></button>
  {/if}
</div>

<style>
  .search-bar {
    position: relative;
    display: flex;
    align-items: center;
  }
  .icon-search {
    position: absolute;
    left: 0.6rem;
    display: flex;
    align-items: center;
    color: var(--text-muted);
    pointer-events: none;
    flex-shrink: 0;
  }
  input {
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 14px;
    padding: 0.4rem 2rem 0.4rem 2.1rem;
    outline: none;
    transition: border-color 0.15s, background 0.15s;
  }
  input:focus {
    border-color: var(--accent-border);
    background: var(--bg-surface-hover);
  }
  input::placeholder { color: var(--text-muted); }
  input::-webkit-search-cancel-button { display: none; }
  .clear-btn {
    position: absolute;
    right: 0.4rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 0.15rem;
    cursor: pointer;
    border-radius: var(--radius-chip);
    transition: color 0.1s;
  }
  .clear-btn:hover { color: var(--text); }
</style>
