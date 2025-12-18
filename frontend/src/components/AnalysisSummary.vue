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
        
        <!-- 第一行：Query ID 和 Status -->
        <div class="info-row">
          <div class="info-item info-item-wide">
            <span class="info-label">Query ID</span>
            <span class="info-value query-id">{{ summary?.query_id || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Status</span>
            <span class="info-value status" :class="statusClass">
              {{ summary?.query_state || "N/A" }}
            </span>
          </div>
        </div>
        
        <!-- 第二行：Total Time, Start Time, End Time -->
        <div class="info-row">
          <div class="info-item">
            <span class="info-label">Total Time</span>
            <span class="info-value total-time-highlight">{{ summary?.total_time || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Start Time</span>
            <span class="info-value">{{ summary?.start_time || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">End Time</span>
            <span class="info-value">{{ summary?.end_time || "N/A" }}</span>
          </div>
        </div>
        
        <!-- 其他行：按 Profile 顺序 -->
        <div class="info-row">
          <div class="info-item">
            <span class="info-label">Task Type</span>
            <span class="info-value">{{ summary?.query_type || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">User</span>
            <span class="info-value">{{ summary?.user || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Default Catalog</span>
            <span class="info-value">{{ summary?.default_catalog || "N/A" }}</span>
          </div>
        </div>
        
        <div class="info-row">
          <div class="info-item">
            <span class="info-label">Default Database</span>
            <span class="info-value">{{ summary?.default_db || "N/A" }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Doris Version</span>
            <span class="info-value">{{ summary?.doris_version || "N/A" }}</span>
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

    <!-- Execution Summary -->
    <div v-if="hasExecutionSummary" class="exec-summary-section">
      <h4><i class="fas fa-chart-line"></i> Execution Summary</h4>
      <div class="exec-summary-grid">
        <div v-if="summary?.execution_summary['Workload Group']" class="exec-item">
          <span class="exec-label">Workload Group</span>
          <span class="exec-value">{{ summary.execution_summary['Workload Group'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Parse SQL Time']" class="exec-item">
          <span class="exec-label">Parse SQL Time</span>
          <span class="exec-value">{{ summary.execution_summary['Parse SQL Time'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Plan Time']" class="exec-item">
          <span class="exec-label">Plan Time</span>
          <span class="exec-value">{{ summary.execution_summary['Plan Time'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Schedule Time']" class="exec-item">
          <span class="exec-label">Schedule Time</span>
          <span class="exec-value">{{ summary.execution_summary['Schedule Time'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Wait and Fetch Result Time']" class="exec-item">
          <span class="exec-label">Wait and Fetch Result Time</span>
          <span class="exec-value">{{ summary.execution_summary['Wait and Fetch Result Time'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Fetch Result Time']" class="exec-item">
          <span class="exec-label">Fetch Result Time</span>
          <span class="exec-value">{{ summary.execution_summary['Fetch Result Time'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Write Result Time']" class="exec-item">
          <span class="exec-label">Write Result Time</span>
          <span class="exec-value">{{ summary.execution_summary['Write Result Time'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Is Cached']" class="exec-item">
          <span class="exec-label">Is Cached</span>
          <span class="exec-value">{{ summary.execution_summary['Is Cached'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Total Instances Num']" class="exec-item">
          <span class="exec-label">Total Instances Num</span>
          <span class="exec-value">{{ summary.execution_summary['Total Instances Num'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Parallel Fragment Exec Instance Num']" class="exec-item">
          <span class="exec-label">Parallel Fragment Exec Instance Num</span>
          <span class="exec-value">{{ summary.execution_summary['Parallel Fragment Exec Instance Num'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Instances Num Per BE']" class="exec-item exec-item-wide">
          <span class="exec-label">Instances Num Per BE</span>
          <span class="exec-value exec-value-code">{{ summary.execution_summary['Instances Num Per BE'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Schedule Time Of BE']" class="exec-item exec-item-full">
          <span class="exec-label">Schedule Time Of BE</span>
          <span class="exec-value exec-value-code">{{ summary.execution_summary['Schedule Time Of BE'] }}</span>
        </div>
        <div v-if="summary?.execution_summary['Splits Assignment Weight']" class="exec-item exec-item-wide">
          <span class="exec-label">Splits Assignment Weight</span>
          <span class="exec-value exec-value-code">{{ summary.execution_summary['Splits Assignment Weight'] }}</span>
        </div>
      </div>
    </div>

    <!-- Session Variables Section -->
    <div v-if="summary?.session_variables && summary.session_variables.length > 0" class="card session-variables-section">
      <h3>Session Variables</h3>
      <div class="table-wrapper">
        <table class="session-variables-table">
          <thead>
            <tr>
              <th>Variable Name</th>
              <th>Current Value</th>
              <th>Default Value</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(variable, index) in summary.session_variables" :key="index">
              <td class="var-name">{{ variable.VarName }}</td>
              <td class="var-current" :class="{ 'value-changed': variable.CurrentValue !== variable.DefaultValue }">
                {{ variable.CurrentValue }}
              </td>
              <td class="var-default">{{ variable.DefaultValue }}</td>
            </tr>
          </tbody>
        </table>
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
    
    const hasExecutionSummary = computed(() => {
      return props.summary?.execution_summary && 
             Object.keys(props.summary.execution_summary).length > 0;
    });

    return {
      scoreClass,
      scoreCategory,
      statusClass,
      isFormatted,
      displayedSql,
      highlightedSql,
      compactHighlightedSql,
      toggleSqlFormat,
      hasExecutionSummary,
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
  grid-template-columns: 150px 1.5fr 1fr;
  gap: 24px;

  @media (max-width: 1200px) {
    grid-template-columns: 150px 1fr;
  }

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

.info-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 12px;
  
  &:last-child {
    margin-bottom: 0;
  }
  
  @media (max-width: 1200px) {
    grid-template-columns: repeat(2, 1fr);
  }
  
  @media (max-width: 768px) {
    grid-template-columns: 1fr;
  }
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;

  &.info-item-wide {
    grid-column: span 2;
    
    @media (max-width: 1200px) {
      grid-column: span 1;
    }
  }

  .info-label {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .info-value {
    font-weight: 500;
    color: var(--text-primary);
    word-break: break-all;

    &.query-id {
      word-break: break-all;
      overflow-wrap: anywhere;
      font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
      font-size: 13px;
    }

    &.total-time-highlight {
      color: #409EFF;
      font-weight: 700;
      font-size: 16px;
      font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
    }

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

.exec-summary-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border-light);

  h4 {
    margin: 0 0 16px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);

    i {
      margin-right: 8px;
    }
  }
}

.exec-summary-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  
  @media (max-width: 1200px) {
    grid-template-columns: repeat(2, 1fr);
  }
  
  @media (max-width: 768px) {
    grid-template-columns: 1fr;
  }
}

.exec-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border-light);

  &.exec-item-wide {
    grid-column: span 2;
    
    @media (max-width: 1200px) {
      grid-column: span 1;
    }
  }
  
  &.exec-item-full {
    grid-column: span 3;
    
    @media (max-width: 1200px) {
      grid-column: span 2;
    }
    
    @media (max-width: 768px) {
      grid-column: span 1;
    }
  }

  .exec-label {
    font-size: 11px;
    color: var(--text-tertiary);
    font-weight: 500;
  }

  .exec-value {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    word-break: break-word;
    
    &.exec-value-code {
      font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
      font-size: 11px;
      font-weight: 400;
      color: #666;
      white-space: pre-wrap;
      word-break: break-all;
    }
  }
}

/* Session Variables Section */
.session-variables-section {
  margin-top: 20px;
  
  h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }
  
  .table-wrapper {
    overflow-x: auto;
  }
  
  .session-variables-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
    
    thead {
      background-color: #f8f9fa;
      border-bottom: 2px solid #dee2e6;
      
      th {
        padding: 12px 16px;
        text-align: left;
        font-weight: 600;
        color: var(--text-secondary);
        font-size: 12px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }
    }
    
    tbody {
      tr {
        border-bottom: 1px solid #e9ecef;
        transition: background-color 0.2s ease;
        
        &:hover {
          background-color: #f8f9fa;
        }
        
        &:last-child {
          border-bottom: none;
        }
      }
      
      td {
        padding: 12px 16px;
        color: var(--text-primary);
        
        &.var-name {
          font-weight: 500;
          color: #2c5aa0;
        }
        
        &.var-current {
          font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
          font-size: 12px;
          
          &.value-changed {
            color: #d9534f;
            font-weight: 600;
          }
        }
        
        &.var-default {
          font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
          font-size: 12px;
          color: #6c757d;
        }
      }
    }
  }
}
</style>

