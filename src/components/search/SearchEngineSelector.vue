<script setup lang="ts">
import { ref } from 'vue';

export interface SearchEngine {
  id: string;
  name: string;
  description: string;
}

const engines: SearchEngine[] = [
  {
    id: 'ripgrep',
    name: 'ripgrep',
    description: 'Fast, modern search tool with regex support'
  },
  {
    id: 'grep',
    name: 'grep',
    description: 'Traditional Unix search utility'
  }
];

defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const isOpen = ref(false);
</script>

<template>
  <div class="relative">
    <button
      type="button"
      class="flex items-center gap-1.5 px-2 h-9 text-xs bg-gray-100 hover:bg-blue-600 rounded"
      @click="isOpen = !isOpen"
    >
      <span class="font-medium">{{ engines.find(e => e.id === modelValue)?.name || 'Select Engine' }}</span>
      <svg class="w-3 h-3" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
      </svg>
    </button>

    <div
      v-if="isOpen"
      class="absolute left-0 mt-1 w-48 bg-white border rounded-md shadow-lg z-10"
      v-click-outside="() => isOpen = false"
    >
      <div class="py-1">
        <button
          v-for="engine in engines"
          :key="engine.id"
          class="w-full px-3 py-2 text-left hover:bg-blue-600 hover:text-white flex flex-col"
          :class="{ 'bg-blue-50': engine.id === modelValue }"
          @click="emit('update:modelValue', engine.id); isOpen = false"
        >
          <span class="text-sm font-medium">{{ engine.name }}</span>
          <span class="text-xs text-gray-400">{{ engine.description }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
