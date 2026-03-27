<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import SearchForm from './components/SearchForm.vue';
import SearchResults from './components/SearchResults.vue';
import { invoke } from '@tauri-apps/api/core';
import { LogicalSize, getCurrentWindow } from '@tauri-apps/api/window';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { attachConsole, error as logError, info as logInfo } from '@tauri-apps/plugin-log';
import { getPathHistoryFromStore } from './lib/store';

interface SearchResult {
  path: string;
  line_number: number;
  content: string;
  before_context?: string[];
  after_context?: string[];
}

interface SearchPageStats {
  total_matches: number;
  returned_in_page: number;
  page_size: number;
  elapsed_ms: number;
  files_scanned: number;
  files_skipped: number;
}

interface AppSettings {
  hotkey: string;
  autostart: boolean;
  prefill_strategy: string;
  search_defaults: {
    case_sensitive: boolean;
    whole_word: boolean;
    use_regex: boolean;
    literal: boolean;
    multiline: boolean;
    before_context: number;
    after_context: number;
    engine: 'rust_regex' | 'pcre2';
    binary_policy: 'skip' | 'lossy';
    max_depth?: number;
    file_types: string[];
    exclude_patterns: string[];
    page_size: number;
    max_results: number;
    timeout_seconds: number;
  };
  quick_window_sizes: {
    width: number;
    idle_height: number;
    searching_height: number;
    no_results_height: number;
    error_height: number;
    results_height: number;
  };
  history: {
    limit: number;
  };
  tray: {
    menu_on_left_click: boolean;
  };
  behavior: {
    search_as_you_type: boolean;
    search_debounce_ms: number;
  };
  search_profiles: Array<{
    id: string;
    name: string;
    file_types: string[];
    exclude_patterns: string[];
  }>;
  saved_searches: Array<{
    id: string;
    name: string;
    path: string;
    pattern: string;
    case_sensitive: boolean;
    whole_word: boolean;
    use_regex: boolean;
    literal: boolean;
    multiline: boolean;
    before_context: number;
    after_context: number;
    engine: 'rust_regex' | 'pcre2';
    binary_policy: 'skip' | 'lossy';
    max_depth?: number;
    file_types: string[];
    exclude_patterns: string[];
    page_size: number;
    max_results: number;
    timeout_seconds: number;
  }>;
  editor: {
    mode: 'system' | 'preset' | 'custom_template';
    preset:
      | 'vs_code'
      | 'vs_code_insiders'
      | 'vs_codium'
      | 'cursor'
      | 'windsurf'
      | 'zed'
      | 'sublime'
      | 'vim'
      | 'neo_vim'
      | 'emacs'
      | 'helix'
      | 'nano'
      | 'text_mate'
      | 'notepad_plus_plus'
      | 'xcode'
      | 'jet_brains'
      | 'jet_brains_intellij'
      | 'jet_brains_web_storm'
      | 'jet_brains_php_storm'
      | 'jet_brains_py_charm'
      | 'jet_brains_ruby_mine'
      | 'jet_brains_go_land'
      | 'jet_brains_c_lion'
      | 'jet_brains_rider'
      | 'jet_brains_data_grip'
      | 'jet_brains_android_studio';
    custom_template: string;
  };
}

const results = ref<SearchResult[]>([]);
const isSearching = ref(false);
const isLoadingMore = ref(false);
const error = ref<string | null>(null);
const searchStats = ref<SearchPageStats | null>(null);
const lastGlobalPattern = ref('');
const lastGlobalUseRegex = ref(false);
const lastGlobalCaseSensitive = ref(false);
const isQuickWindow = ref(false);
const prefillPath = ref('');
const focusPatternSignal = ref(0);
const hasSearched = ref(false);
const settings = ref<AppSettings | null>(null);
const settingsDraft = ref<AppSettings | null>(null);
const settingsError = ref<string | null>(null);
const isSavingSettings = ref(false);
const isCapturingHotkey = ref(false);
const lastCapturedHotkey = ref('');
const editorTestStatus = ref('');
const includePatternInput = ref('');
const excludePatternInput = ref('');
let abortController: AbortController | null = null;
const searchSessionId = ref<string | null>(null);
const nextCursor = ref<number | null>(null);
const hasMoreServerResults = ref(false);
const activeResultIndex = ref(0);
const activeProfileId = ref('everything');
const latestSearchParams = ref<{
  path: string;
  pattern: string;
  case_sensitive: boolean;
  whole_word: boolean;
  use_regex: boolean;
  literal: boolean;
  multiline: boolean;
  before_context: number;
  after_context: number;
  engine: 'rust_regex' | 'pcre2';
  binary_policy: 'skip' | 'lossy';
  max_depth?: number;
  file_types: string[];
  exclude_patterns: string[];
  page_size: number;
  max_results: number;
  timeout_seconds: number;
} | null>(null);
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let searchRequestSeq = 0;
let unlistenQuickActivated: UnlistenFn | null = null;
let unlistenQuickFocus: UnlistenFn | null = null;
let detachLogConsole: (() => void) | null = null;

const FALLBACK_SETTINGS: AppSettings = {
  hotkey: 'CommandOrControl+Shift+F',
  autostart: false,
  prefill_strategy: 'active_then_clipboard_then_history',
  search_defaults: {
    case_sensitive: false,
    whole_word: false,
    use_regex: false,
    literal: false,
    multiline: false,
    before_context: 0,
    after_context: 0,
    engine: 'rust_regex',
    binary_policy: 'lossy',
    max_depth: undefined,
    file_types: [],
    exclude_patterns: [],
    page_size: 50,
    max_results: 100,
    timeout_seconds: 60,
  },
  quick_window_sizes: {
    width: 980,
    idle_height: 760,
    searching_height: 820,
    no_results_height: 905,
    error_height: 995,
    results_height: 1145,
  },
  history: {
    limit: 20,
  },
  tray: {
    menu_on_left_click: true,
  },
  behavior: {
    search_as_you_type: true,
    search_debounce_ms: 300,
  },
  search_profiles: [
    {
      id: 'code',
      name: 'Code',
      file_types: [],
      exclude_patterns: [
        'node_modules/**',
        '.git/**',
        'dist/**',
        'build/**',
        'target/**',
        'vendor/**',
      ],
    },
    {
      id: 'frontend',
      name: 'Frontend',
      file_types: ['*.ts', '*.tsx', '*.js', '*.jsx', '*.vue', '*.css', '*.scss', '*.html'],
      exclude_patterns: ['node_modules/**', 'dist/**', 'coverage/**', '.git/**'],
    },
    {
      id: 'rust',
      name: 'Rust',
      file_types: ['*.rs', '*.toml'],
      exclude_patterns: ['target/**', '.git/**'],
    },
    {
      id: 'everything',
      name: 'Everything',
      file_types: [],
      exclude_patterns: [],
    },
  ],
  saved_searches: [],
  editor: {
    mode: 'system',
    preset: 'vs_code',
    custom_template: 'code -g "{path}:{line}:{column}"',
  },
};

function currentSettings(): AppSettings {
  return settings.value ?? FALLBACK_SETTINGS;
}

function isLikelyPath(value: string): boolean {
  if (!value) return false;
  return value.startsWith('/') || /^[a-zA-Z]:\\/.test(value) || value.startsWith('~');
}

async function computePathFromHistoryThenClipboard(): Promise<string> {
  try {
    const pathHistory = await getPathHistoryFromStore();
    if (typeof pathHistory[0] === 'string') {
      return pathHistory[0];
    }
  } catch {
    // Ignore store errors and return empty fallback.
  }
  return '';
}

async function computePathFromClipboardThenHistory(): Promise<string> {
  try {
    const clipText = await navigator.clipboard.readText();
    if (isLikelyPath(clipText.trim())) {
      return clipText.trim();
    }
  } catch {
    // Clipboard can be unavailable depending on platform permissions.
  }

  return computePathFromHistoryThenClipboard();
}

async function computeFallbackPath(): Promise<string> {
  const strategy = currentSettings().prefill_strategy;
  if (strategy === 'history_then_clipboard') {
    const history = await computePathFromHistoryThenClipboard();
    return history || (await computePathFromClipboardThenHistory());
  }
  if (strategy === 'clipboard_then_history') {
    return computePathFromClipboardThenHistory();
  }
  // active_then_clipboard_then_history and unknown strategy fallback
  return computePathFromClipboardThenHistory();
}

async function prefillQuickPath() {
  try {
    const detected = await invoke<string | null>('detect_active_path');
    prefillPath.value = detected?.trim() || (await computeFallbackPath());
  } catch {
    prefillPath.value = await computeFallbackPath();
  } finally {
    await nextTick();
    focusPatternSignal.value += 1;
    focusPatternInputWithRetry();
  }
}

function focusPatternInputWithRetry() {
  const tryFocus = () => {
    const patternInput = document.querySelector<HTMLInputElement>(
      'input[placeholder="Search pattern..."]'
    );
    if (patternInput) {
      patternInput.focus();
      patternInput.select();
      return true;
    }
    return false;
  };

  if (tryFocus()) return;
  setTimeout(() => {
    if (tryFocus()) return;
    setTimeout(() => {
      tryFocus();
    }, 60);
  }, 30);
}

async function activateQuickWindowState(reset = true) {
  if (!isQuickWindow.value) return;
  if (reset) {
    results.value = [];
    error.value = null;
    hasSearched.value = false;
    const quick = currentSettings().quick_window_sizes;
    await resizeQuickWindow({ width: quick.width, height: quick.idle_height }, false);
  }
  await prefillQuickPath();
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3;
}

async function resizeQuickWindow(target: { width: number; height: number }, animate = true) {
  if (!isQuickWindow.value) return;
  const currentWindow = getCurrentWindow();
  if (!animate) {
    await currentWindow.setSize(new LogicalSize(target.width, target.height));
    return;
  }

  const current = await currentWindow.innerSize();
  const startWidth = current.width;
  const startHeight = current.height;
  const widthDelta = target.width - startWidth;
  const heightDelta = target.height - startHeight;
  const steps = 7;

  if (widthDelta === 0 && heightDelta === 0) return;

  for (let i = 1; i <= steps; i += 1) {
    const progress = easeOutCubic(i / steps);
    const width = Math.round(startWidth + widthDelta * progress);
    const height = Math.round(startHeight + heightDelta * progress);
    await currentWindow.setSize(new LogicalSize(width, height));
    if (i < steps) {
      await sleep(14);
    }
  }
}

function handleGlobalKeydown(event: KeyboardEvent) {
  if (!isQuickWindow.value) return;
  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    invoke('hide_quick_search_window').catch(() => {});
    return;
  }
  if (!results.value.length) return;
  const target = event.target as HTMLElement | null;
  const isTypingField = target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA';

  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'c') {
    const selected =
      results.value[Math.max(0, Math.min(activeResultIndex.value, results.value.length - 1))];
    if (selected) {
      navigator.clipboard.writeText(`${selected.path}:${selected.line_number}`).catch(() => {});
      event.preventDefault();
    }
    return;
  }
  if (!isTypingField && event.key === 'ArrowDown') {
    activeResultIndex.value = Math.min(results.value.length - 1, activeResultIndex.value + 1);
    event.preventDefault();
    return;
  }
  if (!isTypingField && event.key === 'ArrowUp') {
    activeResultIndex.value = Math.max(0, activeResultIndex.value - 1);
    event.preventDefault();
    return;
  }
  if (!isTypingField && event.key === 'Enter') {
    const selected =
      results.value[Math.max(0, Math.min(activeResultIndex.value, results.value.length - 1))];
    if (selected) {
      invoke('open_result_in_editor', {
        path: selected.path,
        line: selected.line_number,
        column: 1,
      }).catch(() => {});
      event.preventDefault();
    }
    return;
  }
  if (event.key === 'Tab') {
    const fields = [
      document.querySelector<HTMLInputElement>('[data-field-id="path-input"]'),
      document.querySelector<HTMLInputElement>('[data-field-id="pattern-input"]'),
      document.querySelector<HTMLInputElement>('[data-field-id="local-filter"]'),
    ].filter(Boolean) as HTMLInputElement[];
    if (!fields.length) return;
    const active = document.activeElement as HTMLInputElement | null;
    const idx = fields.findIndex((item) => item === active);
    const next = fields[(idx + 1 + fields.length) % fields.length];
    next.focus();
    next.select();
    event.preventDefault();
  }
}

async function handleSearch(params: {
  path: string;
  pattern: string;
  case_sensitive: boolean;
  whole_word: boolean;
  use_regex: boolean;
  literal: boolean;
  multiline: boolean;
  before_context: number;
  after_context: number;
  engine: 'rust_regex' | 'pcre2';
  binary_policy: 'skip' | 'lossy';
  max_depth?: number;
  file_types: string[];
  exclude_patterns: string[];
  page_size: number;
  max_results: number;
  timeout_seconds: number;
}) {
  const requestSeq = ++searchRequestSeq;
  if (searchSessionId.value) {
    await invoke('search_cancel', { sessionId: searchSessionId.value }).catch(() => {});
    searchSessionId.value = null;
  }
  isSearching.value = true;
  error.value = null;
  hasSearched.value = true;
  abortController = new AbortController();
  results.value = [];
  searchSessionId.value = null;
  nextCursor.value = null;
  hasMoreServerResults.value = false;
  searchStats.value = null;
  lastGlobalPattern.value = params.pattern;
  lastGlobalCaseSensitive.value = params.case_sensitive;
  lastGlobalUseRegex.value = params.use_regex;
  latestSearchParams.value = params;

  try {
    const start = await invoke<{
      session_id: string;
      page: {
        items: SearchResult[];
        has_more: boolean;
        next_cursor: number | null;
        stats: SearchPageStats;
      };
    }>('search_start', {
      params,
      signal: abortController.signal,
    });
    if (requestSeq !== searchRequestSeq) {
      await invoke('search_cancel', { sessionId: start.session_id }).catch(() => {});
      return;
    }
    searchSessionId.value = start.session_id;
    results.value = start.page.items;
    hasMoreServerResults.value = start.page.has_more;
    nextCursor.value = start.page.next_cursor;
    searchStats.value = start.page.stats;
  } catch (e) {
    if ((e as Error)?.name === 'AbortError') {
      results.value = [];
      return;
    }
    error.value = e as string;
    results.value = [];
  } finally {
    if (requestSeq === searchRequestSeq) {
      isSearching.value = false;
      abortController = null;
    }
  }
}

function handleCancel() {
  searchRequestSeq += 1;
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer);
    searchDebounceTimer = null;
  }
  if (abortController) {
    abortController.abort();
    isSearching.value = false;
  }
  if (searchSessionId.value) {
    invoke('search_cancel', { sessionId: searchSessionId.value }).catch(() => {});
    searchSessionId.value = null;
    nextCursor.value = null;
    hasMoreServerResults.value = false;
  }
}

function scheduleIncrementalSearch(params: {
  path: string;
  pattern: string;
  case_sensitive: boolean;
  whole_word: boolean;
  use_regex: boolean;
  literal: boolean;
  multiline: boolean;
  before_context: number;
  after_context: number;
  engine: 'rust_regex' | 'pcre2';
  binary_policy: 'skip' | 'lossy';
  max_depth?: number;
  file_types: string[];
  exclude_patterns: string[];
  page_size: number;
  max_results: number;
  timeout_seconds: number;
}) {
  latestSearchParams.value = params;
  const behavior = currentSettings().behavior;
  if (!behavior.search_as_you_type) return;
  if (!params.path.trim() || !params.pattern.trim()) return;
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer);
  }
  const debounceMs = Math.min(400, Math.max(200, Number(behavior.search_debounce_ms) || 300));
  searchDebounceTimer = setTimeout(() => {
    handleSearch(params).catch(() => {});
  }, debounceMs);
}

function applySearchProfile(profileId: string) {
  activeProfileId.value = profileId;
  const draft = settingsDraft.value;
  if (!draft) return;
  const profile = draft.search_profiles.find((item) => item.id === profileId);
  if (!profile) return;
  draft.search_defaults.file_types = [...profile.file_types];
  draft.search_defaults.exclude_patterns = [...profile.exclude_patterns];
  saveSettings().catch(() => {});
}

function saveCurrentPreset(
  name: string,
  params: {
    path: string;
    pattern: string;
    case_sensitive: boolean;
    whole_word: boolean;
    use_regex: boolean;
    literal: boolean;
    multiline: boolean;
    before_context: number;
    after_context: number;
    engine: 'rust_regex' | 'pcre2';
    binary_policy: 'skip' | 'lossy';
    max_depth?: number;
    file_types: string[];
    exclude_patterns: string[];
    page_size: number;
    max_results: number;
    timeout_seconds: number;
  }
) {
  if (!settingsDraft.value) return;
  const id = `preset-${Date.now()}`;
  settingsDraft.value.saved_searches.unshift({ id, name, ...params });
  saveSettings().catch(() => {});
}

function applySavedPreset(presetId: string) {
  const preset = settingsDraft.value?.saved_searches.find((item) => item.id === presetId);
  if (!preset || !latestSearchParams.value) return;
  const merged = {
    ...latestSearchParams.value,
    ...preset,
  };
  latestSearchParams.value = merged;
  handleSearch(merged).catch(() => {});
}

function deleteSavedPreset(presetId: string) {
  if (!settingsDraft.value) return;
  settingsDraft.value.saved_searches = settingsDraft.value.saved_searches.filter(
    (item) => item.id !== presetId
  );
  saveSettings().catch(() => {});
}

async function handleLoadMore() {
  if (!searchSessionId.value || !hasMoreServerResults.value || isLoadingMore.value) return;
  isLoadingMore.value = true;
  try {
    const page = await invoke<{
      items: SearchResult[];
      has_more: boolean;
      next_cursor: number | null;
      stats: SearchPageStats;
    }>('search_next', {
      sessionId: searchSessionId.value,
      cursor: nextCursor.value ?? 0,
    });
    results.value = [...results.value, ...page.items];
    hasMoreServerResults.value = page.has_more;
    nextCursor.value = page.next_cursor;
    searchStats.value = page.stats;
  } catch (e) {
    error.value = String(e);
  } finally {
    isLoadingMore.value = false;
  }
}

async function loadSettings() {
  settingsError.value = null;
  try {
    const loaded = await invoke<AppSettings>('get_settings');
    settings.value = loaded;
    settingsDraft.value = JSON.parse(JSON.stringify(loaded));
    activeProfileId.value = loaded.search_profiles[0]?.id ?? 'everything';
  } catch (e) {
    settingsError.value = String(e);
    settings.value = FALLBACK_SETTINGS;
    settingsDraft.value = JSON.parse(JSON.stringify(FALLBACK_SETTINGS));
    activeProfileId.value = 'everything';
  }
}

function buildSanitizedSettingsDraft(): AppSettings | null {
  if (!settingsDraft.value) return null;
  const draft = JSON.parse(JSON.stringify(settingsDraft.value)) as AppSettings;
  // Normalize numeric settings to avoid storing invalid/NaN values.
  draft.history.limit = Math.max(
    1,
    Number.isFinite(draft.history.limit) ? Math.floor(draft.history.limit) : 20
  );
  draft.search_defaults.max_results = Math.max(
    1,
    Number.isFinite(draft.search_defaults.max_results)
      ? Math.floor(draft.search_defaults.max_results)
      : 100
  );
  draft.search_defaults.timeout_seconds = Math.max(
    1,
    Number.isFinite(draft.search_defaults.timeout_seconds)
      ? Math.floor(draft.search_defaults.timeout_seconds)
      : 60
  );
  draft.search_defaults.page_size = Math.max(
    1,
    Number.isFinite(draft.search_defaults.page_size)
      ? Math.floor(draft.search_defaults.page_size)
      : 50
  );
  draft.search_defaults.before_context = Math.max(
    0,
    Number.isFinite(draft.search_defaults.before_context)
      ? Math.floor(draft.search_defaults.before_context)
      : 0
  );
  draft.search_defaults.after_context = Math.max(
    0,
    Number.isFinite(draft.search_defaults.after_context)
      ? Math.floor(draft.search_defaults.after_context)
      : 0
  );
  draft.quick_window_sizes.width = Math.max(
    600,
    Number.isFinite(draft.quick_window_sizes.width)
      ? Math.floor(draft.quick_window_sizes.width)
      : 980
  );
  draft.quick_window_sizes.idle_height = Math.max(
    300,
    Number.isFinite(draft.quick_window_sizes.idle_height)
      ? Math.floor(draft.quick_window_sizes.idle_height)
      : 520
  );
  draft.quick_window_sizes.searching_height = Math.max(
    300,
    Number.isFinite(draft.quick_window_sizes.searching_height)
      ? Math.floor(draft.quick_window_sizes.searching_height)
      : 560
  );
  draft.quick_window_sizes.no_results_height = Math.max(
    300,
    Number.isFinite(draft.quick_window_sizes.no_results_height)
      ? Math.floor(draft.quick_window_sizes.no_results_height)
      : 620
  );
  draft.quick_window_sizes.error_height = Math.max(
    300,
    Number.isFinite(draft.quick_window_sizes.error_height)
      ? Math.floor(draft.quick_window_sizes.error_height)
      : 680
  );
  draft.quick_window_sizes.results_height = Math.max(
    300,
    Number.isFinite(draft.quick_window_sizes.results_height)
      ? Math.floor(draft.quick_window_sizes.results_height)
      : 780
  );
  draft.behavior.search_debounce_ms = Math.max(
    200,
    Math.min(
      400,
      Number.isFinite(draft.behavior.search_debounce_ms)
        ? Math.floor(draft.behavior.search_debounce_ms)
        : 300
    )
  );
  if (!Array.isArray(draft.search_profiles) || draft.search_profiles.length === 0) {
    draft.search_profiles = FALLBACK_SETTINGS.search_profiles;
  }
  if (!Array.isArray(draft.saved_searches)) {
    draft.saved_searches = [];
  }
  draft.hotkey = draft.hotkey.trim();
  if (!draft.hotkey) {
    settingsError.value = 'Hotkey cannot be empty.';
    return null;
  }
  if (draft.editor.mode === 'custom_template' && !draft.editor.custom_template.includes('{path}')) {
    settingsError.value = 'Custom template must include {path}.';
    return null;
  }
  return draft;
}

async function saveSettings() {
  if (!settingsDraft.value || isSavingSettings.value) return;
  const draft = buildSanitizedSettingsDraft();
  if (!draft) return;
  isSavingSettings.value = true;
  settingsError.value = null;
  try {
    const updated = await invoke<AppSettings>('update_settings', { settings: draft });
    settings.value = updated;
    settingsDraft.value = JSON.parse(JSON.stringify(updated));
  } catch (e) {
    settingsError.value = String(e);
  } finally {
    isSavingSettings.value = false;
  }
}

function startHotkeyCapture() {
  isCapturingHotkey.value = true;
  settingsError.value = null;
}

function stopHotkeyCapture() {
  isCapturingHotkey.value = false;
}

function resetHotkeyToDefault() {
  if (!settingsDraft.value) return;
  settingsDraft.value.hotkey = FALLBACK_SETTINGS.hotkey;
  isCapturingHotkey.value = false;
  saveSettings().catch(() => {});
}

function codeToHotkeyKey(event: KeyboardEvent): string {
  if (event.code.startsWith('Key')) return event.code.slice(3).toUpperCase();
  if (event.code.startsWith('Digit')) return event.code.slice(5);
  if (/^F\d{1,2}$/.test(event.code)) return event.code;
  const key = event.key.length === 1 ? event.key.toUpperCase() : event.key;
  const allowed = new Set([
    'Space',
    'Enter',
    'Tab',
    'Escape',
    'Backspace',
    'Insert',
    'Delete',
    'Home',
    'End',
    'PageUp',
    'PageDown',
    'ArrowUp',
    'ArrowDown',
    'ArrowLeft',
    'ArrowRight',
  ]);
  return allowed.has(key) ? key : '';
}

function handleHotkeyCapture(event: KeyboardEvent) {
  if (!isCapturingHotkey.value || isQuickWindow.value || !settingsDraft.value) return;
  event.preventDefault();
  event.stopPropagation();

  if (event.key === 'Escape') {
    stopHotkeyCapture();
    return;
  }

  const primary = codeToHotkeyKey(event);
  if (!primary) return;

  const parts: string[] = [];
  if (event.metaKey || event.ctrlKey) parts.push('CommandOrControl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  parts.push(primary);
  settingsDraft.value.hotkey = parts.join('+');
  lastCapturedHotkey.value = settingsDraft.value.hotkey;
  stopHotkeyCapture();
  saveSettings().catch(() => {});
}

async function testEditorOpen() {
  editorTestStatus.value = '';
  try {
    await logInfo('[editor-test] creating demo file and launching editor');
    const demoPath = await invoke<string>('test_open_in_editor');
    editorTestStatus.value = `Editor launch requested: ${demoPath}`;
  } catch (e) {
    const message = String(e);
    settingsError.value = message;
    editorTestStatus.value = message;
    await logError(`[editor-test] failed: ${message}`);
  }
}

async function openSettingsFromQuickSearch() {
  try {
    await invoke('hide_quick_search_window');
  } finally {
    await invoke('show_settings_window');
  }
}

function addDefaultIncludePattern() {
  if (!settingsDraft.value) return;
  const value = includePatternInput.value.trim();
  if (!value) return;
  if (!settingsDraft.value.search_defaults.file_types.includes(value)) {
    settingsDraft.value.search_defaults.file_types.push(value);
  }
  includePatternInput.value = '';
  saveSettings().catch(() => {});
}

function removeDefaultIncludePattern(pattern: string) {
  if (!settingsDraft.value) return;
  settingsDraft.value.search_defaults.file_types =
    settingsDraft.value.search_defaults.file_types.filter((p) => p !== pattern);
  saveSettings().catch(() => {});
}

function addDefaultExcludePattern() {
  if (!settingsDraft.value) return;
  const value = excludePatternInput.value.trim();
  if (!value) return;
  if (!settingsDraft.value.search_defaults.exclude_patterns.includes(value)) {
    settingsDraft.value.search_defaults.exclude_patterns.push(value);
  }
  excludePatternInput.value = '';
  saveSettings().catch(() => {});
}

function removeDefaultExcludePattern(pattern: string) {
  if (!settingsDraft.value) return;
  settingsDraft.value.search_defaults.exclude_patterns =
    settingsDraft.value.search_defaults.exclude_patterns.filter((p) => p !== pattern);
  saveSettings().catch(() => {});
}

onMounted(async () => {
  if (!detachLogConsole) {
    try {
      detachLogConsole = await attachConsole();
    } catch {
      // Ignore attachConsole failures in restricted runtime contexts.
    }
  }

  const currentWindow = getCurrentWindow();
  isQuickWindow.value = currentWindow.label === 'quick-search';

  if (isQuickWindow.value) {
    await loadSettings();
    await activateQuickWindowState(false);
    unlistenQuickActivated = await currentWindow.listen('quick-search-activated', async () => {
      await activateQuickWindowState(true);
    });
    unlistenQuickFocus = await currentWindow.onFocusChanged(async (event) => {
      if (event.payload) {
        await activateQuickWindowState(false);
      }
    });
    window.addEventListener('keydown', handleGlobalKeydown, true);
    document.addEventListener('keydown', handleGlobalKeydown, true);
  } else {
    await loadSettings();
  }
});

onUnmounted(() => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer);
    searchDebounceTimer = null;
  }
  if (unlistenQuickActivated) {
    unlistenQuickActivated();
  }
  if (unlistenQuickFocus) {
    unlistenQuickFocus();
  }
  window.removeEventListener('keydown', handleGlobalKeydown, true);
  document.removeEventListener('keydown', handleGlobalKeydown, true);
  if (detachLogConsole) {
    detachLogConsole();
    detachLogConsole = null;
  }
});

watch(
  () => ({
    quick: isQuickWindow.value,
    resultCount: results.value.length,
    searching: isSearching.value,
    hasError: !!error.value,
    searched: hasSearched.value,
  }),
  async ({ quick, resultCount, searching, hasError, searched }) => {
    if (!quick) return;
    const sizes = currentSettings().quick_window_sizes;
    if (searching) {
      await resizeQuickWindow({ width: sizes.width, height: sizes.searching_height });
      return;
    }
    if (hasError) {
      await resizeQuickWindow({ width: sizes.width, height: sizes.error_height });
      return;
    }
    if (resultCount > 0) {
      await resizeQuickWindow({ width: sizes.width, height: sizes.results_height });
      return;
    }
    if (searched) {
      await resizeQuickWindow({ width: sizes.width, height: sizes.no_results_height });
      return;
    }
    await resizeQuickWindow({ width: sizes.width, height: sizes.idle_height });
  }
);

watch(
  () => results.value,
  () => {
    activeResultIndex.value = 0;
  }
);
</script>

<template>
  <div class="min-h-screen bg-slate-900 text-slate-100 p-3">
    <div
      v-if="!isQuickWindow"
      class="max-w-4xl mx-auto rounded-lg border border-slate-700 bg-slate-900/80 p-4 space-y-4"
    >
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-cyan-300 uppercase tracking-wide">Settings</h2>
        <button
          type="button"
          class="h-8 px-3 rounded border border-cyan-500/60 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-200 text-xs"
          @click="invoke('show_quick_search_window')"
        >
          Open Quick Search
        </button>
      </div>

      <div
        v-if="settingsDraft"
        class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs"
        @change.capture="saveSettings"
      >
        <div class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2">
          <p class="font-semibold text-cyan-300">Hotkey</p>
          <div class="flex gap-2">
            <input
              v-model="settingsDraft.hotkey"
              class="flex-1 h-8 px-2 border border-slate-700 rounded bg-slate-900"
              @focus="startHotkeyCapture"
              @blur="stopHotkeyCapture"
              @keydown="handleHotkeyCapture"
            />
            <button
              type="button"
              class="h-8 px-3 rounded border border-slate-600 bg-slate-800 hover:bg-slate-700 text-slate-200"
              @click="resetHotkeyToDefault"
            >
              Reset
            </button>
          </div>
          <p class="text-slate-400">Focus input and press desired key combination.</p>
          <p v-if="isCapturingHotkey" class="text-amber-300">Capturing...</p>
          <p v-else-if="lastCapturedHotkey" class="text-cyan-300">
            Captured: {{ lastCapturedHotkey }}
          </p>
        </div>

        <div class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2">
          <p class="font-semibold text-cyan-300">Startup & Tray</p>
          <label class="flex items-center gap-2"
            ><input v-model="settingsDraft.autostart" type="checkbox" /> Autostart</label
          >
          <label class="flex items-center gap-2"
            ><input v-model="settingsDraft.tray.menu_on_left_click" type="checkbox" /> Tray menu on
            left click</label
          >
        </div>

        <div class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2">
          <p class="font-semibold text-cyan-300">Editor</p>
          <div class="relative">
            <select
              v-model="settingsDraft.editor.mode"
              class="h-8 px-2 pr-8 border border-slate-700 rounded bg-slate-900 text-slate-100 w-full appearance-none focus:outline-none focus:ring-2 focus:ring-cyan-500/40 focus:border-cyan-500"
            >
              <option value="system">System default app</option>
              <option value="preset">Preset editor</option>
              <option value="custom_template">Custom template</option>
            </select>
            <span
              class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 text-[10px]"
              >▼</span
            >
          </div>

          <div v-if="settingsDraft.editor.mode === 'preset'" class="relative">
            <select
              v-model="settingsDraft.editor.preset"
              class="h-8 px-2 pr-8 border border-slate-700 rounded bg-slate-900 text-slate-100 w-full appearance-none focus:outline-none focus:ring-2 focus:ring-cyan-500/40 focus:border-cyan-500"
            >
              <option value="vs_code">VS Code</option>
              <option value="vs_code_insiders">VS Code Insiders</option>
              <option value="vs_codium">VSCodium</option>
              <option value="cursor">Cursor</option>
              <option value="windsurf">Windsurf</option>
              <option value="zed">Zed</option>
              <option value="sublime">Sublime Text</option>
              <option value="vim">Vim</option>
              <option value="neo_vim">NeoVim</option>
              <option value="emacs">Emacs</option>
              <option value="helix">Helix</option>
              <option value="nano">Nano</option>
              <option value="text_mate">TextMate (line only)</option>
              <option value="notepad_plus_plus">Notepad++</option>
              <option value="xcode">Xcode (line only)</option>
              <option value="jet_brains">JetBrains IDE (auto, `idea`)</option>
              <option value="jet_brains_intellij">JetBrains IntelliJ (`idea`)</option>
              <option value="jet_brains_web_storm">JetBrains WebStorm (`webstorm`)</option>
              <option value="jet_brains_php_storm">JetBrains PhpStorm (`phpstorm`)</option>
              <option value="jet_brains_py_charm">JetBrains PyCharm (`pycharm`)</option>
              <option value="jet_brains_ruby_mine">JetBrains RubyMine (`rubymine`)</option>
              <option value="jet_brains_go_land">JetBrains GoLand (`goland`)</option>
              <option value="jet_brains_c_lion">JetBrains CLion (`clion`)</option>
              <option value="jet_brains_rider">JetBrains Rider (`rider`)</option>
              <option value="jet_brains_data_grip">JetBrains DataGrip (`datagrip`)</option>
              <option value="jet_brains_android_studio">JetBrains Android Studio (`studio`)</option>
            </select>
            <span
              class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 text-[10px]"
              >▼</span
            >
          </div>
          <input
            v-if="settingsDraft.editor.mode === 'custom_template'"
            v-model="settingsDraft.editor.custom_template"
            class="w-full h-8 px-2 border border-slate-700 rounded bg-slate-900"
            placeholder='code -g "{path}:{line}:{column}"'
          />
          <p class="text-slate-400">
            Template placeholders: <code>{path}</code>, <code>{line}</code>, <code>{column}</code>.
          </p>
          <p class="text-[11px] text-slate-500">
            Test button creates a temporary demo file and opens it in the configured editor.
          </p>
          <button
            type="button"
            class="h-8 px-3 rounded border border-slate-600 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs"
            @click="testEditorOpen"
          >
            Test editor launch
          </button>
          <p v-if="editorTestStatus" class="text-[11px] text-slate-300 break-all">
            {{ editorTestStatus }}
          </p>
        </div>

        <div class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2">
          <p class="font-semibold text-cyan-300">Search defaults</p>
          <label class="flex items-center gap-2"
            ><input v-model="settingsDraft.search_defaults.case_sensitive" type="checkbox" /> Case
            sensitive</label
          >
          <label class="flex items-center gap-2"
            ><input v-model="settingsDraft.search_defaults.whole_word" type="checkbox" /> Whole
            word</label
          >
          <label class="flex items-center gap-2"
            ><input v-model="settingsDraft.search_defaults.use_regex" type="checkbox" /> Use
            regex</label
          >
          <div class="grid grid-cols-1 gap-2">
            <label class="space-y-1">
              <span class="text-slate-300">Max results</span>
              <input
                v-model.number="settingsDraft.search_defaults.max_results"
                type="number"
                min="1"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
                placeholder="100"
              />
              <span class="text-[11px] text-slate-500"
                >Maximum number of matches returned by one search.</span
              >
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Timeout (seconds)</span>
              <input
                v-model.number="settingsDraft.search_defaults.timeout_seconds"
                type="number"
                min="1"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
                placeholder="60"
              />
              <span class="text-[11px] text-slate-500"
                >Stop search if it runs longer than this limit.</span
              >
            </label>
          </div>
        </div>

        <div class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2">
          <p class="font-semibold text-cyan-300">Quick window</p>
          <div class="grid grid-cols-1 gap-2">
            <label class="space-y-1">
              <span class="text-slate-300">Window width (px)</span>
              <input
                v-model.number="settingsDraft.quick_window_sizes.width"
                type="number"
                min="600"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Height when idle (px)</span>
              <input
                v-model.number="settingsDraft.quick_window_sizes.idle_height"
                type="number"
                min="300"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Height while searching (px)</span>
              <input
                v-model.number="settingsDraft.quick_window_sizes.searching_height"
                type="number"
                min="300"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Height with no results (px)</span>
              <input
                v-model.number="settingsDraft.quick_window_sizes.no_results_height"
                type="number"
                min="300"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Height on error (px)</span>
              <input
                v-model.number="settingsDraft.quick_window_sizes.error_height"
                type="number"
                min="300"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Height with results (px)</span>
              <input
                v-model.number="settingsDraft.quick_window_sizes.results_height"
                type="number"
                min="300"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
          </div>
        </div>

        <div class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2 md:col-span-2">
          <p class="font-semibold text-cyan-300">Behavior</p>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
            <div class="relative">
              <select
                v-model="settingsDraft.prefill_strategy"
                class="h-8 px-2 pr-8 border border-slate-700 rounded bg-slate-900 text-slate-100 w-full appearance-none focus:outline-none focus:ring-2 focus:ring-cyan-500/40 focus:border-cyan-500"
              >
                <option value="active_then_clipboard_then_history">
                  Active folder &gt; clipboard &gt; history
                </option>
                <option value="clipboard_then_history">Clipboard &gt; history</option>
                <option value="history_then_clipboard">History &gt; clipboard</option>
              </select>
              <span
                class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 text-[10px]"
                >▼</span
              >
            </div>
            <label class="space-y-1">
              <span class="text-slate-300">History limit</span>
              <input
                v-model.number="settingsDraft.history.limit"
                type="number"
                min="1"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
            <label class="flex items-center gap-2 mt-5">
              <input v-model="settingsDraft.behavior.search_as_you_type" type="checkbox" />
              Search as you type
            </label>
            <label class="space-y-1">
              <span class="text-slate-300">Debounce (ms)</span>
              <input
                v-model.number="settingsDraft.behavior.search_debounce_ms"
                type="number"
                min="200"
                max="400"
                class="h-8 px-2 border border-slate-700 rounded bg-slate-900 w-full"
              />
            </label>
          </div>
          <p class="text-[11px] text-slate-500">
            Prefill strategy controls how quick search chooses the initial path.
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <span v-if="isSavingSettings" class="text-xs text-cyan-300">Saving settings...</span>
        <button
          type="button"
          class="h-8 px-3 rounded border border-slate-600 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs"
          @click="loadSettings"
        >
          Reload
        </button>
      </div>
      <p v-if="settingsError" class="text-xs text-red-300">{{ settingsError }}</p>
      <div
        v-if="settingsDraft"
        class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2 md:col-span-2 text-xs"
      >
        <p class="font-semibold text-cyan-300">Default include patterns</p>
        <div class="flex gap-2">
          <input
            v-model="includePatternInput"
            class="flex-1 h-8 px-2 border border-slate-700 rounded bg-slate-900"
            placeholder="*.rs or src/**"
            @keyup.enter="addDefaultIncludePattern"
          />
          <button
            type="button"
            class="h-8 px-3 rounded border border-slate-600 bg-slate-800 hover:bg-slate-700 text-slate-200"
            @click="addDefaultIncludePattern"
          >
            Add
          </button>
        </div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="pattern in settingsDraft.search_defaults.file_types"
            :key="pattern"
            class="px-2 py-1 rounded border border-slate-600 bg-slate-900 text-slate-200"
          >
            {{ pattern }}
            <button
              type="button"
              class="ml-1 text-slate-400 hover:text-red-300"
              @click="removeDefaultIncludePattern(pattern)"
            >
              ×
            </button>
          </span>
        </div>
      </div>

      <div
        v-if="settingsDraft"
        class="rounded border border-slate-700 bg-slate-800/60 p-3 space-y-2 md:col-span-2 text-xs"
      >
        <p class="font-semibold text-cyan-300">Default exclude patterns</p>
        <div class="flex gap-2">
          <input
            v-model="excludePatternInput"
            class="flex-1 h-8 px-2 border border-slate-700 rounded bg-slate-900"
            placeholder="node_modules/** or *.log"
            @keyup.enter="addDefaultExcludePattern"
          />
          <button
            type="button"
            class="h-8 px-3 rounded border border-slate-600 bg-slate-800 hover:bg-slate-700 text-slate-200"
            @click="addDefaultExcludePattern"
          >
            Add
          </button>
        </div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="pattern in settingsDraft.search_defaults.exclude_patterns"
            :key="pattern"
            class="px-2 py-1 rounded border border-slate-600 bg-slate-900 text-slate-200"
          >
            {{ pattern }}
            <button
              type="button"
              class="ml-1 text-slate-400 hover:text-red-300"
              @click="removeDefaultExcludePattern(pattern)"
            >
              ×
            </button>
          </span>
        </div>
      </div>

      <p class="text-[11px] text-slate-500">
        This window starts hidden and is available from tray.
      </p>
    </div>

    <div v-else class="max-w-6xl mx-auto space-y-3">
      <div
        data-tauri-drag-region
        class="h-9 rounded-md border border-slate-700 bg-slate-800/80 px-2 flex items-center justify-between"
        @dblclick="invoke('hide_quick_search_window')"
      >
        <span data-tauri-drag-region class="text-[11px] tracking-wide uppercase text-slate-300">
          Quick Search
        </span>
        <div class="flex items-center gap-1">
          <button
            type="button"
            class="h-6 px-2 inline-flex items-center justify-center rounded border border-slate-600 text-slate-300 hover:text-white hover:border-cyan-400 hover:bg-cyan-500/20 text-[11px]"
            aria-label="Open settings window"
            title="Open Settings"
            @click="openSettingsFromQuickSearch"
          >
            Settings
          </button>
          <button
            type="button"
            class="h-6 w-6 inline-flex items-center justify-center rounded border border-slate-600 text-slate-300 hover:text-white hover:border-red-400 hover:bg-red-500/20 text-sm leading-none"
            aria-label="Close quick search window"
            title="Close (Esc)"
            @click="invoke('hide_quick_search_window')"
          >
            ×
          </button>
        </div>
      </div>

      <SearchForm
        :is-searching="isSearching"
        :quick-mode="isQuickWindow"
        :prefill-path="prefillPath"
        :focus-pattern-signal="focusPatternSignal"
        :search-defaults="currentSettings().search_defaults"
        :history-limit="currentSettings().history.limit"
        :search-profiles="currentSettings().search_profiles"
        :active-profile-id="activeProfileId"
        :saved-searches="currentSettings().saved_searches"
        @search="handleSearch"
        @params-change="scheduleIncrementalSearch"
        @apply-profile="applySearchProfile"
        @save-preset="saveCurrentPreset"
        @apply-preset="applySavedPreset"
        @delete-preset="deleteSavedPreset"
        @cancel="handleCancel"
      />

      <div v-if="error" class="bg-red-500/10 border border-red-500/40 rounded-md p-3">
        <div class="flex">
          <div class="shrink-0">
            <svg class="h-5 w-5 text-red-300" viewBox="0 0 20 20" fill="currentColor">
              <path
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                clip-rule="evenodd"
              />
            </svg>
          </div>
          <div class="ml-3">
            <p class="text-sm text-red-200">
              {{ error }}
            </p>
          </div>
        </div>
      </div>

      <SearchResults
        :results="results"
        :is-searching="isSearching"
        :stats="searchStats"
        :global-pattern="lastGlobalPattern"
        :global-use-regex="lastGlobalUseRegex"
        :global-case-sensitive="lastGlobalCaseSensitive"
        :has-more-server-results="hasMoreServerResults"
        :is-loading-more="isLoadingMore"
        @load-more="handleLoadMore"
      />
    </div>
  </div>
</template>
