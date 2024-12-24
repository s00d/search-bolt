<script setup lang="ts">
import { computed } from 'vue';

interface SearchResult {
  path: string;
  line_number: number;
  content: string;
}

const props = defineProps<{
  results: SearchResult[];
  isSearching: boolean;
}>();

const hasResults = computed(() => props.results.length > 0);

function cleanUpCode(content: string): string {
  return content
    // Декодируем Unicode escape-последовательности со слешем
    .replace(/\\\\u([0-9a-fA-F]{4})/g, (_, code) => String.fromCharCode(parseInt(code, 16)))
    // Декодируем обычные Unicode escape-последовательности
    .replace(/\\u([0-9a-fA-F]{4})/g, (_, code) => String.fromCharCode(parseInt(code, 16)))
    // Декодируем слеши в начале символов
    .replace(/\\([^\\])/g, '$1');
}

function copyToClipboard(text: string) {
  const decodedContent = cleanUpCode(text);
  navigator.clipboard.writeText(decodedContent);
}

function copyAllResults() {
  const allContent = props.results.map(result => cleanUpCode(result.content)).join('\n');
  copyToClipboard(allContent);
}

function truncatePath(path: string): string {
  const maxLength = 100;
  return path.length > maxLength ? '...' + path.slice(-(maxLength - 3)) : path;
}
</script>

<template>
  <div v-if="hasResults" class="bg-white rounded-lg shadow-md p-3">
    <div class="flex justify-between items-center mb-2">
      <h2 class="text-sm font-medium">Results ({{ results.length }})</h2>
      <button
        @click="copyAllResults"
        class="px-2 py-1 text-xs bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
      >
        Copy All
      </button>
    </div>

    <div class="space-y-1">
      <div
        v-for="(result, index) in results"
        :key="index"
        class="border p-2 hover:border-gray-300 transition-colors"
      >
        <div class="flex justify-between text-xs mb-1">
          <span class="text-gray-500 truncate" :title="result.path">
            {{ truncatePath(result.path) }}:{{ result.line_number }}
          </span>
          <button
            @click="copyToClipboard(result.content)"
            class="text-blue-500 hover:text-white hover:bg-blue-600"
          >
            Copy
          </button>
        </div>

        <div class="result bg-gray-100 p-1.5 text-xs text-gray-300 overflow-x-auto">
          <div class="data block text-gray-900">{{ cleanUpCode(result.content) }}</div>
        </div>
      </div>
    </div>
  </div>

  <div
    v-else-if="!isSearching"
    class="text-center text-gray-500 mt-4 text-sm"
  >
    No results found
  </div>
</template>

<style scoped>
.result {
  max-height: 200px;
  padding: 0.5rem;
  border-radius: 4px;
  overflow-y: auto;
}

.result .data {
  font-size: 0.875rem;
  line-height: 1.25rem;
  word-wrap: break-word;
  white-space: pre-wrap;
  font-weight: bold;
  margin: 0;
}

div:hover {
  border-color: #5e5e5e;
}

:deep(.text-gray-300) {
  color: #d4d4d4;
}
</style>
