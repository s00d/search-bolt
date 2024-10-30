<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { FolderIcon, DocumentIcon } from '@heroicons/vue/24/outline';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { UnlistenFn } from '@tauri-apps/api/event';
import HistoryDropdown from './HistoryDropdown.vue';

defineProps<{
  modelValue: string;
  pathHistory: string[];
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'save-history': [];
  'clear-history': [];
}>();

const isDragging = ref(false);
const showPathHistory = ref(false);
let dragDropUnlisten: UnlistenFn | null = null;

onMounted(async () => {
  dragDropUnlisten = await getCurrentWebview().onDragDropEvent((event) => {
    // @ts-ignore
    if (event.payload.type === 'hover') {
      isDragging.value = true;
    } else if (event.payload.type === 'drop') {
      const paths = event.payload.paths;
      if (paths.length > 0) {
        emit('update:modelValue', paths[0]);
        emit('save-history');
      }
      isDragging.value = false;
    } else {
      // cancelled
      isDragging.value = false;
    }
  });
});

onUnmounted(() => {
  if (dragDropUnlisten) {
    dragDropUnlisten();
  }
});

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
</script>

<template>
  <div class="relative">
    <div
      class="flex gap-1"
      :class="{ 'ring-1 ring-blue-500 ring-opacity-50': isDragging }"
    >
      <div class="relative flex-1">
        <input
          :value="modelValue"
          type="text"
          placeholder="Select file or folder to search..."
          class="w-full h-9 px-2 text-sm border rounded-md focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
          readonly
          @focus="showPathHistory = true"
        />

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
          class="p-2 bg-gray-100 hover:bg-gray-200 rounded-md"
          title="Select File"
        >
          <DocumentIcon class="w-4 h-4" />
        </button>
        <button
          @click="selectFolder"
          class="p-2 bg-gray-100 hover:bg-gray-200 rounded-md"
          title="Select Folder"
        >
          <FolderIcon class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div
      v-if="isDragging"
      class="absolute inset-0 bg-blue-50 bg-opacity-50 rounded-md flex items-center justify-center pointer-events-none"
    >
      <p class="text-blue-600 text-sm">Drop file or folder here</p>
    </div>
  </div>
</template>
