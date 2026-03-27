<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  label?: string;
}>();

defineEmits<{
  'update:modelValue': [value: boolean];
}>();
</script>

<template>
  <label class="inline-flex items-center gap-2 cursor-pointer select-none">
    <div class="relative">
      <input
        type="checkbox"
        class="sr-only"
        :checked="modelValue"
        @change="$emit('update:modelValue', !modelValue)"
      />
      <div
        class="block w-10 h-5 rounded-full border transition-colors duration-200"
        :class="modelValue ? 'bg-cyan-500 border-cyan-400' : 'bg-slate-700 border-slate-600'"
      />
      <div
        class="absolute left-0.5 top-0.5 bg-white w-4 h-4 rounded-full transition-transform duration-200"
        :class="modelValue ? 'translate-x-5' : 'translate-x-0'"
      />
    </div>
    <span v-if="label" class="text-xs text-slate-300">{{ label }}</span>
  </label>
</template>
