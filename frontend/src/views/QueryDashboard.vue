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
        <button class="btn btn-outline" @click="toggleDebug">
          <i class="fas fa-bug"></i> {{ showDebug ? 'Hide' : 'Show' }} Debug JSON
        </button>
      </div>

      <!-- Debug JSON Panel (显示在 Summary 之前) -->
      <div v-if="showDebug" class="card debug-card">
        <div class="card-header">
          <h3><i class="fas fa-code"></i> Debug: Execution Tree JSON</h3>
          <div class="debug-actions">
            <button class="btn btn-sm" @click="copyJson">
              <i class="fas fa-copy"></i> Copy
            </button>
            <span v-if="copySuccess" class="copy-success">Copied!</span>
          </div>
        </div>
        <div class="debug-content">
          <div class="debug-stats">
            <span class="stat">
              <strong>Total Nodes:</strong> {{ executionTree?.nodes?.length || 0 }}
            </span>
            <span class="stat">
              <strong>Root:</strong> {{ executionTree?.root?.operator_name || 'N/A' }}
            </span>
            <span class="stat">
              <strong>Fragments:</strong> {{ fragmentCount }}
            </span>
          </div>
          <div class="json-tabs">
            <button 
              :class="['tab-btn', { active: debugTab === 'tree' }]"
              @click="debugTab = 'tree'"
            >
              Execution Tree
            </button>
            <button 
              :class="['tab-btn', { active: debugTab === 'nodes' }]"
              @click="debugTab = 'nodes'"
            >
              All Nodes ({{ executionTree?.nodes?.length || 0 }})
            </button>
            <button 
              :class="['tab-btn', { active: debugTab === 'summary' }]"
              @click="debugTab = 'summary'"
            >
              Summary
            </button>
            <button 
              :class="['tab-btn', { active: debugTab === 'hotspots' }]"
              @click="debugTab = 'hotspots'"
            >
              Hotspots ({{ hotspots?.length || 0 }})
            </button>
          </div>
          <pre class="json-content">{{ formattedDebugJson }}</pre>
        </div>
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
            <span class="node-count">{{ executionTree?.nodes?.length || 0 }} nodes</span>
          </div>
          <ExecutionGraph ref="executionGraph" :tree="executionTree" />
        </div>

        <!-- Performance Issues Panel -->
        <div class="card hotspots-card">
          <div class="card-header">
            <h3><i class="fas fa-fire"></i> Performance Issues</h3>
            <span class="badge" :class="hotspotBadgeClass">
              {{ hotspots.length }} issue(s)
            </span>
          </div>
          <PerformanceIssues 
            :hotspots="hotspots" 
            :suggestions="suggestions" 
            :profile-text="profileText"
            @node-click="handleHotspotNodeClick"
            @hotspot-updated="handleHotspotUpdated" 
          />
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
import ExecutionGraph from "@/components/ExecutionGraph.vue";
import PerformanceIssues from "@/components/PerformanceIssues.vue";

export default {
  name: "QueryDashboard",
  components: {
    FileUploader,
    TextInput,
    AnalysisSummary,
    ExecutionGraph,
    PerformanceIssues,
  },
  setup() {
    const store = useStore();
    const activeTab = ref("file");
    const showDebug = ref(false);
    const debugTab = ref("tree");
    const copySuccess = ref(false);
    const executionGraph = ref(null);

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
    const profileText = computed(() => store.getters.profileText);

    const fragmentCount = computed(() => {
      if (!executionTree.value?.nodes) return 0;
      const fragments = new Set();
      executionTree.value.nodes.forEach(n => {
        if (n.fragment_id) fragments.add(n.fragment_id);
      });
      return fragments.size;
    });

    const formattedDebugJson = computed(() => {
      let data = null;
      switch (debugTab.value) {
        case 'tree':
          data = executionTree.value;
          break;
        case 'nodes':
          data = executionTree.value?.nodes || [];
          break;
        case 'summary':
          data = summary.value;
          break;
        case 'hotspots':
          data = hotspots.value;
          break;
        default:
          data = executionTree.value;
      }
      return JSON.stringify(data, null, 2);
    });

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

    const toggleDebug = () => {
      showDebug.value = !showDebug.value;
    };

    const copyJson = async () => {
      try {
        await navigator.clipboard.writeText(formattedDebugJson.value);
        copySuccess.value = true;
        setTimeout(() => {
          copySuccess.value = false;
        }, 2000);
      } catch (err) {
        console.error('Failed to copy:', err);
      }
    };

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

    const handleHotspotNodeClick = (nodeId) => {
      if (executionGraph.value) {
        executionGraph.value.locateAndCenterNode(nodeId);
      }
    };

    const handleHotspotUpdated = (updatedHotspot) => {
      store.commit("UPDATE_HOTSPOT", updatedHotspot);
    };

    return {
      activeTab,
      showDebug,
      debugTab,
      copySuccess,
      executionGraph,
      profileText,
      hasResult,
      isLoading,
      error,
      hotspots,
      suggestions,
      executionTree,
      summary,
      performanceScore,
      conclusion,
      fragmentCount,
      formattedDebugJson,
      hotspotBadgeClass,
      toggleDebug,
      copyJson,
      handleFileUpload,
      handleTextSubmit,
      handleReset,
      clearError,
      handleHotspotNodeClick,
      handleHotspotUpdated,
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
  padding: 0 20px;
}

// 为所有主要内容块设置统一的最大宽度和居中
.action-bar,
.analysis-summary,
.debug-card,
.result-grid {
  max-width: 1800px;
  margin-left: auto;
  margin-right: auto;
  width: 100%;
}

.action-bar {
  margin-bottom: 20px;
  display: flex;
  gap: 12px;
  align-items: center;
}

.btn-outline {
  background: transparent;
  border: 1px solid #dcdfe6;
  color: #606266;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 6px;

  &:hover {
    border-color: #409eff;
    color: #409eff;
  }

  i {
    font-size: 12px;
  }
}

.btn-sm {
  padding: 4px 12px;
  font-size: 12px;
  border-radius: 4px;
  border: 1px solid #dcdfe6;
  background: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;

  &:hover {
    border-color: #409eff;
    color: #409eff;
  }
}

.debug-card {
  margin-bottom: 20px;
  
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .debug-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .copy-success {
    color: #67c23a;
    font-size: 12px;
  }
}

.debug-content {
  padding: 16px;
}

.debug-stats {
  display: flex;
  gap: 20px;
  margin-bottom: 16px;
  padding: 12px;
  background: #f0f9ff;
  border-radius: 6px;

  .stat {
    font-size: 13px;
    color: #606266;

    strong {
      color: #303133;
    }
  }
}

.json-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
  padding-bottom: 12px;

  .tab-btn {
    padding: 6px 14px;
    border: 1px solid #dcdfe6;
    border-radius: 4px;
    background: #fff;
    color: #606266;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;

    &:hover {
      border-color: #409eff;
      color: #409eff;
    }

    &.active {
      background: #409eff;
      border-color: #409eff;
      color: #fff;
    }
  }
}

.json-content {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 8px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  line-height: 1.5;
  max-height: 500px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.result-grid {
  display: grid;
  grid-template-columns: 1fr 450px; // Execution Plan 占更多空间，Hotspots 450px
  gap: 20px;
  margin-top: 20px;
  height: calc(100vh - 220px); // 优化高度计算（header 64px + padding 24px*2 + action-bar ~50px + margins ~60px）
  min-height: 700px;

  @media (max-width: 1400px) {
    grid-template-columns: 1fr; // 小屏幕时垂直布局
    height: auto;
  }
}

.execution-tree-card {
  display: flex;
  flex-direction: column;
  min-height: 0; // 允许 flex item 缩小

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .node-count {
    font-size: 12px;
    color: #909399;
    background: #f4f4f5;
    padding: 2px 8px;
    border-radius: 10px;
  }

  // ExecutionGraph 组件占据剩余空间
  :deep(.execution-graph) {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }
}

.hotspots-card {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;

  @media (max-width: 1400px) {
    max-height: 600px;
  }
}
</style>
