import { load } from '@tauri-apps/plugin-store';
import { updateHotkey } from '$lib/api';

const DEFAULTS = {
  hotkey: 'Ctrl+Shift+Space',
  fontSize: 13,
  quickNoteStayOpen: false,
  theme: 'dark',
};

const STORE_FILE = 'settings.json';

class SettingsStore {
  hotkey = $state(DEFAULTS.hotkey);
  fontSize = $state(DEFAULTS.fontSize);
  quickNoteStayOpen = $state(DEFAULTS.quickNoteStayOpen);
  theme = $state(DEFAULTS.theme);
  loaded = $state(false);

  async load(): Promise<void> {
    const store = await load(STORE_FILE, { autoSave: false, defaults: DEFAULTS });
    this.hotkey = (await store.get<string>('hotkey')) ?? DEFAULTS.hotkey;
    this.fontSize = (await store.get<number>('fontSize')) ?? DEFAULTS.fontSize;
    this.quickNoteStayOpen = (await store.get<boolean>('quickNoteStayOpen')) ?? DEFAULTS.quickNoteStayOpen;
    this.theme = (await store.get<string>('theme')) ?? DEFAULTS.theme;
    this.loaded = true;
    this.applyFontSize();
    this.applyTheme();
  }

  applyFontSize(): void {
    document.documentElement.style.setProperty('--font-size-editor', `${this.fontSize}px`);
  }

  applyTheme(): void {
    document.documentElement.setAttribute('data-theme', this.theme);
  }

  async save(patch: {
    hotkey?: string;
    fontSize?: number;
    quickNoteStayOpen?: boolean;
    theme?: string;
  }): Promise<void> {
    const store = await load(STORE_FILE, { autoSave: false, defaults: DEFAULTS });
    if (patch.hotkey !== undefined && patch.hotkey !== this.hotkey) {
      await updateHotkey(patch.hotkey);
      this.hotkey = patch.hotkey;
      await store.set('hotkey', patch.hotkey);
    }
    if (patch.fontSize !== undefined) {
      this.fontSize = patch.fontSize;
      this.applyFontSize();
      await store.set('fontSize', patch.fontSize);
    }
    if (patch.quickNoteStayOpen !== undefined) {
      this.quickNoteStayOpen = patch.quickNoteStayOpen;
      await store.set('quickNoteStayOpen', patch.quickNoteStayOpen);
    }
    if (patch.theme !== undefined) {
      this.theme = patch.theme;
      this.applyTheme();
      await store.set('theme', patch.theme);
    }
    await store.save();
  }
}

export const settings = new SettingsStore();
