import type { Snippet, Tag, CreateSnippetInput, UpdateSnippetInput, TimePeriod } from '$lib/types';
import { listSnippets, createSnippet, updateSnippet, deleteSnippet, listTags, searchSnippets, listSnippetsByPeriod } from '$lib/api';

class SnippetStore {
  snippets = $state<Snippet[]>([]);
  selectedId = $state<number | null>(null);
  tags = $state<Tag[]>([]);
  tagFilter = $state<string | null>(null);
  searchQuery = $state<string>('');
  timePeriodFilter = $state<TimePeriod | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const [list, tagList] = await Promise.all([
        this.searchQuery
          ? searchSnippets(this.searchQuery)
          : this.timePeriodFilter
            ? listSnippetsByPeriod(this.timePeriodFilter)
            : listSnippets(this.tagFilter ?? undefined),
        listTags(),
      ]);
      this.snippets = list;
      this.tags = tagList;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async create(input: CreateSnippetInput): Promise<Snippet | null> {
    this.error = null;
    try {
      const s = await createSnippet(input);
      await this.load();
      this.selectedId = s.id;
      return s;
    } catch (e) {
      this.error = String(e);
      return null;
    }
  }

  async update(input: UpdateSnippetInput): Promise<void> {
    this.error = null;
    try {
      const updated = await updateSnippet(input);
      const idx = this.snippets.findIndex(x => x.id === updated.id);
      if (idx >= 0) this.snippets[idx] = updated;
      this.tags = await listTags();
    } catch (e) {
      this.error = String(e);
    }
  }

  async remove(id: number): Promise<void> {
    this.error = null;
    try {
      await deleteSnippet(id);
      if (this.selectedId === id) this.selectedId = null;
      this.snippets = this.snippets.filter(s => s.id !== id);
    } catch (e) {
      this.error = String(e);
    }
  }

  select(id: number | null): void {
    this.selectedId = id;
  }

  setTagFilter(tag: string | null): void {
    this.tagFilter = tag;
    this.searchQuery = '';
    this.timePeriodFilter = null;
    void this.load();
  }

  setSearch(query: string): void {
    this.searchQuery = query;
    this.tagFilter = null;
    this.timePeriodFilter = null;
    void this.load();
  }

  setTimePeriodFilter(period: TimePeriod | null): void {
    this.timePeriodFilter = period;
    this.tagFilter = null;
    this.searchQuery = '';
    void this.load();
  }
}

export const snippets = new SnippetStore();
