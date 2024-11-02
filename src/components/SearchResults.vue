<script setup lang="ts">
import { computed } from 'vue';
import hljs from 'highlight.js';
import 'highlight.js/styles/github-dark.css';

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

function highlightCode(content: string): string {
  // Сначала раскодируем HTML-сущности
  const decodedContent = cleanUpCode(content);

  return hljs.highlightAuto(decodedContent).value;
}

function copyToClipboard(text: string) {
  const decodedContent = cleanUpCode(text);
  // Копируем оригинальный текст без экранирования
  navigator.clipboard.writeText(decodedContent);
}

function copyAllResults() {
  const allContent = props.results
    .map(result => cleanUpCode(result.content))
    .join('\n');
  copyToClipboard(allContent);
}

function truncatePath(path: string): string {
  const maxLength = 50;
  return path.length > maxLength
    ? '...' + path.slice(-(maxLength - 3))
    : path;
}
</script>

<template>
  <div v-if="hasResults" class="bg-white rounded-lg shadow-md p-4">
    <div class="flex justify-between items-center mb-3">
      <h2 class="text-lg font-medium">Results ({{ results.length }})</h2>
      <button
        @click="copyAllResults"
        class="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
      >
        Copy All Results
      </button>
    </div>

    <div class="space-y-2">
      <div
        v-for="(result, index) in results"
        :key="index"
        class="border rounded-lg p-3 hover:border-gray-300 transition-colors"
      >
        <div class="flex items-center justify-between text-sm mb-1.5">
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-gray-600 truncate" :title="result.path">
              {{ truncatePath(result.path) }}
            </span>
            <span class="text-gray-400 shrink-0">:{{ result.line_number }}</span>
          </div>
          <button
            @click="copyToClipboard(result.content)"
            class="text-blue-500 hover:text-blue-600 shrink-0 ml-2"
          >
            Copy
          </button>
        </div>

        <div class="relative">
          <pre class="bg-gray-900 rounded p-2.5 overflow-x-auto whitespace-pre-wrap break-all"><code v-html="highlightCode(result.content)" class="text-sm leading-relaxed"></code></pre>
        </div>
      </div>
    </div>
  </div>

  <div
    v-else-if="!isSearching"
    class="text-center text-gray-500 mt-6"
  >
    No results found
  </div>
</template>

<style scoped>
pre {
  max-height: 300px;
}

pre code {
  word-wrap: break-word;
  white-space: pre-wrap;
  color: #d9d9d9;
}

:deep(.hljs) {
  padding: 0;
}
</style>
