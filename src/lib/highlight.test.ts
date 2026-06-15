import { describe, it, expect } from 'vitest';
import { detectSnippet, highlightPreviewHtml } from './highlight';

describe('detectSnippet', () => {
  it('returns text for empty content', () => {
    expect(detectSnippet('')).toEqual({ content_type: 'text', language: null });
  });

  it('detects a sudo/docker pipeline as cli', () => {
    const result = detectSnippet(
      "sudo docker logs --tail 50000 zones-module-container_01 | grep 'ERROR'",
    );
    expect(result.content_type).toBe('cli');
    expect(result.language).toBeNull();
  });

  it('detects a shell prompt line as cli', () => {
    const result = detectSnippet('$ npm install react');
    expect(result.content_type).toBe('cli');
    expect(result.language).toBeNull();
  });

  it('detects a multi-line shell script as cli', () => {
    const result = detectSnippet('cd /tmp\nmkdir build\ntar -czf out.tgz build');
    expect(result.content_type).toBe('cli');
  });

  it('detects a powershell pipeline as cli', () => {
    const result = detectSnippet('Get-Process | Where-Object { $_.CPU -gt 10 }');
    expect(result.content_type).toBe('cli');
  });

  it('detects a typescript function as code, not csharp', () => {
    const content =
      'function greet(name: string): string {\n' +
      "  const msg = 'Hello, ' + name;\n" +
      '  return msg;\n' +
      '}';
    const result = detectSnippet(content);
    expect(result.content_type).toBe('code');
    expect(['typescript', 'javascript']).toContain(result.language);
  });

  it('detects a json blob as code/json', () => {
    const result = detectSnippet('{\n  "name": "faro",\n  "version": "1.0.0"\n}');
    expect(result.content_type).toBe('code');
    expect(result.language).toBe('json');
  });

  it('detects plain prose as text', () => {
    const content =
      'This is a note about my project. It has multiple sentences and no code at all.';
    const result = detectSnippet(content);
    expect(result.content_type).toBe('text');
    expect(result.language).toBeNull();
  });
});

describe('highlightPreviewHtml', () => {
  it('escapes html when no language is given', () => {
    expect(highlightPreviewHtml('<b> & "x"', null)).toBe('&lt;b&gt; &amp; &quot;x&quot;');
  });

  it('returns hljs-classed markup for a known language', () => {
    const html = highlightPreviewHtml('const x = 1;', 'javascript');
    expect(html).toContain('hljs-');
  });

  it('falls back to escaped text for an unknown language', () => {
    expect(highlightPreviewHtml('<x>', 'cobol')).toBe('&lt;x&gt;');
  });
});
