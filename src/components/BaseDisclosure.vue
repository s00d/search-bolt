<script setup lang="ts">
  import { ref } from 'vue';

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
    >
      <slot name="button"></slot>
    </button>

    <transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div v-show="isOpen" class="mt-4">
        <slot></slot>
      </div>
    </transition>
  </div>
</template>
