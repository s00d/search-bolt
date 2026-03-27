<script setup lang="ts">
import { ref } from 'vue';
import { ChevronDownIcon } from '@heroicons/vue/24/outline';

defineProps<{
  buttonClass?: string;
}>();

const isOpen = ref(false);

function toggle() {
  isOpen.value = !isOpen.value;
}
</script>

<template>
  <div>
    <button
      @click="toggle"
      :class="buttonClass"
      class="w-full"
    >
      <div class="flex items-center justify-between w-full">
        <slot name="button" />
        <ChevronDownIcon class="w-4 h-4 transition-transform" :class="{ 'rotate-180': isOpen }" />
      </div>
    </button>

    <transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="transform opacity-0 -translate-y-1"
      enter-to-class="transform opacity-100 translate-y-0"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="transform opacity-100 translate-y-0"
      leave-to-class="transform opacity-0 -translate-y-1"
    >
      <div v-show="isOpen" class="mt-2">
        <slot />
      </div>
    </transition>
  </div>
</template>
