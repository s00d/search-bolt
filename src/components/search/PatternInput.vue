<script setup lang="ts">
import { ref, watch } from 'vue';
import { ChevronDownIcon } from '@heroicons/vue/24/outline';
import HistoryDropdown from './HistoryDropdown.vue';

const props = defineProps<{
  modelValue: string;
  patternHistory: string[];
  focusSignal?: number;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'save-history': [];
  'clear-history': [];
  'search': [];
}>();

const showPatternHistory = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

function handleHistorySelect(pattern: string) {
  emit('update:modelValue', pattern);
  showPatternHistory.value = false;
}

function toggleHistory() {
  showPatternHistory.value = !showPatternHistory.value;
  if (showPatternHistory.value && inputRef.value) {
    inputRef.value.focus();
  }
}

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
</script>

<template>
  <div class="relative flex-1">
    <input
      ref="inputRef"
      :value="modelValue"
      type="text"
      placeholder="Search pattern..."
      class="w-full h-9 pl-3 pr-9 text-sm border border-slate-700 bg-slate-900 text-slate-100 rounded-md focus:outline-none focus:ring-2 focus:ring-cyan-500/50 focus:border-cyan-500"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keyup.enter="$emit('search')"
    />
    <button
      type="button"
      class="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7 rounded border border-slate-700 bg-slate-800/80 text-slate-300 hover:text-white hover:border-cyan-400 hover:bg-slate-700/80 grid place-items-center"
      :aria-label="showPatternHistory ? 'Hide search history' : 'Show search history'"
      :title="showPatternHistory ? 'Hide history' : 'Show history'"
      @click="toggleHistory"
    >
      <ChevronDownIcon class="w-4 h-4" />
    </button>

    <HistoryDropdown
      :show="showPatternHistory"
      :items="patternHistory"
      title="Recent Patterns"
      @select="handleHistorySelect"
      @clear="$emit('clear-history')"
      @close="showPatternHistory = false"
    />
  </div>
</template>
