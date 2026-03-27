<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { ChevronDownIcon, FolderIcon, DocumentIcon } from '@heroicons/vue/24/outline';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { UnlistenFn } from '@tauri-apps/api/event';
import HistoryDropdown from './HistoryDropdown.vue';

const props = defineProps<{
  modelValue: string;
  pathHistory: string[];
  focusSignal?: number;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'save-history': [];
  'clear-history': [];
}>();

const showPathHistory = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);
let dragDropUnlisten: UnlistenFn | null = null;

onMounted(async () => {
  dragDropUnlisten = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'drop') {
      const paths = event.payload.paths;
      if (paths.length > 0) {
        emit('update:modelValue', paths[0]);
        emit('save-history');
      }
    }
  });
});

onUnmounted(() => {
  if (dragDropUnlisten) {
    dragDropUnlisten();
  }
});

watch(
  () => props.focusSignal,
  () => {
    if (inputRef.value) {
      inputRef.value.focus();
      inputRef.value.select();
    }
  },
  { immediate: true }
);

async function selectPath() {
  const selected = await open({
    directory: false,
    multiple: false,
  });
  if (selected) {
    emit('update:modelValue', selected as string);
    emit('save-history');
  }
}

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected) {
    emit('update:modelValue', selected as string);
    emit('save-history');
  }
}

function handleHistorySelect(path: string) {
  emit('update:modelValue', path);
  showPathHistory.value = false;
}

function toggleHistory() {
  showPathHistory.value = !showPathHistory.value;
  if (showPathHistory.value && inputRef.value) {
    inputRef.value.focus();
  }
}
</script>

<template>
  <div class="relative w-full">
    <div class="flex gap-1.5">
      <div class="relative flex-1">
        <input
          ref="inputRef"
          :value="modelValue"
          type="text"
          placeholder="Path to file or directory..."
          class="w-full h-9 pl-3 pr-9 text-sm border border-slate-700 bg-slate-900 text-slate-100 rounded-md focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
          @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        />
        <button
          type="button"
          class="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7 rounded border border-slate-700 bg-slate-800/80 text-slate-300 hover:text-white hover:border-cyan-400 hover:bg-slate-700/80 grid place-items-center"
          :aria-label="showPathHistory ? 'Hide path history' : 'Show path history'"
          :title="showPathHistory ? 'Hide history' : 'Show history'"
          @click="toggleHistory"
        >
          <ChevronDownIcon class="w-4 h-4" />
        </button>

        <HistoryDropdown
          :show="showPathHistory"
          :items="pathHistory"
          title="Recent Paths"
          :truncate="true"
          @select="handleHistorySelect"
          @clear="$emit('clear-history')"
          @close="showPathHistory = false"
        />
      </div>

      <div class="flex gap-1">
        <button
          @click="selectPath"
          class="h-9 w-9 grid place-items-center bg-slate-800 border border-slate-700 text-slate-200 hover:bg-slate-700 hover:text-white rounded-md transition-colors"
          title="Select File"
        >
          <DocumentIcon class="w-4 h-4" />
        </button>
        <button
          @click="selectFolder"
          class="h-9 w-9 grid place-items-center bg-slate-800 border border-slate-700 text-slate-200 hover:bg-slate-700 hover:text-white rounded-md transition-colors"
          title="Select Folder"
        >
          <FolderIcon class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>
