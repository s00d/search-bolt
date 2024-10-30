<script setup lang="ts">
import { ref } from 'vue';
import HistoryDropdown from './HistoryDropdown.vue';

defineProps<{
  modelValue: string;
  patternHistory: string[];
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'save-history': [];
  'clear-history': [];
  'search': [];
}>();

const showPatternHistory = ref(false);

function handleHistorySelect(pattern: string) {
  emit('update:modelValue', pattern);
  showPatternHistory.value = false;
}
</script>

<template>
  <div class="relative flex-1">
    <input
      :value="modelValue"
      type="text"
      placeholder="Search pattern..."
      class="w-full h-9 px-2 text-sm border rounded-md focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keyup.enter="$emit('search')"
      @focus="showPatternHistory = true"
    />

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
