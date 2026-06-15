import { EditorView } from '@codemirror/view';
import {
  StreamLanguage,
  HighlightStyle,
  syntaxHighlighting,
  LanguageDescription,
} from '@codemirror/language';
import { languages } from '@codemirror/language-data';
import { tags as t } from '@lezer/highlight';
import type { Extension } from '@codemirror/state';
import type { ContentType } from './types';
import { cmLanguageName } from './languages';

/** CSS font-family variable for a content type. */
export function editorFontFamily(contentType: ContentType): string {
  return contentType === 'text' ? 'var(--font-body)' : 'var(--font-mono)';
}

/** Editor base theme: transparent background, font from CSS vars. */
export function fontTheme(contentType: ContentType): Extension {
  const family = editorFontFamily(contentType);
  return EditorView.theme(
    {
      '&': {
        backgroundColor: 'transparent',
        color: 'var(--text)',
        fontSize: 'var(--font-size-editor, 14px)',
        height: '100%',
      },
      '.cm-content': { fontFamily: family, lineHeight: '1.65', padding: '0.6rem' },
      '.cm-gutters': { display: 'none' },
      '.cm-scroller': { fontFamily: family, overflow: 'auto' },
      '&.cm-focused': { outline: 'none' },
      '.cm-cursor': { borderLeftColor: 'var(--accent)' },
      '.cm-selectionBackground, &.cm-focused .cm-selectionBackground': {
        backgroundColor: 'var(--cm-selection)',
      },
    },
  );
}

/** Base light/dark flag for CodeMirror built-ins, driven by the active theme. */
export function darkFlag(theme: string): Extension {
  return EditorView.theme({}, { dark: theme !== 'light' });
}

const faroHighlight = HighlightStyle.define([
  { tag: t.keyword, color: 'var(--syntax-keyword)' },
  { tag: [t.string, t.special(t.string)], color: 'var(--syntax-string)' },
  { tag: t.comment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: [t.number, t.bool, t.null], color: 'var(--syntax-number)' },
  { tag: [t.function(t.variableName), t.function(t.propertyName)], color: 'var(--syntax-func)' },
  { tag: [t.typeName, t.className], color: 'var(--syntax-type)' },
  { tag: [t.operator, t.punctuation], color: 'var(--syntax-operator)' },
  { tag: t.propertyName, color: 'var(--syntax-attr)' },
  { tag: [t.tagName], color: 'var(--syntax-tag)' },
]);

/** Syntax-coloring extension; colours resolve from the active theme's CSS vars. */
export function highlightExtension(): Extension {
  return syntaxHighlighting(faroHighlight);
}

/**
 * Resolves a CodeMirror language extension for the given content type/language.
 * Loads grammars lazily; returns null when there is nothing to highlight.
 */
export async function resolveLanguage(
  contentType: ContentType,
  language: string | null,
): Promise<Extension | null> {
  if (contentType === 'cli') {
    const { shell } = await import('@codemirror/legacy-modes/mode/shell');
    return StreamLanguage.define(shell);
  }
  if (contentType === 'code' && language) {
    const name = cmLanguageName(language);
    const desc = name ? LanguageDescription.matchLanguageName(languages, name, true) : null;
    if (desc) {
      const support = await desc.load();
      return support;
    }
  }
  return null;
}
