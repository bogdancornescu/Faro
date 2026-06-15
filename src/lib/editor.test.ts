import { describe, it, expect } from 'vitest';
import { editorFontFamily } from './editor';

describe('editorFontFamily', () => {
  it('uses the body font for text snippets', () => {
    expect(editorFontFamily('text')).toBe('var(--font-body)');
  });

  it('uses the mono font for code snippets', () => {
    expect(editorFontFamily('code')).toBe('var(--font-mono)');
  });

  it('uses the mono font for cli snippets', () => {
    expect(editorFontFamily('cli')).toBe('var(--font-mono)');
  });
});
