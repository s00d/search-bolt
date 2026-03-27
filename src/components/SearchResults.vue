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

const localFilter = ref('');
const showContextLines = ref(true);
const symbolOnly = ref(false);
const normalizedFilter = computed(() => localFilter.value.trim().toLowerCase());
const normalizedGlobalPattern = computed(() => props.globalPattern?.trim() ?? '');
const hasResults = computed(() => props.results.length > 0);
const hasMoreResults = computed(() => !!props.hasMoreServerResults);

function cleanUpCode(content: string): string {
  return content
    .replace(/^\r?\n+/, '')
    .replace(/\\\\u([0-9a-fA-F]{4})/g, (_, code) => String.fromCharCode(parseInt(code, 16)))
    .replace(/\\u([0-9a-fA-F]{4})/g, (_, code) => String.fromCharCode(parseInt(code, 16)))
    .replace(/\\([^\\])/g, '$1')
    .replace(/^\s+/, '');
}

function isSymbolLike(result: SearchResult): boolean {
  const lowerPath = result.path.toLowerCase();
  const extOk = ['.rs', '.ts', '.tsx', '.js', '.jsx', '.vue'].some((ext) =>
    lowerPath.endsWith(ext)
  );
  if (!extOk) return false;
  const line = cleanUpCode(result.content);
  return /(fn\s+\w+|struct\s+\w+|enum\s+\w+|impl\s+|class\s+\w+|interface\s+\w+|type\s+\w+|function\s+\w+|const\s+\w+\s*=\s*\(.*\)\s*=>)/.test(
    line
  );
}

const filteredResults = computed(() => {
  const query = normalizedFilter.value;
  return props.results.filter((result) => {
    if (symbolOnly.value && !isSymbolLike(result)) return false;
    if (!query) return true;
    const haystack =
      `${result.path} ${result.line_number} ${cleanUpCode(result.content)}`.toLowerCase();
    return haystack.includes(query);
  });
});

const groupedResults = computed(() => {
  const groups = new Map<string, SearchResult[]>();
  for (const item of filteredResults.value) {
    const current = groups.get(item.path) ?? [];
    current.push(item);
    groups.set(item.path, current);
  }
  return Array.from(groups.entries()).map(([path, items]) => ({ path, items }));
});

watch(
  () => props.results,
  () => {
    localFilter.value = '';
  }
);

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
  const re = new RegExp(escapeRegex(query), 'gi');
  return highlightWithRegex(
    text,
    re,
    'result-match',
    'background: rgba(251, 191, 36, 0.32); color: #fff7db; border-radius: 2px; padding: 0 1px;'
  );
}

function highlightGlobal(text: string): string {
  const pattern = normalizedGlobalPattern.value;
  if (!pattern) return escapeHtml(text);
  try {
    const useRegex = props.globalUseRegex ?? false;
    const caseSensitive = props.globalCaseSensitive ?? false;
    const re = new RegExp(useRegex ? pattern : escapeRegex(pattern), caseSensitive ? 'g' : 'gi');
    return highlightWithRegex(text, re, 'result-match');
  } catch {
    return escapeHtml(text);
  }
}

function formatLine(line: string): string {
  if (normalizedFilter.value) return highlightLocal(line);
  if (normalizedGlobalPattern.value) return highlightGlobal(line);
  return escapeHtml(line);
}

function copyText(text: string) {
  navigator.clipboard.writeText(text);
}

function copyAllResults() {
  copyText(filteredResults.value.map((r) => cleanUpCode(r.content)).join('\n'));
}

function copyPath(path: string) {
  copyText(path);
}

function copyPathLine(result: SearchResult) {
  copyText(`${result.path}:${result.line_number}`);
}

function truncatePathLeading(path: string, maxLength = 90): string {
  if (path.length <= maxLength) return path;
  return `...${path.slice(-(maxLength - 3))}`;
}

function formatMarkdown() {
  const lines: string[] = [`## Search Results (${filteredResults.value.length})`, ''];
  for (const group of groupedResults.value) {
    lines.push(`### \`${group.path}\``);
    lines.push('');
    for (const row of group.items) {
      lines.push(`- \`${row.path}:${row.line_number}\``);
      lines.push('```text');
      lines.push(cleanUpCode(row.content));
      lines.push('```');
      lines.push('');
    }
  }
  return lines.join('\n');
}

function copyFilteredAsMarkdown() {
  copyText(formatMarkdown());
}

async function openInEditor(result: SearchResult) {
  await invoke('open_result_in_editor', { path: result.path, line: result.line_number, column: 1 });
}

async function reveal(result: SearchResult) {
  await invoke('reveal_in_file_manager', { path: result.path });
}

async function openTerminal(result: SearchResult) {
  await invoke('open_terminal_at_path', { path: result.path });
}

function showMoreResults() {
  emit('load-more');
}

const isSlowSearch = computed(() => (props.stats?.elapsed_ms ?? 0) > 1200);
const hasSkippedFiles = computed(() => (props.stats?.files_skipped ?? 0) > 0);
</script>

<template>
  <div v-if="hasResults" class="bg-slate-950 border border-slate-800 rounded-lg p-2.5">
    <div class="mb-2 pb-2 border-b border-slate-800 space-y-1.5">
      <div class="flex justify-between items-center gap-2 flex-wrap">
        <h2 class="text-[11px] uppercase tracking-wide text-slate-300">
          Results: {{ filteredResults.length }}
          <span class="text-slate-500">(files {{ groupedResults.length }})</span>
        </h2>
        <div class="flex items-center gap-1.5">
          <button
            @click="showContextLines = !showContextLines"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200"
          >
            {{ showContextLines ? 'Context on' : 'Context off' }}
          </button>
          <button
            @click="symbolOnly = !symbolOnly"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200"
          >
            {{ symbolOnly ? 'Symbol only: on' : 'Symbol only: off' }}
          </button>
          <button
            @click="copyAllResults"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200"
          >
            Copy filtered
          </button>
          <button
            @click="copyFilteredAsMarkdown"
            class="h-7 px-2.5 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200"
          >
            Copy MD
          </button>
        </div>
      </div>
      <input
        v-model="localFilter"
        data-field-id="local-filter"
        type="text"
        class="w-full h-7 px-2 border border-slate-700 rounded bg-slate-900 text-xs text-slate-100 placeholder:text-slate-500"
        placeholder="Filter current results (path, line, content)..."
      />
      <p v-if="stats" class="text-[11px] text-slate-500">
        Page size {{ stats.page_size }}, in page {{ stats.returned_in_page }}, total matches
        {{ stats.total_matches }}, scanned {{ stats.files_scanned }}, skipped
        {{ stats.files_skipped }}, elapsed {{ stats.elapsed_ms }} ms.
      </p>
      <div v-if="stats" class="flex items-center gap-1.5 flex-wrap">
        <span
          class="px-1.5 py-0.5 rounded border text-[10px]"
          :class="
            isSlowSearch
              ? 'border-amber-500/40 bg-amber-500/10 text-amber-200'
              : 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200'
          "
        >
          {{ isSlowSearch ? 'Search speed: slow' : 'Search speed: good' }}
        </span>
        <span
          class="px-1.5 py-0.5 rounded border text-[10px]"
          :class="
            hasSkippedFiles
              ? 'border-amber-500/40 bg-amber-500/10 text-amber-200'
              : 'border-slate-600 bg-slate-800 text-slate-300'
          "
        >
          {{ hasSkippedFiles ? 'Skipped files detected' : 'No skipped files' }}
        </span>
      </div>
    </div>

    <div class="space-y-2">
      <div
        v-for="group in groupedResults"
        :key="group.path"
        class="border border-slate-800 rounded bg-slate-900/70"
      >
        <div class="px-2 py-1.5 border-b border-slate-800 flex items-center justify-between gap-2">
          <p class="min-w-0 flex-1 truncate text-xs font-mono text-slate-300" :title="group.path">
            {{ group.path }}
          </p>
          <div class="flex items-center gap-1">
            <button
              class="h-6 px-2 rounded border border-slate-700 text-[11px] text-slate-300"
              @click="copyPath(group.path)"
            >
              Copy path
            </button>
          </div>
        </div>
        <div class="space-y-1 p-1.5">
          <div
            v-for="result in group.items"
            :key="`${result.path}:${result.line_number}:${result.content}`"
            class="border border-slate-800 rounded p-1.5"
          >
            <div class="flex items-center justify-between gap-2 mb-1">
              <p
                class="min-w-0 flex-1 truncate text-xs font-mono"
                :title="`${result.path}:${result.line_number}`"
              >
                <span class="text-slate-400">{{ truncatePathLeading(result.path) }}</span
                ><span class="text-cyan-300">:{{ result.line_number }}</span>
              </p>
              <div class="flex items-center gap-1">
                <button
                  class="h-6 px-2 rounded border border-slate-700 text-[11px] text-slate-300"
                  @click="openInEditor(result)"
                >
                  Open
                </button>
                <button
                  class="h-6 px-2 rounded border border-slate-700 text-[11px] text-slate-300"
                  @click="copyPathLine(result)"
                >
                  Copy path:line
                </button>
                <button
                  class="h-6 px-2 rounded border border-slate-700 text-[11px] text-slate-300"
                  @click="reveal(result)"
                >
                  Reveal
                </button>
                <button
                  class="h-6 px-2 rounded border border-slate-700 text-[11px] text-slate-300"
                  @click="openTerminal(result)"
                >
                  Terminal
                </button>
              </div>
            </div>
            <div
              class="bg-slate-950 border border-slate-800 p-1.5 text-xs text-slate-200 overflow-x-auto rounded"
            >
              <div
                v-if="showContextLines"
                v-for="(line, idx) in result.before_context ?? []"
                :key="`before-${result.path}-${result.line_number}-${idx}`"
                class="block text-slate-500"
              >
                <span class="mr-2 text-slate-600">{{
                  result.line_number - (result.before_context?.length ?? 0) + idx
                }}</span>
                <span v-html="formatLine(line)" />
              </div>
              <div class="block" v-html="formatLine(cleanUpCode(result.content))" />
              <div
                v-if="showContextLines"
                v-for="(line, idx) in result.after_context ?? []"
                :key="`after-${result.path}-${result.line_number}-${idx}`"
                class="block text-slate-500"
              >
                <span class="mr-2 text-slate-600">{{ result.line_number + idx + 1 }}</span>
                <span v-html="formatLine(line)" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="mt-2.5 flex items-center justify-between gap-2">
      <p class="text-[11px] text-slate-500">Showing {{ filteredResults.length }} results.</p>
      <button
        v-if="hasMoreResults"
        @click="showMoreResults"
        :disabled="isLoadingMore"
        class="h-7 px-3 text-[11px] rounded border border-slate-700 bg-slate-800 text-slate-200"
      >
        {{ isLoadingMore ? 'Loading...' : 'More' }}
      </button>
    </div>
  </div>
  <div v-else-if="!isSearching" class="text-center text-slate-500 mt-6 text-sm">
    No results found. Try changing pattern or filters.
  </div>
</template>

<style scoped>
::deep(.result-match) {
  background: rgba(34, 211, 238, 0.28);
  color: #ecfeff;
  border-radius: 2px;
  padding: 0 1px;
}
</style>
