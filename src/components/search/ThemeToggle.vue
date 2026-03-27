<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import BaseSwitch from '../BaseSwitch.vue';

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
  <BaseSwitch
    :model-value="isDark"
    label="Dark mode"
    @update:model-value="isDark = $event"
  />
</template>
