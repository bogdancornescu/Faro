import { invoke } from '@tauri-apps/api/core';
import type { Snippet, Tag, CreateSnippetInput, UpdateSnippetInput, TimePeriod } from './types';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

export async function createSnippet(input: CreateSnippetInput): Promise<Snippet> {
  return invoke('create_snippet', {
    title: input.title,
    content: input.content,
    contentType: input.content_type,
    tags: input.tags,
  });
}

export async function listSnippets(tagFilter?: string): Promise<Snippet[]> {
  return invoke('list_snippets', { tagFilter: tagFilter ?? null });
}

export async function getSnippet(id: number): Promise<Snippet> {
  return invoke('get_snippet', { id });
}

export async function updateSnippet(input: UpdateSnippetInput): Promise<Snippet> {
  return invoke('update_snippet', {
    id: input.id,
    title: input.title,
    content: input.content,
    contentType: input.content_type,
    tags: input.tags,
  });
}

export async function deleteSnippet(id: number): Promise<void> {
  return invoke('delete_snippet', { id });
}

export async function listTags(): Promise<Tag[]> {
  return invoke('list_tags');
}

export async function searchSnippets(query: string): Promise<Snippet[]> {
  return invoke('search_snippets', { query });
}

export async function listSnippetsByPeriod(period: TimePeriod): Promise<Snippet[]> {
  return invoke('list_snippets_by_period', { period });
}

export async function updateHotkey(hotkey: string): Promise<void> {
  return invoke('update_hotkey', { hotkey });
}

export async function recordCopy(id: number): Promise<void> {
  return invoke('record_copy', { id });
}

export async function copyToClipboard(text: string): Promise<void> {
  return writeText(text);
}
