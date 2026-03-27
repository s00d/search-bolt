import { LazyStore } from '@tauri-apps/plugin-store';

const SETTINGS_STORE_PATH = 'settings.json';
const PATH_HISTORY_KEY = 'pathHistory';
const PATTERN_HISTORY_KEY = 'patternHistory';

const store = new LazyStore(SETTINGS_STORE_PATH);

function normalizeStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === 'string');
}

export async function getPathHistoryFromStore(): Promise<string[]> {
  const value = await store.get<string[]>(PATH_HISTORY_KEY);
  return normalizeStringArray(value);
}

export async function setPathHistoryToStore(history: string[]): Promise<void> {
  await store.set(PATH_HISTORY_KEY, history);
  await store.save();
}

export async function clearPathHistoryInStore(): Promise<void> {
  await store.delete(PATH_HISTORY_KEY);
  await store.save();
}

export async function getPatternHistoryFromStore(): Promise<string[]> {
  const value = await store.get<string[]>(PATTERN_HISTORY_KEY);
  return normalizeStringArray(value);
}

export async function setPatternHistoryToStore(history: string[]): Promise<void> {
  await store.set(PATTERN_HISTORY_KEY, history);
  await store.save();
}

export async function clearPatternHistoryInStore(): Promise<void> {
  await store.delete(PATTERN_HISTORY_KEY);
  await store.save();
}
