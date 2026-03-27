<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { MagnifyingGlassIcon } from '@heroicons/vue/24/outline';
import PathSelector from './search/PathSelector.vue';
import PatternInput from './search/PatternInput.vue';
import AdvancedOptions from './search/AdvancedOptions.vue';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { UnlistenFn } from '@tauri-apps/api/event';
import {
  clearPathHistoryInStore,
  clearPatternHistoryInStore,
  getPathHistoryFromStore,
  getPatternHistoryFromStore,
  setPathHistoryToStore,
  setPatternHistoryToStore,
} from '../lib/store';

const MAX_HISTORY = 20;

const props = defineProps<{
  isSearching: boolean;
  quickMode?: boolean;
  prefillPath?: string;
  focusPatternSignal?: number;
  historyLimit?: number;
  activeProfileId?: string;
  searchProfiles?: Array<{
    id: string;
    name: string;
    file_types: string[];
    exclude_patterns: string[];
  }>;
  savedSearches?: Array<{
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
  searchDefaults?: {
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
}>();

const emit = defineEmits<{
  search: [
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
    },
  ];
  cancel: [];
  'params-change': [
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
    },
  ];
  'apply-profile': [profileId: string];
  'save-preset': [
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
    },
  ];
  'apply-preset': [presetId: string];
  'delete-preset': [presetId: string];
}>();

const searchPath = ref('');
const searchPattern = ref('');
const caseSensitive = ref(false);
const wholeWord = ref(false);
const useRegex = ref(false);
const literal = ref(false);
const multiline = ref(false);
const beforeContext = ref(0);
const afterContext = ref(0);
const engine = ref<'rust_regex' | 'pcre2'>('rust_regex');
const binaryPolicy = ref<'skip' | 'lossy'>('lossy');
const maxDepth = ref<number>();
const fileTypes = ref<string[]>([]);
const excludePatterns = ref<string[]>([]);
const pathHistory = ref<string[]>([]);
const patternHistory = ref<string[]>([]);
const isDragging = ref(false);
const maxResults = ref(100);
const timeoutSeconds = ref(60);
const pageSize = ref(50);
const selectedProfileId = ref('everything');
const selectedPresetId = ref('');
const newPresetName = ref('');

let dragDropUnlisten: UnlistenFn | null = null;

onMounted(async () => {
  loadHistory();

  dragDropUnlisten = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      isDragging.value = true;
    } else {
      isDragging.value = false;
    }
  });
});

watch(
  () => [props.prefillPath, props.focusPatternSignal],
  ([nextPath]) => {
    if (typeof nextPath === 'string' && nextPath.trim()) {
      searchPath.value = nextPath.trim();
    }
  },
  { immediate: true }
);

watch(
  () => props.searchDefaults,
  (defaults) => {
    if (!defaults) return;
    caseSensitive.value = defaults.case_sensitive;
    wholeWord.value = defaults.whole_word;
    useRegex.value = defaults.use_regex;
    literal.value = defaults.literal;
    multiline.value = defaults.multiline;
    beforeContext.value = defaults.before_context;
    afterContext.value = defaults.after_context;
    engine.value = defaults.engine;
    binaryPolicy.value = defaults.binary_policy;
    maxDepth.value = defaults.max_depth;
    fileTypes.value = [...defaults.file_types];
    excludePatterns.value = [...defaults.exclude_patterns];
    pageSize.value = defaults.page_size;
    maxResults.value = defaults.max_results;
    timeoutSeconds.value = defaults.timeout_seconds;
  },
  { immediate: true }
);

onUnmounted(() => {
  if (dragDropUnlisten) {
    dragDropUnlisten();
  }
});

function loadHistory() {
  Promise.all([getPathHistoryFromStore(), getPatternHistoryFromStore()])
    .then(([paths, patterns]) => {
      pathHistory.value = paths;
      patternHistory.value = patterns;
    })
    .catch(() => {
      // Keep empty history on store read errors.
    });
}

function savePathHistory() {
  if (!searchPath.value) return;

  const newHistory = [searchPath.value, ...pathHistory.value.filter((p) => p !== searchPath.value)];
  pathHistory.value = newHistory.slice(0, props.historyLimit ?? MAX_HISTORY);
  setPathHistoryToStore(pathHistory.value).catch(() => {
    // Ignore persistence error, keep in-memory history.
  });
}

function savePatternHistory() {
  if (!searchPattern.value) return;

  const newHistory = [
    searchPattern.value,
    ...patternHistory.value.filter((p) => p !== searchPattern.value),
  ];
  patternHistory.value = newHistory.slice(0, props.historyLimit ?? MAX_HISTORY);
  setPatternHistoryToStore(patternHistory.value).catch(() => {
    // Ignore persistence error, keep in-memory history.
  });
}

function clearPathHistory() {
  pathHistory.value = [];
  clearPathHistoryInStore().catch(() => {
    // Ignore persistence error, keep in-memory history.
  });
}

function clearPatternHistory() {
  patternHistory.value = [];
  clearPatternHistoryInStore().catch(() => {
    // Ignore persistence error, keep in-memory history.
  });
}

function handleSearch() {
  if (!searchPath.value || !searchPattern.value) return;

  savePathHistory();
  savePatternHistory();

  emit('search', {
    ...buildParams(),
  });
}

function handleCancel() {
  emit('cancel');
}

function buildParams() {
  return {
    path: searchPath.value,
    pattern: searchPattern.value,
    case_sensitive: caseSensitive.value,
    whole_word: wholeWord.value,
    use_regex: useRegex.value,
    literal: literal.value,
    multiline: multiline.value,
    before_context: beforeContext.value,
    after_context: afterContext.value,
    engine: engine.value,
    binary_policy: binaryPolicy.value,
    max_depth: maxDepth.value,
    file_types: fileTypes.value,
    exclude_patterns: excludePatterns.value,
    page_size: pageSize.value,
    max_results: maxResults.value,
    timeout_seconds: timeoutSeconds.value,
  };
}

function emitParamsChange() {
  emit('params-change', buildParams());
}

function applySelectedProfile() {
  if (!selectedProfileId.value) return;
  emit('apply-profile', selectedProfileId.value);
}

function handleProfileSelection(profileId: string) {
  selectedProfileId.value = profileId;
  applySelectedProfile();
}

function saveCurrentAsPreset() {
  const name = newPresetName.value.trim();
  if (!name) return;
  emit('save-preset', name, buildParams());
  newPresetName.value = '';
}

function applySelectedPreset() {
  if (!selectedPresetId.value) return;
  emit('apply-preset', selectedPresetId.value);
}

function deleteSelectedPreset() {
  if (!selectedPresetId.value) return;
  emit('delete-preset', selectedPresetId.value);
  selectedPresetId.value = '';
}

watch(
  () => props.activeProfileId,
  (next) => {
    if (next) selectedProfileId.value = next;
  },
  { immediate: true }
);

watch(
  () => [
    searchPath.value,
    searchPattern.value,
    caseSensitive.value,
    wholeWord.value,
    useRegex.value,
    literal.value,
    multiline.value,
    beforeContext.value,
    afterContext.value,
    engine.value,
    binaryPolicy.value,
    maxDepth.value,
    fileTypes.value.join('\n'),
    excludePatterns.value.join('\n'),
    pageSize.value,
    maxResults.value,
    timeoutSeconds.value,
  ],
  () => {
    emitParamsChange();
  }
);
</script>

<template>
  <div
    class="relative bg-slate-950 border border-slate-800 rounded-lg p-3 mb-3"
    :class="{ 'ring-2 ring-cyan-500/60': isDragging }"
  >
    <div
      v-if="isDragging"
      class="absolute inset-0 bg-cyan-500/10 rounded-md flex items-center justify-center pointer-events-none z-50"
    >
      <p class="text-cyan-300 text-sm">Drop file or folder here</p>
    </div>
    <div class="space-y-2">
      <div class="flex flex-wrap gap-2 items-center">
        <div class="flex items-center gap-1">
          <span class="text-[11px] uppercase tracking-wide text-slate-400">Profile</span>
          <div class="relative">
            <select
              v-model="selectedProfileId"
              class="h-8 px-2 pr-8 text-xs border border-slate-700 bg-slate-900 text-slate-100 rounded appearance-none focus:outline-none focus:ring-2 focus:ring-cyan-500/40 focus:border-cyan-500"
              @change="applySelectedProfile"
            >
              <option v-for="profile in searchProfiles ?? []" :key="profile.id" :value="profile.id">
                {{ profile.name }}
              </option>
            </select>
            <span class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 text-[10px]">▼</span>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <span class="text-[11px] uppercase tracking-wide text-slate-400">Presets</span>
          <div class="relative">
            <select
              v-model="selectedPresetId"
              class="h-8 min-w-[160px] px-2 pr-8 text-xs border border-slate-700 bg-slate-900 text-slate-100 rounded appearance-none focus:outline-none focus:ring-2 focus:ring-cyan-500/40 focus:border-cyan-500"
            >
              <option value="">Select preset...</option>
              <option v-for="preset in savedSearches ?? []" :key="preset.id" :value="preset.id">
                {{ preset.name }}
              </option>
            </select>
            <span class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 text-[10px]">▼</span>
          </div>
          <button
            type="button"
            class="h-8 px-2 rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 text-xs"
            @click="applySelectedPreset"
          >
            Apply
          </button>
          <button
            type="button"
            class="h-8 px-2 rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-red-500/20 hover:border-red-400 text-xs"
            @click="deleteSelectedPreset"
          >
            Delete
          </button>
        </div>
        <div class="flex items-center gap-1">
          <input
            v-model="newPresetName"
            class="h-8 px-2 text-xs border border-slate-700 bg-slate-900 text-slate-100 rounded min-w-[140px]"
            placeholder="New preset name"
          />
          <button
            type="button"
            class="h-8 px-2 rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 text-xs"
            @click="saveCurrentAsPreset"
          >
            Save preset
          </button>
        </div>
      </div>
      <div class="text-xs uppercase tracking-wide text-slate-400">Search Workspace</div>
      <div class="flex gap-2 items-start">
        <PathSelector
          v-model="searchPath"
          :path-history="pathHistory"
          :history-limit="historyLimit ?? MAX_HISTORY"
          @save-history="savePathHistory"
          @clear-history="clearPathHistory"
        />
      </div>

      <div class="flex gap-2">
        <PatternInput
          v-model="searchPattern"
          :pattern-history="patternHistory"
          :focus-signal="focusPatternSignal"
          :history-limit="historyLimit ?? MAX_HISTORY"
          @save-history="savePatternHistory"
          @clear-history="clearPatternHistory"
          @search="handleSearch"
        />

        <button
          @click="isSearching ? handleCancel() : handleSearch()"
          class="px-3 h-9 rounded-md border text-sm min-w-[132px] justify-center transition-colors flex items-center gap-1.5 disabled:opacity-50"
          :class="
            isSearching
              ? 'bg-red-500/20 border-red-400/50 text-red-200 hover:bg-red-500/30'
              : 'bg-cyan-500/20 border-cyan-400/50 text-cyan-200 hover:bg-cyan-500/30'
          "
          :disabled="!isSearching && (!searchPath || !searchPattern)"
        >
          <template v-if="isSearching">
            <div
              class="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full"
            ></div>
            Cancel
          </template>
          <template v-else>
            <MagnifyingGlassIcon class="w-4 h-4" />
            Search
          </template>
        </button>
      </div>

      <AdvancedOptions
        v-model:case-sensitive="caseSensitive"
        v-model:whole-word="wholeWord"
        v-model:use-regex="useRegex"
        v-model:literal="literal"
        v-model:multiline="multiline"
        v-model:before-context="beforeContext"
        v-model:after-context="afterContext"
        v-model:engine="engine"
        v-model:binary-policy="binaryPolicy"
        v-model:max-depth="maxDepth"
        v-model:file-types="fileTypes"
        v-model:exclude-patterns="excludePatterns"
        v-model:page-size="pageSize"
        v-model:max-results="maxResults"
        v-model:timeout-seconds="timeoutSeconds"
        :profile-id="selectedProfileId"
        :profiles="(searchProfiles ?? []).map((p) => ({ id: p.id, name: p.name }))"
        @apply-profile="handleProfileSelection"
      />
    </div>
  </div>
</template>

<style scoped>
.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
