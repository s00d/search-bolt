<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

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

const props = defineProps<{
  results: SearchResult[];
  isSearching: boolean;
  stats?: SearchPageStats | null;
  globalPattern?: string;
  globalUseRegex?: boolean;
  globalCaseSensitive?: boolean;
  hasMoreServerResults?: boolean;
  isLoadingMore?: boolean;
}>();

const emit = defineEmits<{
  'load-more': [];
}>();

const hasResults = computed(() => props.results.length > 0);
const localFilter = ref('');
const showContextLines = ref(true);

const normalizedFilter = computed(() => localFilter.value.trim().toLowerCase());
const normalizedGlobalPattern = computed(() => props.globalPattern?.trim() ?? '');

const filteredResults = computed(() => {
  const query = normalizedFilter.value;
  if (!query) return props.results;

  return props.results.filter((result) => {
    const haystack = `${result.path} ${result.line_number} ${cleanUpCode(result.content)}`.toLowerCase();
    return haystack.includes(query);
  });
});

const visibleResults = computed(() => filteredResults.value);
const hasMoreResults = computed(() => !!props.hasMoreServerResults);

watch(
  () => props.results,
  () => {
    localFilter.value = '';
  }
);

function cleanUpCode(content: string): string {
  return content
    // Remove leading newlines that may come from the search engine.
    .replace(/^\r?\n+/, '')
    // Decode escaped Unicode sequences with double slash.
    .replace(/\\\\u([0-9a-fA-F]{4})/g, (_, code) => String.fromCharCode(parseInt(code, 16)))
    // Decode standard Unicode escape sequences.
    .replace(/\\u([0-9a-fA-F]{4})/g, (_, code) => String.fromCharCode(parseInt(code, 16)))
    // Decode escaped punctuation like \: or \/
    .replace(/\\([^\\])/g, '$1')
    // For preview readability, trim leading indentation.
    .replace(/^\s+/, '');
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function highlightWithRegex(text: string, re: RegExp, className: string, style?: string): string {
  let out = '';
  let last = 0;
  re.lastIndex = 0;

  while (true) {
    const match = re.exec(text);
    if (!match) break;

    const start = match.index ?? 0;
    const matched = match[0] ?? '';

    out += escapeHtml(text.slice(last, start));
    const styleAttr = style ? ` style="${style}"` : '';
    out += `<mark class="${className}"${styleAttr}>${escapeHtml(matched)}</mark>`;

    last = start + matched.length;

    // Prevent infinite loops for patterns that can match empty strings.
    if (matched.length === 0) {
      re.lastIndex = start + 1;
      if (re.lastIndex > text.length) break;
    }
  }

  out += escapeHtml(text.slice(last));
  return out;
}

function highlightLocal(text: string): string {
  const query = normalizedFilter.value;
  if (!query) return escapeHtml(text);
  const re = new RegExp(escapeRegex(query), 'gi'); // local filter is always case-insensitive
  return highlightWithRegex(
    text,
    re,
    'result-match',
    'background: rgba(251, 191, 36, 0.32); color: #fff7db; border-radius: 2px; padding: 0 1px;',
  );
}

function highlightGlobal(text: string): string {
  const pattern = normalizedGlobalPattern.value;
  if (!pattern) return escapeHtml(text);

  const useRegex = props.globalUseRegex ?? false;
  const caseSensitive = props.globalCaseSensitive ?? false;

  try {
    if (useRegex) {
      const flags = caseSensitive ? 'g' : 'gi';
      const re = new RegExp(pattern, flags);
      return highlightWithRegex(text, re, 'result-match');
    }

    const escaped = escapeRegex(pattern);
    const flags = caseSensitive ? 'g' : 'gi';
    const re = new RegExp(escaped, flags);
    return highlightWithRegex(text, re, 'result-match');
  } catch {
    return escapeHtml(text);
  }
}

function copyToClipboard(text: string) {
  const decodedContent = cleanUpCode(text);
  navigator.clipboard.writeText(decodedContent);
}

function copyAllResults() {
  const allContent = filteredResults.value.map(result => cleanUpCode(result.content)).join('\n');
  copyToClipboard(allContent);
}

function escapeMarkdownInline(value: string): string {
  return value.replace(/`/g, '\\`');
}

function formatResultAsMarkdown(result: SearchResult): string {
  const content = cleanUpCode(result.content);
  return [
    `- \`${escapeMarkdownInline(result.path)}:${result.line_number}\``,
    '```text',
    content,
    '```',
  ].join('\n');
}

function formatFilteredAsMarkdown(): string {
  const body = filteredResults.value.map(formatResultAsMarkdown).join('\n\n');
  return [
    `## Search Results (${filteredResults.value.length})`,
    '',
    body,
  ].join('\n');
}

function copyFilteredAsMarkdown() {
  navigator.clipboard.writeText(formatFilteredAsMarkdown());
}

function copyResultAsMarkdown(result: SearchResult) {
  navigator.clipboard.writeText(formatResultAsMarkdown(result));
}

async function openInEditor(result: SearchResult) {
  try {
    await invoke('open_result_in_editor', {
      path: result.path,
      line: result.line_number,
      column: 1,
    });
  } catch (error) {
    console.error('Failed to open file in editor:', error);
  }
}

function truncatePath(path: string): string {
  const maxLength = 120;
  return path.length > maxLength ? '...' + path.slice(-(maxLength - 3)) : path;
}

function showMoreResults() {
  emit('load-more');
}

function formatContextLine(line: string): string {
  if (normalizedFilter.value) return highlightLocal(line);
  if (normalizedGlobalPattern.value) return highlightGlobal(line);
  return escapeHtml(line);
}

const isSlowSearch = computed(() => (props.stats?.elapsed_ms ?? 0) > 1200);
const hasSkippedFiles = computed(() => (props.stats?.files_skipped ?? 0) > 0);
</script>

<template>
  <div v-if="hasResults" class="bg-slate-950 border border-slate-800 rounded-lg p-2.5">
    <div class="mb-2 pb-2 border-b border-slate-800 space-y-1.5">
      <div class="flex justify-between items-center gap-2 flex-wrap">
        <h2 class="text-[11px] uppercase tracking-wide text-slate-300">
          Results: {{ visibleResults.length }} / {{ filteredResults.length }} <span class="text-slate-500">(total {{ results.length }})</span>
        </h2>
        <div class="flex items-center gap-1.5">
          <button
            @click="showContextLines = !showContextLines"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 transition-colors"
            :title="showContextLines ? 'Hide context lines' : 'Show context lines'"
          >
            {{ showContextLines ? 'Context on' : 'Context off' }}
          </button>
          <button
            @click="copyAllResults"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 transition-colors"
          >
            Copy filtered
          </button>
          <button
            @click="copyFilteredAsMarkdown"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 transition-colors"
            title="Copy filtered results as Markdown"
          >
            Copy MD
          </button>
        </div>
      </div>
      <input
        v-model="localFilter"
        type="text"
        class="w-full h-7 px-2 border border-slate-700 rounded bg-slate-900 text-xs text-slate-100 placeholder:text-slate-500"
        placeholder="Filter current results (path, line, content)..."
      />
      <p v-if="stats" class="text-[11px] text-slate-500">
        Page size {{ stats.page_size }}, in page {{ stats.returned_in_page }}, total matches {{ stats.total_matches }},
        scanned {{ stats.files_scanned }}, skipped {{ stats.files_skipped }}, elapsed {{ stats.elapsed_ms }} ms.
      </p>
      <div v-if="stats" class="flex items-center gap-1.5 flex-wrap">
        <span
          class="px-1.5 py-0.5 rounded border text-[10px]"
          :class="isSlowSearch
            ? 'border-amber-500/40 bg-amber-500/10 text-amber-200'
            : 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200'"
          :title="isSlowSearch
            ? 'Search took longer than 1200ms. Typical reasons: complex regex, large context, many scanned files. Try smaller page size/context or simpler pattern.'
            : 'Search completed within normal threshold (<1200ms).'"
        >
          {{ isSlowSearch ? 'Search speed: slow' : 'Search speed: good' }}
        </span>
        <span
          class="px-1.5 py-0.5 rounded border text-[10px]"
          :class="hasSkippedFiles
            ? 'border-amber-500/40 bg-amber-500/10 text-amber-200'
            : 'border-slate-600 bg-slate-800 text-slate-300'"
          :title="hasSkippedFiles
            ? 'Some files were skipped (permission denied, invalid/binary data or policy constraints). Check binary policy and filesystem permissions.'
            : 'No files were skipped during this search pass.'"
        >
          {{ hasSkippedFiles ? 'Skipped files detected' : 'No skipped files' }}
        </span>
      </div>
    </div>

    <div class="space-y-1">
      <div
        v-for="(result, index) in visibleResults"
        :key="index"
        class="result-row border border-slate-800 bg-slate-900/70 p-2 pl-2.5 rounded hover:border-slate-600 transition-colors"
      >
        <div class="flex justify-between text-xs mb-1 items-center gap-2">
          <p
            v-if="!normalizedFilter"
            class="min-w-0 flex-1 truncate font-mono text-slate-300"
            :title="`${result.path}:${result.line_number}`"
          >
            <span class="text-slate-400">{{ truncatePath(result.path) }}</span
            ><span class="text-cyan-300">:{{ result.line_number }}</span>
          </p>
          <p
            v-else
            class="min-w-0 flex-1 truncate font-mono text-slate-300"
            :title="`${result.path}:${result.line_number}`"
            v-html="highlightLocal(`${truncatePath(result.path)}:${result.line_number}`)"
          />
          <div class="flex items-center gap-1">
            <button
              @click="openInEditor(result)"
              class="h-6 w-12 rounded border border-slate-700 text-[11px] text-slate-300 hover:text-white hover:border-cyan-400 hover:bg-cyan-500/20"
              title="Open in default editor"
            >
              Open
            </button>
            <button
              @click="copyToClipboard(result.content)"
              class="h-6 w-11 rounded border border-slate-700 text-[11px] text-slate-300 hover:text-white hover:border-cyan-400 hover:bg-cyan-500/20"
            >
              Copy
            </button>
            <button
              @click="copyResultAsMarkdown(result)"
              class="h-6 w-9 rounded border border-slate-700 text-[11px] text-slate-300 hover:text-white hover:border-cyan-400 hover:bg-cyan-500/20"
              title="Copy this result as Markdown"
            >
              MD
            </button>
          </div>
        </div>

        <div class="result bg-slate-950 border border-slate-800 p-1.5 text-xs text-slate-200 overflow-x-auto rounded">
          <div
            v-if="showContextLines"
            v-for="(line, idx) in (result.before_context ?? [])"
            :key="`before-${index}-${idx}`"
            class="data block text-slate-500"
          >
            <span class="mr-2 text-slate-600">{{ result.line_number - (result.before_context?.length ?? 0) + idx }}</span>
            <span v-html="formatContextLine(line)"></span>
          </div>
          <div
            v-if="normalizedFilter"
            class="data block"
            v-html="highlightLocal(cleanUpCode(result.content))"
          />
          <div
            v-else-if="normalizedGlobalPattern"
            class="data block"
            v-html="highlightGlobal(cleanUpCode(result.content))"
          />
          <div v-else class="data block">
            {{ cleanUpCode(result.content) }}
          </div>
          <div
            v-if="showContextLines"
            v-for="(line, idx) in (result.after_context ?? [])"
            :key="`after-${index}-${idx}`"
            class="data block text-slate-500"
          >
            <span class="mr-2 text-slate-600">{{ result.line_number + idx + 1 }}</span>
            <span v-html="formatContextLine(line)"></span>
          </div>
        </div>
      </div>
    </div>

    <div class="mt-2.5 flex items-center justify-between gap-2">
      <p class="text-[11px] text-slate-500">
        Showing {{ visibleResults.length }} of {{ filteredResults.length }} filtered results.
      </p>
      <button
        v-if="hasMoreResults"
        @click="showMoreResults"
        :disabled="isLoadingMore"
        class="h-7 px-3 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 transition-colors"
      >
        {{ isLoadingMore ? 'Loading...' : 'More' }}
      </button>
    </div>
  </div>

  <div
    v-else-if="!isSearching && !hasResults"
    class="text-center text-slate-500 mt-6 text-sm"
  >
    No results found. Try changing pattern or filters.
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
  font-size: 0.8rem;
  line-height: 1.2rem;
  word-wrap: break-word;
  white-space: pre-wrap;
  font-weight: 500;
  margin: 0;
}

.result-row {
  position: relative;
}

.result-row::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0.35rem;
  bottom: 0.35rem;
  width: 2px;
  border-radius: 999px;
  background: rgba(34, 211, 238, 0.28);
  opacity: 0;
  transition: opacity 140ms ease;
}

.result-row:hover::before {
  opacity: 1;
}

:deep(.result-match) {
  background: rgba(34, 211, 238, 0.28);
  color: #ecfeff;
  border-radius: 2px;
  padding: 0 1px;
}
</style>
