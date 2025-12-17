<template>
  <div
    class="file-uploader"
    :class="{ 'is-dragging': isDragging }"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <div class="upload-area">
      <i class="fas fa-cloud-upload-alt upload-icon"></i>
      <h3>Drag & Drop Profile File</h3>
      <p class="upload-hint">or click to browse</p>
      <input
        type="file"
        ref="fileInput"
        class="file-input"
        accept=".txt,.log,.profile"
        @change="handleFileChange"
      />
      <button class="btn btn-primary" @click="triggerFileInput">
        <i class="fas fa-folder-open"></i> Select File
      </button>
      <p class="file-types">Supported: .txt, .log, .profile (Max 50MB)</p>
    </div>

    <!-- Selected File Info -->
    <div v-if="selectedFile" class="selected-file">
      <div class="file-info">
        <i class="fas fa-file-alt"></i>
        <div class="file-details">
          <span class="file-name">{{ selectedFile.name }}</span>
          <span class="file-size">{{ formatFileSize(selectedFile.size) }}</span>
        </div>
      </div>
      <button class="btn btn-primary" @click="uploadFile">
        <i class="fas fa-upload"></i> Analyze
      </button>
    </div>
  </div>
</template>

<script>
import { ref } from "vue";

export default {
  name: "FileUploader",
  emits: ["file-selected"],
  setup(props, { emit }) {
    const fileInput = ref(null);
    const selectedFile = ref(null);
    const isDragging = ref(false);

    const triggerFileInput = () => {
      fileInput.value?.click();
    };

    const handleFileChange = (event) => {
      const files = event.target.files;
      if (files && files.length > 0) {
        selectedFile.value = files[0];
      }
    };

    const handleDragOver = () => {
      isDragging.value = true;
    };

    const handleDragLeave = () => {
      isDragging.value = false;
    };

    const handleDrop = (event) => {
      isDragging.value = false;
      const files = event.dataTransfer?.files;
      if (files && files.length > 0) {
        selectedFile.value = files[0];
      }
    };

    const uploadFile = () => {
      if (selectedFile.value) {
        emit("file-selected", selectedFile.value);
      }
    };

    const formatFileSize = (bytes) => {
      if (bytes === 0) return "0 Bytes";
      const k = 1024;
      const sizes = ["Bytes", "KB", "MB", "GB"];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    };

    return {
      fileInput,
      selectedFile,
      isDragging,
      triggerFileInput,
      handleFileChange,
      handleDragOver,
      handleDragLeave,
      handleDrop,
      uploadFile,
      formatFileSize,
    };
  },
};
</script>

<style lang="scss" scoped>
.file-uploader {
  width: 100%;
}

.upload-area {
  border: 2px dashed var(--border-color);
  border-radius: 12px;
  padding: 40px;
  text-align: center;
  transition: all 0.3s;
  cursor: pointer;
  position: relative;

  &:hover {
    border-color: var(--primary-color);
    background: rgba(64, 158, 255, 0.02);
  }

  .is-dragging & {
    border-color: var(--primary-color);
    background: rgba(64, 158, 255, 0.05);
  }
}

.upload-icon {
  font-size: 48px;
  color: var(--primary-color);
  margin-bottom: 16px;
}

h3 {
  margin: 0 0 8px;
  font-weight: 600;
  color: var(--text-primary);
}

.upload-hint {
  color: var(--text-tertiary);
  margin-bottom: 20px;
}

.file-input {
  position: absolute;
  width: 100%;
  height: 100%;
  top: 0;
  left: 0;
  opacity: 0;
  cursor: pointer;
}

.file-types {
  margin-top: 16px;
  font-size: 12px;
  color: var(--text-tertiary);
}

.selected-file {
  margin-top: 20px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;

  .file-info {
    display: flex;
    align-items: center;
    gap: 12px;

    i {
      font-size: 24px;
      color: var(--primary-color);
    }
  }

  .file-details {
    display: flex;
    flex-direction: column;
  }

  .file-name {
    font-weight: 500;
    color: var(--text-primary);
  }

  .file-size {
    font-size: 12px;
    color: var(--text-tertiary);
  }
}
</style>

