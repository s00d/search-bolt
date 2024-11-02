<script setup lang="ts">
import { ref } from 'vue';
import SearchForm from './components/SearchForm.vue';
import SearchResults from './components/SearchResults.vue';
import { invoke } from "@tauri-apps/api/core";

interface SearchResult {
  path: string;
  line_number: number;
  content: string;
}

const results = ref<SearchResult[]>([]);
const isSearching = ref(false);
const error = ref<string | null>(null);
let abortController: AbortController | null = null;

async function handleSearch(params: {
  path: string;
  pattern: string;
  case_sensitive: boolean;
  whole_word: boolean;
  use_regex: boolean;
  max_depth?: number;
  file_types: string[];
  exclude_patterns: string[];
}) {
  isSearching.value = true;
  error.value = null;
  abortController = new AbortController();

  try {
    results.value = await invoke('search', {
      params,
      signal: abortController.signal
    });
  } catch (e) {
    if ((e as Error)?.name === 'AbortError') {
      results.value = [];
      return;
    }
    error.value = e as string;
    results.value = [];
  } finally {
    isSearching.value = false;
    abortController = null;
  }
}

function handleCancel() {
  if (abortController) {
    abortController.abort();
    isSearching.value = false;
  }
}
</script>

<template>
  <div class="min-h-screen bg-gray-100 p-6">
    <div class="max-w-6xl mx-auto">
      <SearchForm
        :is-searching="isSearching"
        @search="handleSearch"
        @cancel="handleCancel"
      />

      <div v-if="error" class="bg-red-50 border-l-4 border-red-400 p-4 mb-2">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3">
            <p class="text-sm text-red-700">
              {{ error }}
            </p>
          </div>
        </div>
      </div>

      <SearchResults
        :results="results"
        :is-searching="isSearching"
      />
    </div>
  </div>
</template>
