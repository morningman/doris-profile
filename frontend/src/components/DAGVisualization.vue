<template>
  <div class="dag-visualization">
    <!-- View Mode Toggle -->
    <div class="view-controls">
      <button 
        :class="['view-btn', { active: viewMode === 'table' }]"
        @click="viewMode = 'table'"
      >
        📋 Table
      </button>
      <button 
        :class="['view-btn', { active: viewMode === 'tree' }]"
        @click="viewMode = 'tree'"
      >
        🌲 Fragment Tree
      </button>
      <button 
        :class="['view-btn', { active: viewMode === 'pipeline' }]"
        @click="viewMode = 'pipeline'"
      >
        📦 Pipeline View
      </button>
      <span class="node-count">{{ nodeCount }} nodes | {{ fragmentCount }} fragments</span>
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
            <span class="operator-dot" :style="{ background: getNodeColor(node) }"></span>
            {{ node.operator_name }}
          </span>
          <span class="col-time">{{ node.metrics?.operator_total_time_raw || '-' }}</span>
          <span class="col-pct" :class="getPctClass(node)">{{ formatPct(node.time_percentage) }}</span>
          <span class="col-rows">{{ formatNumber(node.metrics?.rows_returned) }}</span>
        </div>
      </div>
    </div>

    <!-- Fragment Tree View (showing fragment hierarchy) -->
    <div v-else-if="viewMode === 'tree'" class="tree-view">
      <div class="tree-legend">
        <span class="legend-item"><span class="dot" style="background:#e74c3c"></span> Hotspot (&gt;30%)</span>
        <span class="legend-item"><span class="dot" style="background:#f39c12"></span> High (15-30%)</span>
        <span class="legend-item"><span class="dot" style="background:#9b59b6"></span> SCAN</span>
        <span class="legend-item"><span class="dot" style="background:#e67e22"></span> JOIN</span>
        <span class="legend-item"><span class="dot" style="background:#1abc9c"></span> AGG</span>
      </div>
      <div class="fragment-tree">
        <FragmentTreeNode 
          :fragment="rootFragment" 
          :fragmentTree="fragmentTree"
          :nodesByFragment="nodesByFragment"
          :getNodeColor="getNodeColor"
          :depth="0"
        />
      </div>
    </div>

    <!-- Pipeline View (flat view by fragment) -->
    <div v-else-if="viewMode === 'pipeline'" class="pipeline-view">
      <div v-for="fragId in fragmentIds" :key="fragId" class="fragment-section">
        <div class="fragment-title">
          <span class="frag-icon">📁</span> {{ fragId }}
          <span class="frag-stats">
            {{ getFragmentStats(fragId) }}
          </span>
        </div>
        <div class="pipelines-container">
          <div v-for="pipeId in getPipelineIds(fragId)" :key="pipeId" class="pipeline-section">
            <div class="pipeline-title">📦 {{ pipeId }}</div>
            <div class="operators-list">
              <div 
                v-for="node in getNodesForPipeline(fragId, pipeId)" 
                :key="node.id"
                :class="['operator-item', { hotspot: node.is_hotspot }]"
              >
                <span class="operator-dot" :style="{ background: getNodeColor(node) }"></span>
                <span class="op-name">{{ node.operator_name }}</span>
                <span v-if="node.time_percentage > 1" class="op-pct">{{ node.time_percentage.toFixed(1) }}%</span>
                <span class="op-time">{{ node.metrics?.operator_total_time_raw || '' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, h, defineComponent } from "vue";

// Recursive Fragment Tree Node Component
const FragmentTreeNode = defineComponent({
  name: 'FragmentTreeNode',
  props: ['fragment', 'fragmentTree', 'nodesByFragment', 'getNodeColor', 'depth'],
  setup(props) {
    const isExpanded = ref(true);
    
    const toggle = () => {
      isExpanded.value = !isExpanded.value;
    };
    
    const children = computed(() => {
      return props.fragmentTree[props.fragment] || [];
    });
    
    const nodes = computed(() => {
      return props.nodesByFragment[props.fragment] || [];
    });
    
    const totalTime = computed(() => {
      let total = 0;
      nodes.value.forEach(n => {
        if (n.metrics?.operator_total_time) {
          total += n.metrics.operator_total_time;
        }
      });
      if (total > 1000000000) return (total / 1000000000).toFixed(2) + 's';
      if (total > 1000000) return (total / 1000000).toFixed(2) + 'ms';
      if (total > 1000) return (total / 1000).toFixed(2) + 'us';
      return total + 'ns';
    });
    
    const hotspotNodes = computed(() => {
      return nodes.value.filter(n => n.is_hotspot);
    });
    
    return () => {
      const indent = props.depth * 24;
      
      return h('div', { class: 'tree-fragment', style: { marginLeft: indent + 'px' } }, [
        // Fragment header
        h('div', { 
          class: ['fragment-header', { 'has-hotspot': hotspotNodes.value.length > 0 }],
          onClick: toggle 
        }, [
          h('span', { class: 'expand-icon' }, isExpanded.value ? '▼' : '▶'),
          h('span', { class: 'frag-name' }, props.fragment),
          h('span', { class: 'frag-time' }, totalTime.value),
          hotspotNodes.value.length > 0 && h('span', { class: 'hotspot-badge' }, 
            `🔥 ${hotspotNodes.value.length} hotspot${hotspotNodes.value.length > 1 ? 's' : ''}`
          ),
        ]),
        
        // Fragment content (when expanded)
        isExpanded.value && h('div', { class: 'fragment-content' }, [
          // Operators in this fragment
          h('div', { class: 'operators-block' }, 
            nodes.value.map(node => 
              h('div', { 
                class: ['tree-operator', { hotspot: node.is_hotspot }],
                key: node.id 
              }, [
                h('span', { 
                  class: 'operator-dot', 
                  style: { background: props.getNodeColor(node) } 
                }),
                h('span', { class: 'op-name' }, node.operator_name),
                node.time_percentage > 0.1 && h('span', { 
                  class: ['op-pct', node.time_percentage > 30 ? 'critical' : node.time_percentage > 15 ? 'high' : ''] 
                }, `${node.time_percentage.toFixed(1)}%`),
                node.metrics?.operator_total_time_raw && h('span', { class: 'op-time' }, 
                  node.metrics.operator_total_time_raw
                ),
              ])
            )
          ),
          
          // Child fragments
          children.value.length > 0 && h('div', { class: 'children-block' }, 
            children.value.map(childFrag => 
              h(FragmentTreeNode, {
                key: childFrag,
                fragment: childFrag,
                fragmentTree: props.fragmentTree,
                nodesByFragment: props.nodesByFragment,
                getNodeColor: props.getNodeColor,
                depth: props.depth + 1,
              })
            )
          ),
        ]),
      ]);
    };
  }
});

export default {
  name: "DAGVisualization",
  components: { FragmentTreeNode },
  props: {
    tree: {
      type: Object,
      default: null,
    },
  },
  setup(props) {
    const viewMode = ref('tree');

    const hasNodes = computed(() => props.tree?.nodes?.length > 0);
    const nodeCount = computed(() => props.tree?.nodes?.length || 0);
    const fragmentCount = computed(() => fragmentIds.value.length);

    const fragmentIds = computed(() => {
      if (!props.tree?.nodes) return [];
      const ids = new Set();
      props.tree.nodes.forEach(n => ids.add(n.fragment_id || 'Unknown'));
      return Array.from(ids).sort();
    });

    const nodesByFragment = computed(() => {
      if (!props.tree?.nodes) return {};
      const result = {};
      props.tree.nodes.forEach(node => {
        const fragId = node.fragment_id || 'Unknown';
        if (!result[fragId]) result[fragId] = [];
        result[fragId].push(node);
      });
      return result;
    });

    // Build fragment tree based on DATA_STREAM_SINK -> EXCHANGE relationships
    const fragmentTree = computed(() => {
      if (!props.tree?.nodes) return {};
      
      const tree = {}; // parent -> [children]
      const childToParent = {}; // child -> parent
      
      // Find all EXCHANGE operators and their fragment locations
      const exchangeToFragment = {}; // exchange_id -> fragment_id
      props.tree.nodes.forEach(node => {
        if (node.operator_name?.includes('EXCHANGE_OPERATOR') && 
            !node.operator_name?.includes('LOCAL_EXCHANGE') &&
            !node.operator_name?.includes('SINK')) {
          const id = node.plan_node_id;
          if (id !== null && id !== undefined) {
            exchangeToFragment[id] = node.fragment_id;
          }
        }
      });
      
      // Find all DATA_STREAM_SINK operators and link to their target EXCHANGE
      props.tree.nodes.forEach(node => {
        if (node.operator_name?.includes('DATA_STREAM_SINK')) {
          const destId = node.unique_metrics?.dest_id;
          if (destId) {
            const destIdNum = parseInt(destId);
            const parentFrag = exchangeToFragment[destIdNum];
            const childFrag = node.fragment_id;
            
            if (parentFrag && childFrag && parentFrag !== childFrag) {
              if (!tree[parentFrag]) tree[parentFrag] = [];
              if (!tree[parentFrag].includes(childFrag)) {
                tree[parentFrag].push(childFrag);
              }
              childToParent[childFrag] = parentFrag;
            }
          }
        }
      });
      
      // Sort children
      Object.keys(tree).forEach(k => {
        tree[k].sort();
      });
      
      return tree;
    });

    // Find root fragment (fragment with no parent)
    const rootFragment = computed(() => {
      const allFragments = new Set(fragmentIds.value);
      const childFragments = new Set();
      
      Object.values(fragmentTree.value).forEach(children => {
        children.forEach(c => childFragments.add(c));
      });
      
      for (const frag of allFragments) {
        if (!childFragments.has(frag)) {
          return frag;
        }
      }
      
      return fragmentIds.value[0] || 'Unknown';
    });

    const sortedNodes = computed(() => {
      if (!props.tree?.nodes) return [];
      return [...props.tree.nodes].sort((a, b) => {
        const fragCmp = (a.fragment_id || '').localeCompare(b.fragment_id || '');
        if (fragCmp !== 0) return fragCmp;
        return (a.pipeline_id || '').localeCompare(b.pipeline_id || '');
      });
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

    const getFragmentStats = (fragId) => {
      const nodes = nodesByFragment.value[fragId] || [];
      const hotspots = nodes.filter(n => n.is_hotspot).length;
      let totalTime = 0;
      nodes.forEach(n => {
        if (n.metrics?.operator_total_time) totalTime += n.metrics.operator_total_time;
      });
      
      let timeStr = '';
      if (totalTime > 1000000000) timeStr = (totalTime / 1000000000).toFixed(1) + 's';
      else if (totalTime > 1000000) timeStr = (totalTime / 1000000).toFixed(1) + 'ms';
      else if (totalTime > 1000) timeStr = (totalTime / 1000).toFixed(1) + 'us';
      else timeStr = totalTime + 'ns';
      
      return `${nodes.length} ops, ${timeStr}${hotspots ? `, 🔥${hotspots}` : ''}`;
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
      if (name.includes("AGGREGATE") || name.includes("AGGREGATION")) return "#1abc9c";
      if (name.includes("EXCHANGE") && !name.includes("LOCAL")) return "#34495e";
      if (name.includes("SORT")) return "#16a085";
      if (name.includes("SINK")) return "#95a5a6";
      
      return "#3498db";
    };

    return {
      viewMode,
      hasNodes,
      nodeCount,
      fragmentCount,
      fragmentIds,
      nodesByFragment,
      fragmentTree,
      rootFragment,
      sortedNodes,
      getPipelineIds,
      getNodesForPipeline,
      getFragmentStats,
      getShortId,
      formatNumber,
      formatPct,
      getPctClass,
      getNodeColor,
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
  flex-wrap: wrap;
}

.view-btn {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  font-size: 13px;
  
  &:hover { border-color: #1890ff; color: #1890ff; }
  &.active { background: #1890ff; color: #fff; border-color: #1890ff; }
}

.node-count {
  margin-left: auto;
  font-size: 12px;
  color: #8c8c8c;
}

.no-data {
  text-align: center;
  padding: 60px;
  color: #8c8c8c;
  background: #fafafa;
  border-radius: 8px;
}

/* Table View */
.table-view {
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  overflow: hidden;
  max-height: 500px;
  overflow-y: auto;
}

.table-header {
  display: flex;
  background: #fafafa;
  padding: 10px 12px;
  font-weight: 600;
  font-size: 12px;
  color: #595959;
  border-bottom: 1px solid #e8e8e8;
  position: sticky;
  top: 0;
  z-index: 1;
}

.table-row {
  display: flex;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-size: 12px;
  align-items: center;
  
  &:hover { background: #f5f5f5; }
  &.hotspot { background: #fff1f0; &:hover { background: #ffccc7; } }
}

.col-fragment { width: 45px; flex-shrink: 0; color: #8c8c8c; }
.col-pipeline { width: 45px; flex-shrink: 0; color: #8c8c8c; }
.col-operator { flex: 1; display: flex; align-items: center; gap: 6px; font-weight: 500; }
.col-time { width: 90px; flex-shrink: 0; text-align: right; color: #52c41a; font-size: 11px; }
.col-pct { width: 55px; flex-shrink: 0; text-align: right; font-weight: 600; }
.col-rows { width: 80px; flex-shrink: 0; text-align: right; color: #1890ff; font-size: 11px; }

.operator-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.pct-critical { color: #ff4d4f; }
.pct-high { color: #fa8c16; }
.pct-medium { color: #1890ff; }

/* Fragment Tree View */
.tree-view {
  background: #1a1a2e;
  border-radius: 8px;
  padding: 16px;
  max-height: 600px;
  overflow-y: auto;
  color: #e0e0e0;
  font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
  font-size: 12px;
}

.tree-legend {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #333;
  font-size: 11px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #888;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.fragment-tree {
  line-height: 1.8;
}

:deep(.tree-fragment) {
  margin-bottom: 4px;
}

:deep(.fragment-header) {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
  
  &:hover { background: rgba(255,255,255,0.05); }
  &.has-hotspot { background: rgba(231, 76, 60, 0.15); }
}

:deep(.expand-icon) {
  width: 16px;
  font-size: 10px;
  color: #666;
}

:deep(.frag-name) {
  color: #61afef;
  font-weight: 600;
}

:deep(.frag-time) {
  color: #98c379;
  font-size: 11px;
}

:deep(.hotspot-badge) {
  background: rgba(231, 76, 60, 0.2);
  color: #e74c3c;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 10px;
  margin-left: auto;
}

:deep(.fragment-content) {
  margin-left: 24px;
  border-left: 1px dashed #333;
  padding-left: 12px;
}

:deep(.operators-block) {
  margin: 4px 0;
}

:deep(.tree-operator) {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 3px 6px;
  border-radius: 3px;
  
  &:hover { background: rgba(255,255,255,0.03); }
  &.hotspot { 
    background: rgba(231, 76, 60, 0.1);
    border-left: 2px solid #e74c3c;
  }
}

:deep(.op-name) {
  color: #e5c07b;
}

:deep(.op-pct) {
  color: #56b6c2;
  font-size: 11px;
  
  &.critical { color: #e74c3c; font-weight: 600; }
  &.high { color: #f39c12; font-weight: 600; }
}

:deep(.op-time) {
  color: #5c6370;
  font-size: 10px;
  margin-left: auto;
}

:deep(.children-block) {
  margin-top: 8px;
}

/* Pipeline View */
.pipeline-view {
  max-height: 600px;
  overflow-y: auto;
}

.fragment-section {
  margin-bottom: 16px;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  overflow: hidden;
}

.fragment-title {
  background: #f5f7fa;
  padding: 10px 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid #e8e8e8;
}

.frag-icon { font-size: 16px; }

.frag-stats {
  margin-left: auto;
  font-size: 11px;
  color: #8c8c8c;
  font-weight: normal;
}

.pipelines-container {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px;
}

.pipeline-section {
  flex: 1;
  min-width: 200px;
  max-width: 300px;
  background: #fafafa;
  border-radius: 6px;
  padding: 10px;
}

.pipeline-title {
  font-weight: 500;
  margin-bottom: 8px;
  color: #595959;
  font-size: 13px;
}

.operators-list {
  font-size: 11px;
}

.operator-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 4px;
  margin-bottom: 2px;
  
  &:hover { background: #f0f0f0; }
  &.hotspot { background: #fff1f0; }
}

.op-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.op-pct {
  color: #fa8c16;
  font-weight: 600;
}

.op-time {
  color: #52c41a;
  font-size: 10px;
}
</style>
