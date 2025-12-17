<template>
  <div class="profile-analyzer">
    <!-- Input Section -->
    <div v-if="!hasResult" class="input-section fade-in">
      <div class="card">
        <div class="card-header">
          <h2><i class="fas fa-upload"></i> Upload Profile</h2>
        </div>
        <div class="input-methods">
          <el-tabs v-model="activeTab" type="border-card">
            <el-tab-pane label="Upload File" name="file">
              <FileUploader @file-selected="handleFileUpload" />
            </el-tab-pane>
            <el-tab-pane label="Paste Text" name="text">
              <TextInput @submit="handleTextSubmit" />
            </el-tab-pane>
          </el-tabs>
        </div>
      </div>

      <!-- Loading State -->
      <div v-if="isLoading" class="loading-overlay">
        <div class="loading-content">
          <div class="loading-spinner"></div>
          <p>Analyzing profile...</p>
        </div>
      </div>

      <!-- Error State -->
      <el-alert
        v-if="error"
        :title="error"
        type="error"
        show-icon
        closable
        @close="clearError"
        class="error-alert"
      />
    </div>

    <!-- Result Section -->
    <div v-else class="result-section fade-in">
      <!-- Action Bar -->
      <div class="action-bar">
        <button class="btn btn-secondary" @click="handleReset">
          <i class="fas fa-arrow-left"></i> Analyze Another Profile
        </button>
      </div>

      <!-- Summary Section -->
      <AnalysisSummary
        :summary="summary"
        :score="performanceScore"
        :conclusion="conclusion"
      />

      <!-- Main Content Grid -->
      <div class="result-grid">
        <!-- Execution Tree -->
        <div class="card execution-tree-card">
          <div class="card-header">
            <h3><i class="fas fa-project-diagram"></i> Execution Plan</h3>
          </div>
          <DAGVisualization :tree="executionTree" />
        </div>

        <!-- Hotspots Panel -->
        <div class="card hotspots-card">
          <div class="card-header">
            <h3><i class="fas fa-fire"></i> Performance Hotspots</h3>
            <span class="badge" :class="hotspotBadgeClass">
              {{ hotspots.length }} issue(s)
            </span>
          </div>
          <HotSpotsPanel :hotspots="hotspots" :suggestions="suggestions" />
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { computed, ref } from "vue";
import { useStore } from "vuex";
import FileUploader from "@/components/FileUploader.vue";
import TextInput from "@/components/TextInput.vue";
import AnalysisSummary from "@/components/AnalysisSummary.vue";
import DAGVisualization from "@/components/DAGVisualization.vue";
import HotSpotsPanel from "@/components/HotSpotsPanel.vue";

export default {
  name: "ProfileAnalyzer",
  components: {
    FileUploader,
    TextInput,
    AnalysisSummary,
    DAGVisualization,
    HotSpotsPanel,
  },
  setup() {
    const store = useStore();
    const activeTab = ref("file");

    // Getters
    const hasResult = computed(() => store.getters.hasResult);
    const isLoading = computed(() => store.state.isLoading);
    const error = computed(() => store.state.error);
    const hotspots = computed(() => store.getters.hotspots);
    const suggestions = computed(() => store.getters.suggestions);
    const executionTree = computed(() => store.getters.executionTree);
    const summary = computed(() => store.getters.summary);
    const performanceScore = computed(() => store.getters.performanceScore);
    const conclusion = computed(() => store.getters.conclusion);

    const hotspotBadgeClass = computed(() => {
      const count = hotspots.value.length;
      if (count === 0) return "badge-low";
      const hasCritical = hotspots.value.some(
        (h) => h.severity === "Critical"
      );
      const hasHigh = hotspots.value.some((h) => h.severity === "High");
      if (hasCritical) return "badge-critical";
      if (hasHigh) return "badge-high";
      return "badge-medium";
    });

    // Actions
    const handleFileUpload = async (file) => {
      await store.dispatch("analyzeFile", file);
    };

    const handleTextSubmit = async (text) => {
      await store.dispatch("analyzeProfile", text);
    };

    const handleReset = () => {
      store.dispatch("clearResult");
    };

    const clearError = () => {
      store.commit("SET_ERROR", null);
    };

    return {
      activeTab,
      hasResult,
      isLoading,
      error,
      hotspots,
      suggestions,
      executionTree,
      summary,
      performanceScore,
      conclusion,
      hotspotBadgeClass,
      handleFileUpload,
      handleTextSubmit,
      handleReset,
      clearError,
    };
  },
};
</script>

<style lang="scss" scoped>
.profile-analyzer {
  width: 100%;
}

.input-section {
  max-width: 800px;
  margin: 40px auto;
}

.input-methods {
  :deep(.el-tabs__content) {
    padding: 20px;
  }
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;

  .loading-content {
    text-align: center;

    p {
      margin-top: 16px;
      color: var(--text-secondary);
    }
  }
}

.error-alert {
  margin-top: 16px;
}

.result-section {
  width: 100%;
}

.action-bar {
  margin-bottom: 20px;
}

.result-grid {
  display: grid;
  grid-template-columns: 1fr 400px;
  gap: 20px;
  margin-top: 20px;

  @media (max-width: 1200px) {
    grid-template-columns: 1fr;
  }
}

.execution-tree-card {
  min-height: 500px;
}

.hotspots-card {
  max-height: 700px;
  overflow-y: auto;
}
</style>

