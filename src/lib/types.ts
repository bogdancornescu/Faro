export type ContentType = 'code' | 'cli' | 'text' | 'url';

export interface Tag {
  id: number;
  name: string;
  /** 'user' | 'system' — empty string when loaded without snippet context (list_tags) */
  source: string;
  count?: number;
}

export type TimePeriod = 'today' | 'yesterday' | 'this-week' | 'last-week' | 'older';

export interface Snippet {
  id: number;
  title: string;
  content: string;
  content_type: ContentType;
  copy_count: number;
  created_at: string;
  updated_at: string;
  tags: Tag[];
}

export interface TagInput {
  name: string;
  source: string;
}

export interface CreateSnippetInput {
  title: string;
  content: string;
  content_type: ContentType;
  tags: TagInput[];
}

export interface UpdateSnippetInput {
  id: number;
  title: string;
  content: string;
  content_type: ContentType;
  tags: TagInput[];
}

export interface Settings {
  hotkey: string;
  fontSize: number;
  quickNoteStayOpen: boolean;
  theme: string;
}
