<script setup lang="ts">
import { ref } from 'vue';
import { AdjustmentsHorizontalIcon } from '@heroicons/vue/24/outline';
import BaseSwitch from '../BaseSwitch.vue';
import ThemeToggle from './ThemeToggle.vue';
import BaseDisclosure from '../BaseDisclosure.vue';

const props = defineProps<{
  caseSensitive: boolean;
  wholeWord: boolean;
  useRegex: boolean;
  literal: boolean;
  multiline: boolean;
  beforeContext: number;
  afterContext: number;
  engine: 'rust_regex' | 'pcre2';
  binaryPolicy: 'skip' | 'lossy';
  maxDepth?: number;
  fileTypes: string[];
  excludePatterns: string[];
  pageSize: number;
  maxResults: number;
  timeoutSeconds: number;
}>();

const emit = defineEmits<{
  'update:caseSensitive': [value: boolean];
  'update:wholeWord': [value: boolean];
  'update:useRegex': [value: boolean];
  'update:literal': [value: boolean];
  'update:multiline': [value: boolean];
  'update:beforeContext': [value: number];
  'update:afterContext': [value: number];
  'update:engine': [value: 'rust_regex' | 'pcre2'];
  'update:binaryPolicy': [value: 'skip' | 'lossy'];
  'update:maxDepth': [value: number | undefined];
  'update:fileTypes': [value: string[]];
  'update:excludePatterns': [value: string[]];
  'update:pageSize': [value: number];
  'update:maxResults': [value: number];
  'update:timeoutSeconds': [value: number];
}>();

const fileTypeInput = ref('');
const excludeInput = ref('');

function addFileType() {
  if (fileTypeInput.value && !props.fileTypes.includes(fileTypeInput.value)) {
    emit('update:fileTypes', [...props.fileTypes, fileTypeInput.value]);
    fileTypeInput.value = '';
  }
}

function removeFileType(type: string) {
  emit('update:fileTypes', props.fileTypes.filter(t => t !== type));
}

function addExcludePattern() {
  if (excludeInput.value && !props.excludePatterns.includes(excludeInput.value)) {
    emit('update:excludePatterns', [...props.excludePatterns, excludeInput.value]);
    excludeInput.value = '';
  }
}

function removeExcludePattern(pattern: string) {
  emit('update:excludePatterns', props.excludePatterns.filter(p => p !== pattern));
}
</script>

<template>
  <BaseDisclosure button-class="h-8 px-2 rounded-md border border-slate-700 bg-slate-900 text-slate-300 hover:bg-slate-800 text-xs">
    <template #button>
      <span class="flex items-center gap-1">
        <AdjustmentsHorizontalIcon class="w-4 h-4" />
        Advanced options
      </span>
    </template>

    <div class="space-y-3 p-3 rounded-md border border-slate-700 bg-slate-900/80 text-sm">
      <div class="flex flex-wrap gap-4">
        <BaseSwitch
          :model-value="caseSensitive"
          @update:model-value="$emit('update:caseSensitive', $event)"
          label="Case sensitive"
        />
        <BaseSwitch
          :model-value="wholeWord"
          @update:model-value="$emit('update:wholeWord', $event)"
          label="Whole word"
        />
        <BaseSwitch
          :model-value="useRegex"
          @update:model-value="$emit('update:useRegex', $event)"
          label="Use regex"
        />
        <BaseSwitch
          :model-value="literal"
          @update:model-value="$emit('update:literal', $event)"
          label="Literal"
        />
        <BaseSwitch
          :model-value="multiline"
          @update:model-value="$emit('update:multiline', $event)"
          label="Multiline"
        />

        <ThemeToggle />
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-4 gap-2">
        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Max depth:</span>
          <input
            :value="maxDepth"
            type="number"
            min="1"
            class="w-20 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @input="$emit('update:maxDepth', ($event.target as HTMLInputElement).value ? Number(($event.target as HTMLInputElement).value) : undefined)"
          />
        </div>

        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Max results:</span>
          <input
            :value="maxResults ?? 100"
            type="number"
            min="1"
            class="w-20 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @input="$emit('update:maxResults', Number(($event.target as HTMLInputElement).value) || 100)"
          />
        </div>

        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Page size:</span>
          <input
            :value="pageSize ?? 50"
            type="number"
            min="1"
            class="w-20 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @input="$emit('update:pageSize', Number(($event.target as HTMLInputElement).value) || 50)"
          />
        </div>

        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Timeout (s):</span>
          <input
            :value="timeoutSeconds ?? 60"
            type="number"
            min="1"
            class="w-20 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @input="$emit('update:timeoutSeconds', Number(($event.target as HTMLInputElement).value) || 60)"
          />
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-4 gap-2">
        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Before ctx:</span>
          <input
            :value="beforeContext ?? 0"
            type="number"
            min="0"
            class="w-20 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @input="$emit('update:beforeContext', Number(($event.target as HTMLInputElement).value) || 0)"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">After ctx:</span>
          <input
            :value="afterContext ?? 0"
            type="number"
            min="0"
            class="w-20 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @input="$emit('update:afterContext', Number(($event.target as HTMLInputElement).value) || 0)"
          />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Engine:</span>
          <select
            :value="engine"
            class="h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @change="$emit('update:engine', ($event.target as HTMLSelectElement).value as 'rust_regex' | 'pcre2')"
          >
            <option value="rust_regex">Rust regex</option>
            <option value="pcre2">PCRE2</option>
          </select>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-slate-300 whitespace-nowrap text-xs">Binary:</span>
          <select
            :value="binaryPolicy"
            class="h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @change="$emit('update:binaryPolicy', ($event.target as HTMLSelectElement).value as 'skip' | 'lossy')"
          >
            <option value="lossy">Lossy</option>
            <option value="skip">Skip</option>
          </select>
        </div>
      </div>

      <div>
        <div class="flex gap-2 mb-1">
          <input
            v-model="fileTypeInput"
            type="text"
            placeholder="File types (*.js, *.py)"
            class="flex-1 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @keyup.enter="addFileType"
          />
          <button
            @click="addFileType"
            class="px-3 h-8 text-xs bg-slate-800 border border-slate-700 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 rounded"
          >
            Add
          </button>
        </div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="type in fileTypes"
            :key="type"
            class="px-1.5 py-0.5 bg-slate-800 border border-slate-700 text-slate-200 rounded flex items-center gap-1 text-xs"
          >
            {{ type }}
            <button
              @click="removeFileType(type)"
              class="text-slate-400 hover:text-red-300"
            >
              ×
            </button>
          </span>
        </div>
      </div>

      <div>
        <div class="flex gap-2 mb-1">
          <input
            v-model="excludeInput"
            type="text"
            placeholder="Exclude patterns (node_modules, .git)"
            class="flex-1 h-8 px-2 text-xs border border-slate-700 bg-slate-800 text-slate-100 rounded focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
            @keyup.enter="addExcludePattern"
          />
          <button
            @click="addExcludePattern"
            class="px-3 h-8 text-xs bg-slate-800 border border-slate-700 text-slate-200 hover:bg-cyan-500/20 hover:border-cyan-400 rounded"
          >
            Add
          </button>
        </div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="pattern in excludePatterns"
            :key="pattern"
            class="px-1.5 py-0.5 bg-slate-800 border border-slate-700 text-slate-200 rounded flex items-center gap-1 text-xs"
          >
            {{ pattern }}
            <button
              @click="removeExcludePattern(pattern)"
              class="text-slate-400 hover:text-red-300"
            >
              ×
            </button>
          </span>
        </div>
      </div>
    </div>
  </BaseDisclosure>
</template>
