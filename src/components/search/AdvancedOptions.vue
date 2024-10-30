<script setup lang="ts">
import { ref } from 'vue';
import { AdjustmentsHorizontalIcon } from '@heroicons/vue/24/outline';
import BaseSwitch from '../BaseSwitch.vue';
import BaseDisclosure from '../BaseDisclosure.vue';

const props = defineProps<{
  caseSensitive: boolean;
  wholeWord: boolean;
  useRegex: boolean;
  maxDepth?: number;
  fileTypes: string[];
  excludePatterns: string[];
}>();

const emit = defineEmits<{
  'update:caseSensitive': [value: boolean];
  'update:wholeWord': [value: boolean];
  'update:useRegex': [value: boolean];
  'update:maxDepth': [value: number | undefined];
  'update:fileTypes': [value: string[]];
  'update:excludePatterns': [value: string[]];
}>();

const fileTypeInput = ref('');
const excludeInput = ref('');

function addFileType() {
  if (fileTypeInput.value && !props.fileTypes.includes(fileTypeInput.value)) {
    emit('update:fileTypes', [...props.fileTypes, fileTypeInput.value]);
    fileTypeInput.value = '';
  }
}

function removeFileType(type: string) {
  emit('update:fileTypes', props.fileTypes.filter(t => t !== type));
}

function addExcludePattern() {
  if (excludeInput.value && !props.excludePatterns.includes(excludeInput.value)) {
    emit('update:excludePatterns', [...props.excludePatterns, excludeInput.value]);
    excludeInput.value = '';
  }
}

function removeExcludePattern(pattern: string) {
  emit('update:excludePatterns', props.excludePatterns.filter(p => p !== pattern));
}
</script>

<template>
  <BaseDisclosure button-class="flex items-center gap-1 text-gray-600 hover:text-gray-900 text-sm">
    <template #button>
      <AdjustmentsHorizontalIcon class="w-4 h-4" />
      Advanced
    </template>

    <div class="space-y-3 pt-2 text-sm">
      <div class="flex flex-wrap gap-4">
        <BaseSwitch
          :model-value="caseSensitive"
          @update:model-value="$emit('update:caseSensitive', $event)"
          label="Case sensitive"
        />
        <BaseSwitch
          :model-value="wholeWord"
          @update:model-value="$emit('update:wholeWord', $event)"
          label="Whole word"
        />
        <BaseSwitch
          :model-value="useRegex"
          @update:model-value="$emit('update:useRegex', $event)"
          label="Use regex"
        />
      </div>

      <div class="flex items-center gap-2">
        <span class="text-gray-700 whitespace-nowrap">Max depth:</span>
        <input
          :value="maxDepth"
          type="number"
          min="1"
          class="w-20 h-7 px-2 border rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
          @input="$emit('update:maxDepth', ($event.target as HTMLInputElement).value ? Number(($event.target as HTMLInputElement).value) : undefined)"
        />
      </div>

      <div>
        <div class="flex gap-2 mb-1">
          <input
            v-model="fileTypeInput"
            type="text"
            placeholder="File types (*.js, *.py)"
            class="flex-1 h-7 px-2 border rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
            @keyup.enter="addFileType"
          />
          <button
            @click="addFileType"
            class="px-2 h-7 bg-gray-100 hover:bg-gray-200 rounded"
          >
            Add
          </button>
        </div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="type in fileTypes"
            :key="type"
            class="px-1.5 py-0.5 bg-gray-100 rounded flex items-center gap-1 text-xs"
          >
            {{ type }}
            <button
              @click="removeFileType(type)"
              class="text-gray-500 hover:text-gray-700"
            >
              ×
            </button>
          </span>
        </div>
      </div>

      <div>
        <div class="flex gap-2 mb-1">
          <input
            v-model="excludeInput"
            type="text"
            placeholder="Exclude patterns (node_modules, .git)"
            class="flex-1 h-7 px-2 border rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
            @keyup.enter="addExcludePattern"
          />
          <button
            @click="addExcludePattern"
            class="px-2 h-7 bg-gray-100 hover:bg-gray-200 rounded"
          >
            Add
          </button>
        </div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="pattern in excludePatterns"
            :key="pattern"
            class="px-1.5 py-0.5 bg-gray-100 rounded flex items-center gap-1 text-xs"
          >
            {{ pattern }}
            <button
              @click="removeExcludePattern(pattern)"
              class="text-gray-500 hover:text-gray-700"
            >
              ×
            </button>
          </span>
        </div>
      </div>
    </div>
  </BaseDisclosure>
</template>
