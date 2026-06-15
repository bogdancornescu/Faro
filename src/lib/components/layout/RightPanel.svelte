<script lang="ts">
  import SnippetDetail from '$lib/components/SnippetDetail.svelte';
  import type { Snippet, CreateSnippetInput, UpdateSnippetInput } from '$lib/types';

  let { snippet = null, mode, onSave, onCancel, onDelete, dirty = $bindable(false) }: {
    snippet?: Snippet | null;
    mode: 'idle' | 'creating' | 'editing';
    onSave: (input: CreateSnippetInput | UpdateSnippetInput) => void;
    onCancel?: () => void;
    onDelete?: () => void;
    dirty?: boolean;
  } = $props();
</script>

<div class="right-panel">
  <SnippetDetail {snippet} {mode} {onSave} {onCancel} {onDelete} bind:dirty />
</div>

<style>
  .right-panel { height: 100%; overflow: hidden; }
</style>
