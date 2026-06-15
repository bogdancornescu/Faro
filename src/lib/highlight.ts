import hljs from 'highlight.js/lib/common';
import type { ContentType } from './types';
import { SUBSET } from './languages';

// ── CLI recognition ──────────────────────────────────────────────────────────

const SHELL_COMMANDS = new Set([
  'sudo', 'docker', 'kubectl', 'git', 'npm', 'pnpm', 'yarn', 'node', 'cargo',
  'python', 'python3', 'pip', 'pip3', 'ssh', 'scp', 'rsync', 'curl', 'wget',
  'cat', 'ls', 'cd', 'pwd', 'grep', 'egrep', 'awk', 'sed', 'rm', 'cp', 'mv',
  'mkdir', 'touch', 'chmod', 'chown', 'tar', 'zip', 'unzip', 'apt', 'apt-get',
  'yum', 'dnf', 'brew', 'systemctl', 'journalctl', 'service', 'make', 'cmake',
  'ping', 'nslookup', 'dig', 'traceroute', 'ps', 'top', 'kill', 'killall',
  'export', 'source', 'echo', 'printf', 'find', 'xargs', 'tee', 'head', 'tail',
  'df', 'du', 'free', 'uname', 'whoami', 'ifconfig', 'ip', 'netstat', 'ss',
]);

// A leading shell prompt: "$ ", "PS C:\x>", "C:\x>". Bare "#"/">" are excluded
// to avoid matching comments or prose.
const PROMPT_RE = /^\s*(\$|PS[^>\n]*>|[A-Za-z]:\\[^>\n]*>)\s+/;
// Pipes and logical/redirection operators. A bare "-flag" is intentionally NOT
// matched here: it produced false-positive cli detection on prose that mentions
// command flags.
const OPERATOR_RE = /\s\|\s|&&|\|\||>>|2>&1/;

function lineLooksLikeCli(line: string): boolean {
  const trimmed = line.trim();
  if (!trimmed) return false;
  if (PROMPT_RE.test(line)) return true;
  const first = trimmed.split(/\s+/)[0] ?? '';
  if (SHELL_COMMANDS.has(first)) return true;
  if (OPERATOR_RE.test(` ${trimmed} `) && /[A-Za-z]/.test(first)) return true;
  return false;
}

/** True when the majority of non-empty lines look like shell commands. */
function looksLikeCli(content: string): boolean {
  const lines = content.split('\n').filter((l) => l.trim().length > 0);
  if (lines.length === 0) return false;
  const hits = lines.filter(lineLooksLikeCli).length;
  return hits / lines.length >= 0.5;
}

// ── Language detection ───────────────────────────────────────────────────────

const RELEVANCE_THRESHOLD = 4;

function hasTsJsMarkers(content: string): boolean {
  return /\b(const|let|export)\b|=>|import\s+.*\sfrom\s/.test(content);
}

// highlightAuto leans toward C#/Java on short TS/JS; nudge it back when the
// content carries obvious TS/JS markers.
function tieBreak(content: string, language: string): string {
  if ((language === 'csharp' || language === 'java') && hasTsJsMarkers(content)) {
    const tsShaped = /:\s*(string|number|boolean)\b|\binterface\s|\btype\s+\w+\s*=/.test(content);
    return tsShaped ? 'typescript' : 'javascript';
  }
  return language;
}

export function detectSnippet(
  content: string,
): { content_type: ContentType; language: string | null } {
  const trimmed = content.trim();
  if (!trimmed) return { content_type: 'text', language: null };

  if (looksLikeCli(trimmed)) {
    return { content_type: 'cli', language: null };
  }

  const result = hljs.highlightAuto(trimmed, [...SUBSET]);

  if ((result.relevance ?? 0) >= RELEVANCE_THRESHOLD && result.language) {
    return { content_type: 'code', language: tieBreak(trimmed, result.language) };
  }

  return { content_type: 'text', language: null };
}

// ── Static preview highlighting (used by list cards) ─────────────────────────

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/**
 * Returns hljs-classed HTML for a known language, otherwise escaped plain text.
 * Safe to feed to {@html} — hljs escapes the content it wraps.
 */
export function highlightPreviewHtml(content: string, language: string | null): string {
  if (language && hljs.getLanguage(language)) {
    try {
      return hljs.highlight(content, { language }).value;
    } catch {
      /* fall through to escaped text */
    }
  }
  return escapeHtml(content);
}
