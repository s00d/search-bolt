<script setup lang="ts">
import { ref } from 'vue';
import { FolderIcon, DocumentIcon } from '@heroicons/vue/24/outline';
import { open } from '@tauri-apps/plugin-dialog';
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

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  isDragging.value = true;
}

function handleDragLeave() {
  isDragging.value = false;
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragging.value = false;

  const items = event.dataTransfer?.items;
  if (!items) return;

  const item = items[0];
  if (item.kind === 'file') {
    const file = item.getAsFile();
    if (file) {
      // @ts-ignore
      emit('update:modelValue', file.path);
      emit('save-history');
    }
  }
}

function handleHistorySelect(path: string) {
  emit('update:modelValue', path);
  showPathHistory.value = false;
}
</script>

<template>
  <div
    class="relative"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
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
