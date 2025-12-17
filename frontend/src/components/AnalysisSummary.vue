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
      <h4><i class="fas fa-code"></i> SQL Statement</h4>
      <div class="sql-container">
        <pre class="sql-code">{{ summary.sql_statement }}</pre>
      </div>
    </div>
  </div>
</template>

<script>
import { computed } from "vue";

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

    return {
      scoreClass,
      scoreCategory,
      statusClass,
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

  h4 {
    margin: 0 0 12px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);

    i {
      margin-right: 8px;
    }
  }
}

.sql-container {
  background: #1e1e1e;
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
}

.sql-code {
  margin: 0;
  font-family: "Consolas", "Monaco", "Courier New", monospace;
  font-size: 13px;
  line-height: 1.5;
  color: #d4d4d4;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>

