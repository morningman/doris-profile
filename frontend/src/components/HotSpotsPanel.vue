<template>
  <div class="hotspots-panel">
    <!-- Hotspots List -->
    <div v-if="hotspots.length === 0" class="no-issues">
      <i class="fas fa-check-circle"></i>
      <p>No performance issues detected</p>
    </div>

    <div v-else class="hotspots-list">
      <div
        v-for="(hotspot, index) in hotspots"
        :key="hotspot.node_id"
        class="hotspot-item"
        :class="`severity-${hotspot.severity.toLowerCase()}`"
        @click="handleNodeClick(hotspot)"
      >
        <div class="hotspot-header">
          <div class="hotspot-rank">{{ index + 1 }}</div>
          <div class="hotspot-info">
            <span class="operator-name">{{ hotspot.operator_name }}</span>
            <span class="badge" :class="`badge-${hotspot.severity.toLowerCase()}`">
              {{ hotspot.severity }}
            </span>
          </div>
          <div v-if="hotspot.time_percentage" class="time-pct">
            {{ hotspot.time_percentage.toFixed(1) }}%
          </div>
        </div>
        <div class="hotspot-path">{{ hotspot.node_path }}</div>
        <div class="hotspot-description">{{ hotspot.description }}</div>
        <div v-if="hotspot.suggestion_source && hotspot.suggestion_source !== 'ai'" class="suggestion-status-warning">
          <i class="fas fa-exclamation-circle"></i>
          {{ hotspot.suggestion_source }}
        </div>
        <div v-if="hotspot.suggestion" class="hotspot-suggestion">
          <i class="fas fa-lightbulb"></i>
          <span v-if="hotspot.suggestion_source === 'ai'" class="ai-badge">AI</span>
          {{ hotspot.suggestion }}
        </div>
      </div>
    </div>

    <!-- Suggestions Section -->
    <div v-if="suggestions.length > 0" class="suggestions-section">
      <h4><i class="fas fa-magic"></i> Optimization Suggestions</h4>
      <div class="suggestions-list">
        <div
          v-for="(suggestion, index) in suggestions"
          :key="index"
          class="suggestion-item"
        >
          <div class="suggestion-header">
            <span class="suggestion-title">{{ suggestion.title }}</span>
            <span class="badge" :class="`badge-${priorityClass(suggestion.priority)}`">
              {{ suggestion.priority }}
            </span>
          </div>
          <div class="suggestion-description">{{ suggestion.description }}</div>
          <div class="suggestion-category">
            <i :class="categoryIcon(suggestion.category)"></i>
            {{ suggestion.category }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "HotSpotsPanel",
  props: {
    hotspots: {
      type: Array,
      default: () => [],
    },
    suggestions: {
      type: Array,
      default: () => [],
    },
  },
  emits: ['node-click'],
  setup(props, { emit }) {
    const priorityClass = (priority) => {
      switch (priority) {
        case "Critical":
          return "critical";
        case "High":
          return "high";
        case "Medium":
          return "medium";
        default:
          return "low";
      }
    };

    const categoryIcon = (category) => {
      switch (category) {
        case "Query":
          return "fas fa-search";
        case "Schema":
          return "fas fa-database";
        case "Resource":
          return "fas fa-server";
        case "Configuration":
          return "fas fa-cog";
        default:
          return "fas fa-info-circle";
      }
    };

    const handleNodeClick = (hotspot) => {
      emit('node-click', hotspot.node_id);
    };

    return {
      priorityClass,
      categoryIcon,
      handleNodeClick,
    };
  },
};
</script>

<style lang="scss" scoped>
.hotspots-panel {
  height: 100%;
}

.no-issues {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: #67c23a;

  i {
    font-size: 48px;
    margin-bottom: 12px;
  }

  p {
    margin: 0;
    color: var(--text-secondary);
  }
}

.hotspots-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hotspot-item {
  padding: 16px;
  border-radius: 8px;
  background: var(--bg-secondary);
  border-left: 4px solid;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover {
    transform: translateX(4px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  &.severity-critical {
    border-left-color: var(--severity-critical);
    background: rgba(245, 108, 108, 0.05);
  }

  &.severity-high {
    border-left-color: var(--severity-high);
    background: rgba(230, 162, 60, 0.05);
  }

  &.severity-medium {
    border-left-color: var(--severity-medium);
    background: rgba(64, 158, 255, 0.05);
  }

  &.severity-low {
    border-left-color: var(--severity-low);
    background: rgba(103, 194, 58, 0.05);
  }
}

.hotspot-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.hotspot-rank {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--text-tertiary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.hotspot-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;

  .operator-name {
    font-weight: 600;
    color: var(--text-primary);
  }
}

.time-pct {
  font-weight: 600;
  color: var(--severity-high);
  font-size: 14px;
}

.hotspot-path {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
  font-family: monospace;
}

.hotspot-description {
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 8px;
}

.suggestion-status-warning {
  font-size: 11px;
  padding: 6px 10px;
  border-radius: 4px;
  margin-top: 8px;
  background: rgba(230, 162, 60, 0.15);
  color: #e6a23c;
  border-left: 3px solid #e6a23c;
  
  i {
    margin-right: 6px;
  }
}

.hotspot-suggestion {
  font-size: 12px;
  padding: 8px 12px;
  border-radius: 6px;
  margin-top: 8px;
  background: rgba(64, 158, 255, 0.1);
  color: var(--primary-color);
  position: relative;

  i {
    margin-right: 8px;
  }
  
  .ai-badge {
    display: inline-block;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    font-size: 9px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 3px;
    margin-right: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    vertical-align: middle;
  }
}

.suggestions-section {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border-light);

  h4 {
    margin: 0 0 16px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);

    i {
      margin-right: 8px;
      color: var(--warning-color);
    }
  }
}

.suggestions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.suggestion-item {
  padding: 14px;
  border-radius: 8px;
  background: var(--bg-secondary);
}

.suggestion-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;

  .suggestion-title {
    font-weight: 600;
    color: var(--text-primary);
  }
}

.suggestion-description {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 8px;
}

.suggestion-category {
  font-size: 11px;
  color: var(--text-tertiary);

  i {
    margin-right: 6px;
  }
}
</style>

