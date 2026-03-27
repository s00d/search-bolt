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
    class="absolute z-20 w-full mt-1 bg-slate-900 border border-slate-700 rounded-md shadow-xl text-xs"
    v-click-outside="() => emit('close')"
  >
    <div class="p-2 flex justify-between items-center border-b border-slate-700">
      <span class="text-slate-300 flex items-center gap-1">
        <ClockIcon class="w-3.5 h-3.5" />
        {{ title || 'Recent' }}
      </span>
      <button
        @click="emit('clear')"
        class="text-slate-400 hover:text-red-400"
      >
        Clear
      </button>
    </div>
    <div class="max-h-48 overflow-y-auto">
      <button
        v-for="item in items"
        :key="item"
        @click="emit('select', item)"
        class="w-full px-2 py-1.5 text-left text-slate-200 hover:bg-slate-800"
        :class="{ 'truncate': truncate }"
        :title="truncate ? item : undefined"
      >
        {{ truncate ? truncateText(item) : item }}
      </button>
    </div>
  </div>
</template>
