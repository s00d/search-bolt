<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';

const isDark = ref(false);

onMounted(() => {
  // Check system preference and localStorage on mount
  const savedTheme = localStorage.getItem('theme');
  isDark.value = savedTheme === 'dark' ||
    (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches);
  updateTheme();
});

function updateTheme() {
  // Update document class and localStorage
  document.documentElement.classList.toggle('dark', isDark.value);
  localStorage.setItem('theme', isDark.value ? 'dark' : 'light');
}

watch(isDark, updateTheme);
</script>

<template>
  <div class="flex items-center space-x-2">
    <input
      type="checkbox"
      id="theme-toggle"
      v-model="isDark"
      class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500 dark:border-gray-600 dark:bg-gray-700"
    >
    <label for="theme-toggle" class="text-sm text-gray-700 dark:text-gray-300">
      Dark Mode
    </label>
  </div>
</template>
