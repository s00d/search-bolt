<script setup lang="ts">
import { ClockIcon } from '@heroicons/vue/24/outline';

defineProps<{
  show: boolean;
  items: string[];
  title?: string;
  truncate?: boolean;
}>();

const emit = defineEmits<{
  'select': [value: string];
  'clear': [];
  'close': [];
}>();

function truncateText(text: string): string {
  const maxLength = 50;
  return text.length > maxLength
    ? '...' + text.slice(-(maxLength - 3))
    : text;
}
</script>

<template>
  <div
    v-if="show && items.length > 0"
    class="absolute z-10 w-full mt-1 bg-white border rounded-md shadow-lg text-sm"
    v-click-outside="() => emit('close')"
  >
    <div class="p-1.5 flex justify-between items-center border-b">
      <span class="text-gray-600 flex items-center gap-1">
        <ClockIcon class="w-3.5 h-3.5" />
        {{ title || 'Recent' }}
      </span>
      <button
        @click="emit('clear')"
        class="text-red-500 hover:text-red-600 text-xs"
      >
        Clear
      </button>
    </div>
    <div class="max-h-48 overflow-y-auto">
      <button
        v-for="item in items"
        :key="item"
        @click="emit('select', item)"
        class="w-full px-2 py-1.5 text-left hover:bg-gray-50"
        :class="{ 'truncate': truncate }"
        :title="truncate ? item : undefined"
      >
        {{ truncate ? truncateText(item) : item }}
      </button>
    </div>
  </div>
</template>
