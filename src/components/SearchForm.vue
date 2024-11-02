<script setup lang="ts">
import {ref, onMounted, onUnmounted} from 'vue';
import { MagnifyingGlassIcon } from '@heroicons/vue/24/outline';
import PathSelector from './search/PathSelector.vue';
import PatternInput from './search/PatternInput.vue';
import AdvancedOptions from './search/AdvancedOptions.vue';
import SearchEngineSelector from './search/SearchEngineSelector.vue';
import {getCurrentWebview} from "@tauri-apps/api/webview";
import type {UnlistenFn} from "@tauri-apps/api/event";

const MAX_HISTORY = 20;

defineProps<{
  isSearching: boolean;
}>();

const emit = defineEmits<{
  search: [params: {
    path: string;
    pattern: string;
    engine: string;
    case_sensitive: boolean;
    whole_word: boolean;
    use_regex: boolean;
    max_depth?: number;
    file_types: string[];
    exclude_patterns: string[];
    max_results: number;
    timeout_seconds: number;
  }];
  cancel: [];
}>();

const searchPath = ref('');
const searchPattern = ref('');
const searchEngine = ref('ripgrep');
const caseSensitive = ref(false);
const wholeWord = ref(false);
const useRegex = ref(false);
const maxDepth = ref<number>();
const fileTypes = ref<string[]>([]);
const excludePatterns = ref<string[]>([]);
const pathHistory = ref<string[]>([]);
const patternHistory = ref<string[]>([]);
const isDragging = ref(false);
const maxResults = ref(100);
const timeoutSeconds = ref(60);

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

onUnmounted(() => {
  if (dragDropUnlisten) {
    dragDropUnlisten();
  }
});

function loadHistory() {
  const paths = localStorage.getItem('pathHistory');
  const patterns = localStorage.getItem('patternHistory');

  if (paths) {
    pathHistory.value = JSON.parse(paths);
  }
  if (patterns) {
    patternHistory.value = JSON.parse(patterns);
  }
}

function savePathHistory() {
  if (!searchPath.value) return;

  const newHistory = [searchPath.value, ...pathHistory.value.filter(p => p !== searchPath.value)];
  pathHistory.value = newHistory.slice(0, MAX_HISTORY);
  localStorage.setItem('pathHistory', JSON.stringify(pathHistory.value));
}

function savePatternHistory() {
  if (!searchPattern.value) return;

  const newHistory = [searchPattern.value, ...patternHistory.value.filter(p => p !== searchPattern.value)];
  patternHistory.value = newHistory.slice(0, MAX_HISTORY);
  localStorage.setItem('patternHistory', JSON.stringify(patternHistory.value));
}

function clearPathHistory() {
  pathHistory.value = [];
  localStorage.removeItem('pathHistory');
}

function clearPatternHistory() {
  patternHistory.value = [];
  localStorage.removeItem('patternHistory');
}

function handleSearch() {
  if (!searchPath.value || !searchPattern.value) return;

  savePathHistory();
  savePatternHistory();

  emit('search', {
    path: searchPath.value,
    pattern: searchPattern.value,
    engine: searchEngine.value,
    case_sensitive: caseSensitive.value,
    whole_word: wholeWord.value,
    use_regex: useRegex.value,
    max_depth: maxDepth.value,
    file_types: fileTypes.value,
    exclude_patterns: excludePatterns.value,
    max_results: maxResults.value,
    timeout_seconds: timeoutSeconds.value,
  });
}

function handleCancel() {
  emit('cancel');
}
</script>

<template>
  <div class="bg-white rounded-lg shadow-sm p-3 mb-4" :class="{ 'ring-1 ring-blue-500 ring-opacity-80': isDragging }">
    <div
      v-if="isDragging"
      class="absolute inset-0 bg-blue-50 bg-opacity-50 rounded-md flex items-center justify-center pointer-events-none z-50"
    >
      <p class="text-blue-600 text-sm">Drop file or folder here</p>
    </div>
    <div class="space-y-2">
      <div class="flex gap-2 items-start">
        <SearchEngineSelector v-model="searchEngine" />
        <div class="flex-1">
          <PathSelector
            v-model="searchPath"
            :path-history="pathHistory"
            @save-history="savePathHistory"
            @clear-history="clearPathHistory"
          />
        </div>
      </div>

      <div class="flex gap-2">
        <PatternInput
          v-model="searchPattern"
          :pattern-history="patternHistory"
          @save-history="savePatternHistory"
          @clear-history="clearPatternHistory"
          @search="handleSearch"
        />

        <button
          @click="isSearching ? handleCancel() : handleSearch()"
          class="px-3 h-9 text-white rounded-md hover:bg-opacity-90 flex items-center gap-1.5 disabled:opacity-50 text-sm min-w-[120px] justify-center transition-colors"
          :class="isSearching ? 'bg-red-500 hover:bg-red-600' : 'bg-blue-500 hover:bg-blue-600'"
          :disabled="(!isSearching && (!searchPath || !searchPattern))"
        >
          <template v-if="isSearching">
            <div class="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full"></div>
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
        v-model:max-depth="maxDepth"
        v-model:file-types="fileTypes"
        v-model:exclude-patterns="excludePatterns"
        v-model:max-results="maxResults"
        v-model:timeout-seconds="timeoutSeconds"
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
