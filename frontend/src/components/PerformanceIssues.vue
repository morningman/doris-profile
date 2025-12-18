<template>
  <div class="performance-issues">
    <!-- Performance Issues List -->
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
        
        <!-- AI Diagnosis Button -->
        <div v-if="hotspot.suggestion_source === 'default'" class="ai-diagnosis-section">
          <button 
            class="ai-diagnosis-btn"
            :class="{ 'loading': diagnosingNodes[hotspot.node_id] }"
            :disabled="diagnosingNodes[hotspot.node_id]"
            @click.stop="diagnoseWithAI(hotspot)"
          >
            <i v-if="!diagnosingNodes[hotspot.node_id]" class="fas fa-brain"></i>
            <i v-else class="fas fa-spinner fa-spin"></i>
            {{ diagnosingNodes[hotspot.node_id] ? 'Diagnosing...' : 'AI Diagnosis' }}
          </button>
          <label class="language-checkbox" @click.stop>
            <input 
              type="checkbox" 
              v-model="useChinese[hotspot.node_id]"
              :disabled="diagnosingNodes[hotspot.node_id]"
            />
            <span>中文</span>
          </label>
        </div>
        
        <div v-if="hotspot.suggestion_source && hotspot.suggestion_source !== 'ai' && hotspot.suggestion_source !== 'default'" class="suggestion-status-warning">
          <i class="fas fa-exclamation-circle"></i>
          {{ hotspot.suggestion_source }}
        </div>
        <div v-if="hotspot.suggestion" class="hotspot-suggestion">
          <div class="suggestion-header-line">
            <i class="fas fa-lightbulb"></i>
            <span v-if="hotspot.suggestion_source === 'ai'" class="ai-badge">AI</span>
          </div>
          <div class="suggestion-content markdown-body" v-html="renderMarkdown(hotspot.suggestion)"></div>
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
import { ref } from 'vue';
import { marked } from 'marked';
import { diagnoseNode } from '@/utils/api';

// Configure marked
marked.setOptions({
  breaks: true,
  gfm: true,
});

export default {
  name: "PerformanceIssues",
  props: {
    hotspots: {
      type: Array,
      default: () => [],
    },
    suggestions: {
      type: Array,
      default: () => [],
    },
    profileText: {
      type: String,
      default: '',
    },
  },
  emits: ['node-click', 'hotspot-updated'],
  setup(props, { emit }) {
    const diagnosingNodes = ref({});
    const useChinese = ref({});
    
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

    const diagnoseWithAI = async (hotspot) => {
      if (!props.profileText) {
        console.error('No profile text available');
        return;
      }
      
      // Mark as diagnosing
      diagnosingNodes.value[hotspot.node_id] = true;
      
      try {
        // Get language preference (default to English)
        const language = useChinese.value[hotspot.node_id] ? 'zh' : 'en';
        const result = await diagnoseNode(props.profileText, hotspot.node_id, language);
        
        if (result.success) {
          // Update the hotspot with AI suggestion
          const updatedHotspot = {
            ...hotspot,
            suggestion: result.suggestion,
            suggestion_source: result.suggestion_source,
          };
          emit('hotspot-updated', updatedHotspot);
        } else {
          console.error('Diagnosis failed:', result.error);
        }
      } catch (error) {
        console.error('Failed to diagnose node:', error);
      } finally {
        diagnosingNodes.value[hotspot.node_id] = false;
      }
    };

    const renderMarkdown = (text) => {
      if (!text) return '';
      try {
        return marked.parse(text);
      } catch (error) {
        console.error('Failed to render markdown:', error);
        return text;
      }
    };

    return {
      priorityClass,
      categoryIcon,
      handleNodeClick,
      diagnoseWithAI,
      diagnosingNodes,
      useChinese,
      renderMarkdown,
    };
  },
};
</script>

<style lang="scss" scoped>
.performance-issues {
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

.ai-diagnosis-section {
  margin-top: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.ai-diagnosis-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 600;
  color: white;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
  
  &:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }
  
  &:active:not(:disabled) {
    transform: translateY(0);
  }
  
  &:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
  
  &.loading {
    background: linear-gradient(135deg, #9ca3af 0%, #6b7280 100%);
  }
  
  i {
    font-size: 14px;
  }
}

.language-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
  
  input[type="checkbox"] {
    cursor: pointer;
    width: 16px;
    height: 16px;
    accent-color: #667eea;
    
    &:disabled {
      cursor: not-allowed;
      opacity: 0.5;
    }
  }
  
  span {
    font-weight: 500;
  }
  
  &:hover:not(:has(input:disabled)) {
    color: var(--text-primary);
  }
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
  font-size: 13px;
  padding: 12px 16px;
  border-radius: 8px;
  margin-top: 12px;
  background: rgba(64, 158, 255, 0.05);
  border: 1px solid rgba(64, 158, 255, 0.2);
  position: relative;
  overflow: hidden;
  word-wrap: break-word;

  .suggestion-header-line {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    
    i {
      color: var(--primary-color);
      font-size: 16px;
    }
  }
  
  .ai-badge {
    display: inline-block;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    font-size: 9px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 3px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .suggestion-content {
    color: var(--text-primary);
    line-height: 1.6;
    overflow-x: auto;
    max-width: 100%;
  }
}

// Markdown styles
.markdown-body {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-primary);
  max-width: 100%;
  overflow-wrap: break-word;
  word-wrap: break-word;

  h1, h2, h3, h4, h5, h6 {
    margin-top: 16px;
    margin-bottom: 10px;
    font-weight: 600;
    line-height: 1.4;
    color: var(--text-primary);
    word-break: break-word;
  }

  h1 { font-size: 1.6em; border-bottom: 1px solid var(--border-light); padding-bottom: 8px; }
  h2 { font-size: 1.4em; border-bottom: 1px solid var(--border-light); padding-bottom: 6px; }
  h3 { font-size: 1.25em; }
  h4 { font-size: 1.1em; }

  p {
    margin-top: 0;
    margin-bottom: 12px;
    word-break: break-word;
  }

  ul, ol {
    margin-top: 0;
    margin-bottom: 12px;
    padding-left: 24px;

    li {
      margin-bottom: 6px;
      word-break: break-word;
    }
  }

  ul {
    list-style-type: disc;
  }

  ol {
    list-style-type: decimal;
  }

  strong {
    font-weight: 600;
    color: var(--primary-color);
    word-break: break-word;
  }

  em {
    font-style: italic;
  }

  code {
    padding: 2px 6px;
    background: rgba(0, 0, 0, 0.05);
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
    font-size: 0.9em;
    color: #e83e8c;
    word-break: break-all;
    white-space: pre-wrap;
  }

  pre {
    padding: 12px;
    background: rgba(0, 0, 0, 0.05);
    border-radius: 6px;
    overflow-x: auto;
    margin-bottom: 12px;
    max-width: 100%;
    white-space: pre-wrap;
    word-wrap: break-word;

    code {
      padding: 0;
      background: none;
      color: var(--text-primary);
      white-space: pre-wrap;
      word-break: break-word;
    }
  }

  blockquote {
    margin: 12px 0;
    padding: 8px 16px;
    border-left: 4px solid var(--primary-color);
    background: rgba(64, 158, 255, 0.05);
    color: var(--text-secondary);

    p {
      margin-bottom: 0;
    }
  }

  hr {
    border: none;
    border-top: 1px solid var(--border-light);
    margin: 16px 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 12px;

    th, td {
      padding: 8px 12px;
      border: 1px solid var(--border-light);
      text-align: left;
    }

    th {
      background: rgba(0, 0, 0, 0.03);
      font-weight: 600;
    }
  }

  a {
    color: var(--primary-color);
    text-decoration: none;

    &:hover {
      text-decoration: underline;
    }
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

