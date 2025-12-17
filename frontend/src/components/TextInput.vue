<template>
  <div class="text-input">
    <div class="input-container">
      <el-input
        v-model="profileText"
        type="textarea"
        :rows="15"
        placeholder="Paste your Doris query profile here..."
        resize="vertical"
      />
    </div>
    <div class="input-actions">
      <span class="char-count">{{ charCount }} characters</span>
      <div class="buttons">
        <button
          class="btn btn-secondary"
          @click="clearText"
          :disabled="!profileText"
        >
          <i class="fas fa-times"></i> Clear
        </button>
        <button
          class="btn btn-primary"
          @click="submitText"
          :disabled="!profileText.trim()"
        >
          <i class="fas fa-search"></i> Analyze
        </button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed } from "vue";

export default {
  name: "TextInput",
  emits: ["submit"],
  setup(props, { emit }) {
    const profileText = ref("");

    const charCount = computed(() => {
      return profileText.value.length.toLocaleString();
    });

    const clearText = () => {
      profileText.value = "";
    };

    const submitText = () => {
      if (profileText.value.trim()) {
        emit("submit", profileText.value);
      }
    };

    return {
      profileText,
      charCount,
      clearText,
      submitText,
    };
  },
};
</script>

<style lang="scss" scoped>
.text-input {
  width: 100%;
}

.input-container {
  :deep(.el-textarea__inner) {
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 13px;
    line-height: 1.6;
    padding: 16px;
    border-radius: 8px;
  }
}

.input-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
}

.char-count {
  font-size: 12px;
  color: var(--text-tertiary);
}

.buttons {
  display: flex;
  gap: 12px;
}
</style>

