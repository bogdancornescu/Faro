/**
 * Canonical language ids are highlight.js language names. These are what
 * detectSnippet() stores as the system tag and what hljs.highlight() expects.
 * cmLanguageName() maps them to @codemirror/language-data display names for
 * grammar loading.
 */

/** highlight.js language names that detection is allowed to choose from. */
export const SUBSET = [
  'typescript', 'javascript', 'python', 'rust', 'go', 'java',
  'c', 'cpp', 'csharp', 'ruby', 'php', 'sql', 'json', 'yaml',
  'xml', 'css', 'kotlin', 'swift',
] as const;

/** hljs name -> @codemirror/language-data display name. */
const CM_NAMES: Record<string, string> = {
  typescript: 'TypeScript',
  javascript: 'JavaScript',
  python: 'Python',
  rust: 'Rust',
  go: 'Go',
  java: 'Java',
  c: 'C',
  cpp: 'C++',
  csharp: 'C#',
  ruby: 'Ruby',
  php: 'PHP',
  sql: 'SQL',
  json: 'JSON',
  yaml: 'YAML',
  xml: 'HTML',
  css: 'CSS',
  kotlin: 'Kotlin',
  swift: 'Swift',
};

/** Returns the CodeMirror display name for an hljs language, or null. */
export function cmLanguageName(hljsName: string): string | null {
  return CM_NAMES[hljsName] ?? null;
}

/** Options for the language override dropdown: { value: hljsName, label }. */
export const LANGUAGE_OPTIONS = SUBSET.map((value) => ({
  value,
  label: CM_NAMES[value] ?? value,
}));
