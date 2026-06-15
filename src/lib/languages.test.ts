import { describe, it, expect } from 'vitest';
import { SUBSET, cmLanguageName, LANGUAGE_OPTIONS } from './languages';

describe('languages', () => {
  it('SUBSET contains the common languages', () => {
    expect(SUBSET).toContain('typescript');
    expect(SUBSET).toContain('csharp');
    expect(SUBSET).toContain('json');
  });

  it('cmLanguageName maps hljs names to CodeMirror display names', () => {
    expect(cmLanguageName('typescript')).toBe('TypeScript');
    expect(cmLanguageName('cpp')).toBe('C++');
    expect(cmLanguageName('csharp')).toBe('C#');
  });

  it('cmLanguageName returns null for unknown languages', () => {
    expect(cmLanguageName('cobol')).toBeNull();
  });

  it('LANGUAGE_OPTIONS pairs hljs value with display label', () => {
    const ts = LANGUAGE_OPTIONS.find(o => o.value === 'typescript');
    expect(ts?.label).toBe('TypeScript');
  });
});
