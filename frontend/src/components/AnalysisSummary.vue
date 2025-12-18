<template>
  <div class="analysis-summary card">
    <div class="summary-grid">
      <!-- Performance Score -->
      <div class="score-section">
        <div class="score-circle" :class="scoreClass">
          <span class="score-value">{{ score }}</span>
          <span class="score-label">Score</span>
        </div>
        <div class="score-category" :class="scoreClass">
          {{ scoreCategory }}
        </div>
      </div>

      <!-- Query Info -->
      <div class="info-section">
        <h3><i class="fas fa-info-circle"></i> Query Summary</h3>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Query ID</span>
            <span class="info-value">{{ summary?.query_id || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Total Time</span>
            <span class="info-value">{{ summary?.total_time || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Status</span>
            <span class="info-value status" :class="statusClass">
              {{ summary?.query_state || "N/A" }}
            </span>
          </div>
          <div class="info-item">
            <span class="info-label">Doris Version</span>
            <span class="info-value">{{
              summary?.doris_version || "N/A"
            }}</span>
          </div>
        </div>
      </div>

      <!-- Conclusion -->
      <div class="conclusion-section">
        <h3><i class="fas fa-clipboard-check"></i> Analysis Conclusion</h3>
        <p class="conclusion-text">{{ conclusion }}</p>
      </div>
    </div>

    <!-- SQL Statement -->
    <div v-if="summary?.sql_statement" class="sql-section">
      <div class="sql-header">
        <h4><i class="fas fa-code"></i> SQL Statement</h4>
        <button @click="toggleSqlFormat" class="format-btn">
          <i :class="isFormatted ? 'fas fa-compress-alt' : 'fas fa-expand-alt'"></i>
          {{ isFormatted ? 'Compact' : 'Format' }}
        </button>
      </div>
      <div class="sql-container">
        <pre v-if="!isFormatted" class="sql-code compact"><code class="sql" v-html="compactHighlightedSql"></code></pre>
        <pre v-else class="sql-code formatted"><code class="sql" v-html="highlightedSql"></code></pre>
      </div>
    </div>
  </div>
</template>

<script>
import { computed, ref } from "vue";
import { format } from "sql-formatter";
import hljs from "highlight.js/lib/core";
import sql from "highlight.js/lib/languages/sql";
import "highlight.js/styles/atom-one-dark.css";

// Register SQL language
hljs.registerLanguage("sql", sql);

export default {
  name: "AnalysisSummary",
  props: {
    summary: {
      type: Object,
      default: null,
    },
    score: {
      type: Number,
      default: 0,
    },
    conclusion: {
      type: String,
      default: "",
    },
  },
  setup(props) {
    const isFormatted = ref(true); // Default to formatted (multi-line)
    
    const scoreClass = computed(() => {
      if (props.score >= 90) return "score-excellent";
      if (props.score >= 70) return "score-good";
      if (props.score >= 50) return "score-fair";
      if (props.score >= 30) return "score-poor";
      return "score-critical";
    });

    const scoreCategory = computed(() => {
      if (props.score >= 90) return "Excellent";
      if (props.score >= 70) return "Good";
      if (props.score >= 50) return "Fair";
      if (props.score >= 30) return "Poor";
      return "Critical";
    });

    const statusClass = computed(() => {
      const state = props.summary?.query_state?.toUpperCase() || "";
      if (state === "FINISHED" || state === "OK") return "status-success";
      if (state === "ERROR" || state === "FAILED") return "status-error";
      return "status-default";
    });
    
    const displayedSql = computed(() => {
      if (!props.summary?.sql_statement) return "";
      
      if (isFormatted.value) {
        // This won't be used for formatted view, but kept for consistency
        return props.summary.sql_statement;
      } else {
        // Compact to single line
        return props.summary.sql_statement
          .split('\n')
          .map(line => line.trim())
          .filter(line => line.length > 0)
          .join(' ')
          .replace(/\s+/g, ' '); // Replace multiple spaces with single space
      }
    });
    
    const highlightedSql = computed(() => {
      if (!props.summary?.sql_statement) return "";
      
      try {
        // Format SQL with proper indentation
        const formattedSql = format(props.summary.sql_statement, {
          language: "sql",
          tabWidth: 2,
          keywordCase: "upper",
          linesBetweenQueries: 2,
        });
        
        // Apply syntax highlighting
        const highlighted = hljs.highlight(formattedSql, { language: "sql" });
        return highlighted.value;
      } catch (error) {
        console.error("SQL formatting error:", error);
        // Fallback to original SQL with highlighting
        try {
          const highlighted = hljs.highlight(props.summary.sql_statement, {
            language: "sql",
          });
          return highlighted.value;
        } catch (hlError) {
          // Ultimate fallback: return as-is
          return props.summary.sql_statement;
        }
      }
    });
    
    const compactHighlightedSql = computed(() => {
      if (!props.summary?.sql_statement) return "";
      
      try {
        // Compact SQL to single line
        const compactSql = props.summary.sql_statement
          .split('\n')
          .map(line => line.trim())
          .filter(line => line.length > 0)
          .join(' ')
          .replace(/\s+/g, ' '); // Replace multiple spaces with single space
        
        // Apply syntax highlighting to compact SQL
        const highlighted = hljs.highlight(compactSql, { language: "sql" });
        return highlighted.value;
      } catch (error) {
        console.error("SQL highlighting error:", error);
        // Fallback to plain compact SQL
        return props.summary.sql_statement
          .split('\n')
          .map(line => line.trim())
          .filter(line => line.length > 0)
          .join(' ')
          .replace(/\s+/g, ' ');
      }
    });
    
    const toggleSqlFormat = () => {
      isFormatted.value = !isFormatted.value;
    };

    return {
      scoreClass,
      scoreCategory,
      statusClass,
      isFormatted,
      displayedSql,
      highlightedSql,
      compactHighlightedSql,
      toggleSqlFormat,
    };
  },
};
</script>

<style lang="scss" scoped>
.analysis-summary {
  padding: 24px;
}

.summary-grid {
  display: grid;
  grid-template-columns: 150px 1fr 1fr;
  gap: 24px;

  @media (max-width: 900px) {
    grid-template-columns: 1fr;
  }
}

.score-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.score-circle {
  width: 100px;
  height: 100px;
  border-radius: 50%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border: 4px solid currentColor;

  .score-value {
    font-size: 32px;
    font-weight: 700;
    line-height: 1;
  }

  .score-label {
    font-size: 12px;
    color: var(--text-tertiary);
    margin-top: 4px;
  }
}

.score-category {
  margin-top: 8px;
  font-weight: 600;
  font-size: 14px;
}

.score-excellent {
  color: #67c23a;
}
.score-good {
  color: #85ce61;
}
.score-fair {
  color: #e6a23c;
}
.score-poor {
  color: #f56c6c;
}
.score-critical {
  color: #c45656;
}

.info-section,
.conclusion-section {
  h3 {
    margin: 0 0 16px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);

    i {
      margin-right: 8px;
    }
  }
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .info-label {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .info-value {
    font-weight: 500;
    color: var(--text-primary);
    word-break: break-all;

    &.status-success {
      color: #67c23a;
    }
    &.status-error {
      color: #f56c6c;
    }
  }
}

.conclusion-text {
  margin: 0;
  color: var(--text-secondary);
  line-height: 1.6;
}

.sql-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border-light);
}

.sql-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;

  h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);

    i {
      margin-right: 8px;
    }
  }
}

.format-btn {
  padding: 6px 12px;
  font-size: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-light);
  border-radius: 4px;
  cursor: pointer;
  color: var(--text-primary);
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;

  i {
    font-size: 11px;
  }

  &:hover {
    background: var(--bg-hover);
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  &:active {
    transform: scale(0.98);
  }
}

.sql-container {
  background: #282c34;
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
}

.sql-code {
  margin: 0;
  font-family: "Consolas", "Monaco", "Courier New", monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #abb2bf;
  
  code {
    background: transparent;
    padding: 0;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    
    &.sql {
      display: block;
    }
  }
  
  &.formatted {
    white-space: pre;
    word-break: normal;
    overflow-x: auto;
  }
  
  &.compact {
    white-space: pre-wrap;
    word-wrap: break-word;
    word-break: break-word;
    overflow-x: hidden;
  }
}
</style>

