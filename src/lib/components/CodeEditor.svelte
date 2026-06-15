<script lang="ts">
  import { untrack } from 'svelte';
  import { EditorView, keymap } from '@codemirror/view';
  import { EditorState, Compartment } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import type { ContentType } from '$lib/types';
  import { settings } from '$lib/stores/settings.svelte';
  import { fontTheme, highlightExtension, resolveLanguage, darkFlag } from '$lib/editor';

  let { value = $bindable(''), contentType, language = null, onblur }: {
    value: string;
    contentType: ContentType;
    language?: string | null;
    onblur?: () => void;
  } = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  const fontCompartment = new Compartment();
  const langCompartment = new Compartment();
  const themeCompartment = new Compartment();

  // Mount the editor once the container exists. value/contentType are read
  // untracked so typing or type changes don't tear down and rebuild the editor —
  // those updates are handled by the dedicated effects below.
  $effect(() => {
    if (!container) return;
    const initialDoc = untrack(() => value);
    const initialType = untrack(() => contentType);
    const state = EditorState.create({
      doc: initialDoc,
      extensions: [
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        EditorView.lineWrapping,
        EditorState.tabSize.of(2),
        highlightExtension(),
        fontCompartment.of(fontTheme(initialType)),
        themeCompartment.of(darkFlag(settings.theme)),
        langCompartment.of([]),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) value = u.state.doc.toString();
        }),
        EditorView.domEventHandlers({
          blur: () => {
            onblur?.();
            return false;
          },
        }),
      ],
    });
    view = new EditorView({ state, parent: container });
    return () => {
      view?.destroy();
      view = null;
    };
  });

  // Push external value changes into the editor without clobbering edits.
  $effect(() => {
    const v = value;
    if (view && v !== view.state.doc.toString()) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } });
    }
  });

  // Reconfigure font when the content type changes.
  $effect(() => {
    const ct = contentType;
    if (view) view.dispatch({ effects: fontCompartment.reconfigure(fontTheme(ct)) });
  });

  // Reconfigure the light/dark base flag when the theme changes.
  $effect(() => {
    const theme = settings.theme;
    if (view) view.dispatch({ effects: themeCompartment.reconfigure(darkFlag(theme)) });
  });

  // Reconfigure grammar (async) when type/language change.
  $effect(() => {
    const ct = contentType;
    const lang = language;
    let cancelled = false;
    resolveLanguage(ct, lang).then((ext) => {
      if (!cancelled && view) {
        view.dispatch({ effects: langCompartment.reconfigure(ext ?? []) });
      }
    });
    return () => {
      cancelled = true;
    };
  });
</script>

<div class="editor" bind:this={container}></div>

<style>
  .editor {
    flex: 1;
    min-height: 0;
    height: 100%;
    background: var(--bg-editor);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    overflow: hidden;
  }
  .editor :global(.cm-editor) {
    height: 100%;
  }
  .editor :global(.cm-editor.cm-focused) {
    border-color: var(--accent-border);
  }
</style>
