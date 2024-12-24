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
  <label class="inline-flex items-center cursor-pointer">
    <div class="relative">
      <input
        type="checkbox"
        class="sr-only"
        :checked="modelValue"
        @change="$emit('update:modelValue', !modelValue)"
      />
      <div
        class="disabled-switch block w-11 h-6 rounded-full transition-colors duration-200"
        :class="modelValue ? 'bg-blue-600' : 'bg-gray-200'"
      ></div>
      <div
        class="absolute left-1 top-1 bg-white w-4 h-4 rounded-full transition-transform duration-200"
        :class="modelValue ? 'translate-x-5' : 'translate-x-0'"
      ></div>
    </div>
    <span v-if="label" class="ml-3 text-gray-700">{{ label }}</span>
  </label>
</template>
