<template>
  <div class="dag-visualization">
    <!-- View Mode Toggle -->
    <div class="view-controls">
      <button 
        :class="['view-btn', { active: viewMode === 'table' }]"
        @click="viewMode = 'table'"
      >
        📋 Table View
      </button>
      <button 
        :class="['view-btn', { active: viewMode === 'tree' }]"
        @click="viewMode = 'tree'"
      >
        🌲 Tree View
      </button>
      <button 
        :class="['view-btn', { active: viewMode === 'graph' }]"
        @click="setGraphMode"
      >
        📊 Graph View
      </button>
      <span class="node-count">{{ nodeCount }} nodes</span>
    </div>

    <div v-if="!hasNodes" class="no-data">
      <p>No execution tree data available</p>
    </div>
    
    <!-- Table View -->
    <div v-else-if="viewMode === 'table'" class="table-view">
      <div class="table-header">
        <span class="col-fragment">Fragment</span>
        <span class="col-pipeline">Pipeline</span>
        <span class="col-operator">Operator</span>
        <span class="col-time">Time</span>
        <span class="col-pct">%</span>
        <span class="col-rows">Rows</span>
      </div>
      <div class="table-body">
        <div 
          v-for="node in sortedNodes" 
          :key="node.id"
          :class="['table-row', { hotspot: node.is_hotspot }]"
        >
          <span class="col-fragment">{{ getShortId(node.fragment_id) }}</span>
          <span class="col-pipeline">{{ getShortId(node.pipeline_id) }}</span>
          <span class="col-operator">
            <span 
              class="operator-dot" 
              :style="{ background: getNodeColor(node) }"
            ></span>
            {{ node.operator_name }}
          </span>
          <span class="col-time">{{ node.metrics?.operator_total_time_raw || '-' }}</span>
          <span class="col-pct" :class="getPctClass(node)">
            {{ formatPct(node.time_percentage) }}
          </span>
          <span class="col-rows">{{ formatNumber(node.metrics?.rows_returned) }}</span>
        </div>
      </div>
    </div>

    <!-- Tree View (Text-based) -->
    <div v-else-if="viewMode === 'tree'" class="tree-view">
      <div v-for="fragId in fragmentIds" :key="fragId" class="fragment-block">
        <div class="fragment-header">📁 {{ fragId }}</div>
        <div v-for="pipeId in getPipelineIds(fragId)" :key="pipeId" class="pipeline-block">
          <div class="pipeline-header">└─ 📦 {{ pipeId }}</div>
          <div 
            v-for="node in getNodesForPipeline(fragId, pipeId)" 
            :key="node.id"
            :class="['tree-node', { hotspot: node.is_hotspot }]"
          >
            <span class="tree-indent">   └─</span>
            <span 
              class="operator-dot" 
              :style="{ background: getNodeColor(node) }"
            ></span>
            <span class="node-name">{{ node.operator_name }}</span>
            <span v-if="node.time_percentage > 1" class="node-pct">
              ({{ node.time_percentage.toFixed(1) }}%)
            </span>
            <span v-if="node.metrics?.operator_total_time_raw" class="node-time">
              {{ node.metrics.operator_total_time_raw }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Graph View (Simple Canvas) -->
    <div v-else class="graph-view">
      <canvas ref="canvasRef" width="800" height="500" class="graph-canvas"></canvas>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch, onMounted, nextTick } from "vue";

export default {
  name: "DAGVisualization",
  props: {
    tree: {
      type: Object,
      default: null,
    },
  },
  setup(props) {
    const viewMode = ref('table');
    const canvasRef = ref(null);

    const hasNodes = computed(() => {
      return props.tree?.nodes?.length > 0;
    });

    const nodeCount = computed(() => props.tree?.nodes?.length || 0);

    const sortedNodes = computed(() => {
      if (!props.tree?.nodes) return [];
      return [...props.tree.nodes].sort((a, b) => {
        const fragA = a.fragment_id || '';
        const fragB = b.fragment_id || '';
        const fragCmp = fragA.localeCompare(fragB);
        if (fragCmp !== 0) return fragCmp;
        
        const pipeA = a.pipeline_id || '';
        const pipeB = b.pipeline_id || '';
        return pipeA.localeCompare(pipeB);
      });
    });

    const fragmentIds = computed(() => {
      if (!props.tree?.nodes) return [];
      const ids = new Set();
      props.tree.nodes.forEach(n => ids.add(n.fragment_id || 'Unknown'));
      return Array.from(ids).sort();
    });

    const getPipelineIds = (fragId) => {
      if (!props.tree?.nodes) return [];
      const ids = new Set();
      props.tree.nodes.forEach(n => {
        if ((n.fragment_id || 'Unknown') === fragId) {
          ids.add(n.pipeline_id || 'Unknown');
        }
      });
      return Array.from(ids).sort();
    };

    const getNodesForPipeline = (fragId, pipeId) => {
      if (!props.tree?.nodes) return [];
      return props.tree.nodes.filter(n => 
        (n.fragment_id || 'Unknown') === fragId && 
        (n.pipeline_id || 'Unknown') === pipeId
      );
    };

    const getShortId = (id) => {
      if (!id) return '-';
      return id.replace('Fragment ', 'F').replace('Pipeline ', 'P');
    };

    const formatNumber = (num) => {
      if (num === null || num === undefined) return "-";
      return num.toLocaleString();
    };

    const formatPct = (pct) => {
      if (pct === null || pct === undefined || pct === 0) return "-";
      return pct.toFixed(1) + '%';
    };

    const getPctClass = (node) => {
      const pct = node.time_percentage || 0;
      if (pct >= 30) return 'pct-critical';
      if (pct >= 15) return 'pct-high';
      if (pct >= 5) return 'pct-medium';
      return '';
    };

    const getNodeColor = (node) => {
      if (node.is_most_consuming) return "#e74c3c";
      if (node.is_second_most_consuming) return "#f39c12";
      if (node.is_hotspot) {
        switch (node.hotspot_severity) {
          case "Critical": return "#e74c3c";
          case "High": return "#f39c12";
          case "Medium": return "#3498db";
          default: return "#27ae60";
        }
      }
      
      const name = (node.operator_name || '').toUpperCase();
      if (name.includes("SCAN")) return "#9b59b6";
      if (name.includes("JOIN")) return "#e67e22";
      if (name.includes("AGGREGATE")) return "#1abc9c";
      if (name.includes("EXCHANGE")) return "#34495e";
      if (name.includes("SORT")) return "#16a085";
      if (name.includes("SINK")) return "#95a5a6";
      
      return "#3498db";
    };

    const setGraphMode = () => {
      viewMode.value = 'graph';
      nextTick(() => {
        renderCanvas();
      });
    };

    const renderCanvas = () => {
      if (!canvasRef.value || !props.tree?.nodes?.length) return;

      const canvas = canvasRef.value;
      const ctx = canvas.getContext('2d');
      const width = canvas.width;
      const height = canvas.height;

      // Clear canvas
      ctx.fillStyle = '#f5f7fa';
      ctx.fillRect(0, 0, width, height);

      const nodes = props.tree.nodes;
      const frags = fragmentIds.value;
      const fragHeight = height / Math.max(frags.length, 1);

      // Position nodes
      const positions = {};
      frags.forEach((fragId, fragIdx) => {
        const fragNodes = nodes.filter(n => (n.fragment_id || 'Unknown') === fragId);
        const nodeWidth = width / Math.max(fragNodes.length + 1, 1);
        
        fragNodes.forEach((node, nodeIdx) => {
          positions[node.id] = {
            x: 30 + (nodeIdx + 0.5) * nodeWidth,
            y: 30 + fragIdx * fragHeight + fragHeight / 2
          };
        });
      });

      // Draw fragment labels
      ctx.fillStyle = '#666';
      ctx.font = '11px sans-serif';
      frags.forEach((fragId, fragIdx) => {
        ctx.fillText(fragId, 5, 20 + fragIdx * fragHeight);
      });

      // Draw edges
      ctx.strokeStyle = '#ddd';
      ctx.lineWidth = 1;
      nodes.forEach(node => {
        const from = positions[node.id];
        if (!from) return;
        (node.children || []).forEach(childId => {
          const to = positions[childId];
          if (!to) return;
          ctx.beginPath();
          ctx.moveTo(from.x, from.y);
          ctx.lineTo(to.x, to.y);
          ctx.stroke();
        });
      });

      // Draw nodes
      nodes.forEach(node => {
        const pos = positions[node.id];
        if (!pos) return;
        
        const pct = node.time_percentage || 0;
        const radius = Math.max(6, Math.min(18, 6 + pct / 5));
        
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, radius, 0, Math.PI * 2);
        ctx.fillStyle = getNodeColor(node);
        ctx.fill();
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Label
        ctx.fillStyle = '#333';
        ctx.font = '9px sans-serif';
        ctx.textAlign = 'center';
        const label = (node.operator_name || '').replace(/_OPERATOR|_SINK/g, '').substring(0, 8);
        ctx.fillText(label, pos.x, pos.y + radius + 12);

        // Percentage for hotspots
        if (pct > 5) {
          ctx.fillStyle = getNodeColor(node);
          ctx.font = 'bold 10px sans-serif';
          ctx.fillText(pct.toFixed(0) + '%', pos.x, pos.y - radius - 5);
        }
      });
    };

    watch(() => props.tree, () => {
      if (viewMode.value === 'graph') {
        nextTick(renderCanvas);
      }
    }, { deep: true });

    return {
      viewMode,
      canvasRef,
      hasNodes,
      nodeCount,
      sortedNodes,
      fragmentIds,
      getPipelineIds,
      getNodesForPipeline,
      getShortId,
      formatNumber,
      formatPct,
      getPctClass,
      getNodeColor,
      setGraphMode,
    };
  },
};
</script>

<style lang="scss" scoped>
.dag-visualization {
  width: 100%;
}

.view-controls {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  padding: 10px;
  background: #f0f2f5;
  border-radius: 8px;
  align-items: center;
}

.view-btn {
  padding: 8px 14px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
  
  &:hover {
    border-color: #1890ff;
    color: #1890ff;
  }
  
  &.active {
    background: #1890ff;
    color: #fff;
    border-color: #1890ff;
  }
}

.node-count {
  margin-left: auto;
  font-size: 12px;
  color: #8c8c8c;
  background: #fff;
  padding: 4px 10px;
  border-radius: 10px;
}

.no-data {
  text-align: center;
  padding: 60px 20px;
  color: #8c8c8c;
  background: #fafafa;
  border-radius: 8px;
}

/* Table View */
.table-view {
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  overflow: hidden;
  max-height: 450px;
  overflow-y: auto;
}

.table-header {
  display: flex;
  background: #fafafa;
  padding: 12px;
  font-weight: 600;
  font-size: 12px;
  color: #595959;
  border-bottom: 1px solid #e8e8e8;
  position: sticky;
  top: 0;
  z-index: 1;
}

.table-body {
  font-size: 12px;
}

.table-row {
  display: flex;
  padding: 10px 12px;
  border-bottom: 1px solid #f0f0f0;
  align-items: center;
  
  &:hover {
    background: #f5f5f5;
  }
  
  &.hotspot {
    background: #fff1f0;
    
    &:hover {
      background: #ffccc7;
    }
  }
}

.col-fragment { width: 50px; flex-shrink: 0; color: #8c8c8c; }
.col-pipeline { width: 50px; flex-shrink: 0; color: #8c8c8c; }
.col-operator { 
  flex: 1; 
  display: flex; 
  align-items: center; 
  gap: 8px;
  font-weight: 500;
}
.col-time { width: 100px; flex-shrink: 0; text-align: right; color: #52c41a; }
.col-pct { width: 60px; flex-shrink: 0; text-align: right; font-weight: 600; }
.col-rows { width: 100px; flex-shrink: 0; text-align: right; color: #1890ff; }

.operator-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.pct-critical { color: #ff4d4f; }
.pct-high { color: #fa8c16; }
.pct-medium { color: #1890ff; }

/* Tree View */
.tree-view {
  font-family: 'SF Mono', 'Monaco', 'Menlo', 'Consolas', monospace;
  font-size: 13px;
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 8px;
  max-height: 450px;
  overflow-y: auto;
  line-height: 1.6;
}

.fragment-block {
  margin-bottom: 12px;
}

.fragment-header {
  color: #569cd6;
  font-weight: 600;
}

.pipeline-block {
  margin-left: 20px;
}

.pipeline-header {
  color: #4ec9b0;
}

.tree-node {
  margin-left: 20px;
  display: flex;
  align-items: center;
  gap: 6px;
  
  &.hotspot {
    background: rgba(255, 77, 79, 0.15);
    margin-left: 16px;
    padding: 2px 8px;
    border-radius: 4px;
  }
}

.tree-indent { color: #4d4d4d; }
.node-name { color: #ce9178; }
.node-pct { color: #ff4d4f; font-weight: 600; }
.node-time { color: #6a9955; }

/* Graph View */
.graph-view {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 10px;
  display: flex;
  justify-content: center;
}

.graph-canvas {
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  background: #fff;
}
</style>
