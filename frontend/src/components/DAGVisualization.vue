<template>
  <div class="dag-visualization">
    <!-- View Mode Toggle -->
    <div class="view-controls">
      <button 
        :class="['view-btn', { active: viewMode === 'graph' }]"
        @click="viewMode = 'graph'"
      >
        📊 Graph View
      </button>
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
    
    <!-- Graph View (SVG DAG Visualization) -->
    <div v-else-if="viewMode === 'graph'" class="graph-view-container">
      <!-- 工具栏 -->
      <div class="dag-toolbar">
        <button @click="zoomIn" class="toolbar-btn" title="放大">
          <i class="fas fa-search-plus"></i>
        </button>
        <button @click="zoomOut" class="toolbar-btn" title="缩小">
          <i class="fas fa-search-minus"></i>
        </button>
        <button @click="fitToScreen" class="toolbar-btn" title="适应屏幕">
          <i class="fas fa-expand"></i>
        </button>
        <button @click="resetView" class="toolbar-btn" title="重置视图">
          <i class="fas fa-redo"></i>
        </button>
      </div>

      <!-- SVG 画布 -->
      <div class="svg-wrapper">
        <svg
          ref="svgCanvas"
          class="dag-svg"
          :width="svgWidth"
          :height="svgHeight"
          @wheel.prevent="handleWheel"
          @mousedown="startPan"
          @mousemove="doPan"
          @mouseup="endPan"
          @mouseleave="endPan"
        >
          <defs>
            <marker id="arrow" markerWidth="6" markerHeight="6" refX="4" refY="3" orient="auto">
              <polygon points="0 0, 5 3, 0 6" fill="#999" />
            </marker>
            <marker id="arrow-hotspot" markerWidth="6" markerHeight="6" refX="4" refY="3" orient="auto">
              <polygon points="0 0, 5 3, 0 6" fill="#E57373" />
            </marker>
            <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
              <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#f5f5f5" stroke-width="0.5" />
            </pattern>
          </defs>

          <rect width="100%" height="100%" fill="url(#grid)" @click="deselectNode" />

          <g :transform="`translate(${panX}, ${panY}) scale(${zoom})`">
            <!-- 连接线 -->
            <g class="links-group">
              <path
                v-for="link in links"
                :key="`link-${link.id}`"
                :d="link.path"
                class="connection-line"
                :class="{ 'hotspot-link': link.isHotspot }"
                :marker-end="`url(#${link.isHotspot ? 'arrow-hotspot' : 'arrow'})`"
                :style="{ strokeWidth: link.strokeWidth + 'px' }"
              />
              <text
                v-for="link in links"
                :key="`label-${link.id}`"
                :x="link.labelX"
                :y="link.labelY"
                class="link-label"
              >
                {{ link.label }}
              </text>
            </g>

            <!-- 节点 -->
            <g class="nodes-group">
              <g
                v-for="node in renderedNodes"
                :key="node.id"
                :transform="`translate(${node.x}, ${node.y})`"
                class="node-group"
                :class="{ selected: selectedNodeId === node.id, hotspot: node.is_hotspot }"
                @click.stop="selectNode(node)"
              >
                <rect class="node-header" :class="`header-${getNodeColorClass(node)}`" :width="NODE_WIDTH" :height="NODE_HEADER_HEIGHT" rx="3" />
                <rect class="node-body" :width="NODE_WIDTH" :y="NODE_HEADER_HEIGHT" :height="NODE_BODY_HEIGHT" />
                <rect class="progress-bg" :y="NODE_HEADER_HEIGHT + NODE_BODY_HEIGHT" :width="NODE_WIDTH" :height="NODE_PROGRESS_HEIGHT" />
                <rect v-if="node.time_percentage" class="progress-fill" :y="NODE_HEADER_HEIGHT + NODE_BODY_HEIGHT" :width="getProgressWidth(node)" :height="NODE_PROGRESS_HEIGHT" :fill="getProgressColor(node)" />
                <rect class="node-border" :width="NODE_WIDTH" :height="NODE_HEIGHT" rx="3" />
                
                <!-- 节点标题（显示合并标记） -->
                <text class="node-title" x="10" :y="19">
                  {{ formatOperatorName(node.operator_name) }}
                  <tspan v-if="node.isMerged" class="merged-badge" dx="5" style="font-size: 12px; fill: #FFD700;">⚡</tspan>
                </text>
                
                <!-- 节点详情 -->
                <template v-if="node.isMerged">
                  <!-- 合并节点显示两个节点的简化信息 -->
                  <text class="node-detail-small" x="10" :y="NODE_HEADER_HEIGHT + 12" style="font-size: 10px;">
                    {{ node.primaryNode.operator_name }}
                  </text>
                  <text class="node-detail-small" x="10" :y="NODE_HEADER_HEIGHT + 24" style="font-size: 10px;">
                    + {{ node.secondaryNode.operator_name }}
                  </text>
                  <text class="node-detail" x="10" :y="NODE_HEADER_HEIGHT + 40">
                    总耗时: {{ formatGraphTime(node) }}
                  </text>
                  <text class="node-percentage" :x="NODE_WIDTH - 10" :y="NODE_HEADER_HEIGHT + 40" text-anchor="end">
                    {{ formatPct(node.time_percentage) }}
                  </text>
                </template>
                <template v-else>
                  <!-- 普通节点 -->
                  <text class="node-detail" x="10" :y="NODE_HEADER_HEIGHT + 15">
                    plan_node_id={{ node.plan_node_id || 'N/A' }}
                  </text>
                  <text class="node-detail" x="10" :y="NODE_HEADER_HEIGHT + 32">
                    耗时: {{ formatGraphTime(node) }}
                  </text>
                  <text class="node-percentage" :x="NODE_WIDTH - 10" :y="NODE_HEADER_HEIGHT + 32" text-anchor="end">
                    {{ formatPct(node.time_percentage) }}
                  </text>
                </template>
              </g>
            </g>
          </g>
        </svg>
      </div>

      <!-- 节点详情面板 -->
      <transition name="slide">
        <div v-if="selectedNode" class="detail-panel">
          <div class="detail-header">
            <h3>{{ selectedNode.operator_name }}</h3>
            <button @click="deselectNode" class="close-btn"><i class="fas fa-times"></i></button>
          </div>
          <div class="detail-content">
            <!-- 合并节点显示 -->
            <div v-if="selectedNode.isMerged" class="merged-node-details">
              <div class="detail-section merged-indicator">
                <h4>⚡ 合并节点</h4>
                <div class="detail-item">
                  <span class="label">类型:</span>
                  <span class="value">{{ selectedNode.mergedType }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">总时间:</span>
                  <span class="value">{{ formatGraphTime(selectedNode) }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">总占比:</span>
                  <span class="value">{{ formatPct(selectedNode.time_percentage) }}</span>
                </div>
              </div>

              <!-- 主节点信息 -->
              <div class="detail-section sub-node-section">
                <h4>🔹 {{ selectedNode.primaryNode.operator_name }}</h4>
                <div class="detail-item">
                  <span class="label">Fragment:</span>
                  <span class="value">{{ selectedNode.primaryNode.fragment_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Pipeline:</span>
                  <span class="value">{{ selectedNode.primaryNode.pipeline_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">执行时间:</span>
                  <span class="value">{{ formatGraphTime(selectedNode.primaryNode) }}</span>
                </div>
              </div>

              <!-- 次节点信息 (SINK) -->
              <div class="detail-section sub-node-section">
                <h4>🔹 {{ selectedNode.secondaryNode.operator_name }}</h4>
                <div class="detail-item">
                  <span class="label">Fragment:</span>
                  <span class="value">{{ selectedNode.secondaryNode.fragment_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Pipeline:</span>
                  <span class="value">{{ selectedNode.secondaryNode.pipeline_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">执行时间:</span>
                  <span class="value">{{ formatGraphTime(selectedNode.secondaryNode) }}</span>
                </div>
              </div>
            </div>

            <!-- 普通节点显示 -->
            <div v-else>
              <div class="detail-section">
                <h4>基本信息</h4>
                <div class="detail-item"><span class="label">Plan Node ID:</span><span class="value">{{ selectedNode.plan_node_id }}</span></div>
                <div class="detail-item"><span class="label">Fragment:</span><span class="value">{{ selectedNode.fragment_id }}</span></div>
                <div class="detail-item"><span class="label">Pipeline:</span><span class="value">{{ selectedNode.pipeline_id }}</span></div>
              </div>
              <div class="detail-section">
                <h4>性能指标</h4>
                <div class="detail-item"><span class="label">执行时间:</span><span class="value">{{ formatGraphTime(selectedNode) }}</span></div>
                <div class="detail-item"><span class="label">时间占比:</span><span class="value">{{ formatPct(selectedNode.time_percentage) }}</span></div>
                <div class="detail-item"><span class="label">处理行数:</span><span class="value">{{ formatNumber(selectedNode.metrics?.rows_returned) }}</span></div>
              </div>
            </div>
          </div>
        </div>
      </transition>
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

    <!-- Fragment Tree View -->
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

    <!-- Pipeline View -->
    <div v-else-if="viewMode === 'pipeline'" class="pipeline-view">
      <div v-for="fragId in fragmentIds" :key="fragId" class="fragment-section">
        <div class="fragment-title">
          <span class="frag-icon">📁</span> {{ fragId }}
          <span class="frag-stats">{{ getFragmentStats(fragId) }}</span>
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
                <span v-if="node.time_percentage > 1" class="op-pct">{{ formatPct(node.time_percentage) }}</span>
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
    const toggle = () => { isExpanded.value = !isExpanded.value; };
    const children = computed(() => props.fragmentTree[props.fragment] || []);
    const nodes = computed(() => props.nodesByFragment[props.fragment] || []);
    
    return () => {
      const indent = props.depth * 24;
      return h('div', { class: 'fragment-node', style: { marginLeft: `${indent}px` } }, [
        h('div', { 
          class: 'fragment-header',
          onClick: toggle 
        }, [
          h('span', { class: 'toggle-icon' }, isExpanded.value ? '▼' : '▶'),
          h('span', { class: 'fragment-icon' }, '📁'),
          h('span', { class: 'fragment-name' }, props.fragment),
          h('span', { class: 'fragment-node-count' }, `${nodes.value.length} nodes`)
        ]),
        isExpanded.value && nodes.value.length > 0 && h('div', { class: 'fragment-nodes' },
          nodes.value.map(node => h('div', { 
            class: ['node-item', { hotspot: node.is_hotspot }],
            key: node.id 
          }, [
            h('span', { class: 'node-dot', style: { background: props.getNodeColor(node) } }),
            h('span', { class: 'node-op-name' }, node.operator_name),
            node.time_percentage > 1 && h('span', { class: 'node-pct' }, `${node.time_percentage.toFixed(1)}%`)
          ]))
        ),
        isExpanded.value && children.value.length > 0 && h('div', { class: 'fragment-children' },
          children.value.map(childFrag => h(FragmentTreeNode, {
            key: childFrag,
            fragment: childFrag,
            fragmentTree: props.fragmentTree,
            nodesByFragment: props.nodesByFragment,
            getNodeColor: props.getNodeColor,
            depth: props.depth + 1
          }))
        )
      ]);
    };
  }
});

export default {
  name: 'DAGVisualization',
  components: { FragmentTreeNode },
  props: {
    tree: { type: Object, required: true }
  },
  data() {
    return {
      viewMode: 'graph', // Default to graph view
      
      // Graph view data
      NODE_WIDTH: 200,
      NODE_HEIGHT: 90,
      NODE_HEADER_HEIGHT: 28,
      NODE_BODY_HEIGHT: 56,
      NODE_PROGRESS_HEIGHT: 6,
      zoom: 1,
      panX: 50,
      panY: 50,
      isPanning: false,
      panStartX: 0,
      panStartY: 0,
      svgWidth: 1200,
      svgHeight: 800,
      renderedNodes: [],
      links: [],
      selectedNodeId: null,
      selectedNode: null,
      maxTime: 0,
    };
  },
  computed: {
    hasNodes() {
      return this.tree?.nodes && this.tree.nodes.length > 0;
    },
    nodeCount() {
      return this.tree?.nodes?.length || 0;
    },
    fragmentCount() {
      if (!this.hasNodes) return 0;
      const fragments = new Set(this.tree.nodes.map(n => n.fragment_id).filter(Boolean));
      return fragments.size;
    },
    sortedNodes() {
      if (!this.hasNodes) return [];
      return [...this.tree.nodes].sort((a, b) => (b.time_percentage || 0) - (a.time_percentage || 0));
    },
    fragmentIds() {
      if (!this.hasNodes) return [];
      const ids = new Set(this.tree.nodes.map(n => n.fragment_id).filter(Boolean));
      return Array.from(ids).sort((a, b) => {
        const numA = parseInt(a.replace('Fragment ', ''));
        const numB = parseInt(b.replace('Fragment ', ''));
        return numA - numB;
      });
    },
    nodesByFragment() {
      const map = {};
      if (!this.hasNodes) return map;
      this.tree.nodes.forEach(node => {
        const fid = node.fragment_id;
        if (fid) {
          if (!map[fid]) map[fid] = [];
          map[fid].push(node);
        }
      });
      return map;
    },
    fragmentTree() {
      const tree = {};
      if (!this.hasNodes) return tree;
      
      // Build fragment parent-child relationships from DATA_STREAM_SINK -> EXCHANGE connections
      this.tree.nodes.forEach(node => {
        if (node.operator_name && node.operator_name.includes('DATA_STREAM_SINK')) {
          // This node sends data, find its EXCHANGE children
          if (node.children && node.children.length > 0) {
            node.children.forEach(childId => {
              const child = this.tree.nodes.find(n => n.id === childId);
              if (child && child.operator_name && child.operator_name.includes('EXCHANGE') && !child.operator_name.includes('LOCAL')) {
                // Child is in different fragment
                const parentFrag = node.fragment_id;
                const childFrag = child.fragment_id;
                if (parentFrag && childFrag && parentFrag !== childFrag) {
                  if (!tree[parentFrag]) tree[parentFrag] = [];
                  if (!tree[parentFrag].includes(childFrag)) {
                    tree[parentFrag].push(childFrag);
                  }
                }
              }
            });
          }
        }
      });
      return tree;
    },
    rootFragment() {
      // Find fragment with no parent
      const allFragments = this.fragmentIds;
      const childFragments = new Set();
      Object.values(this.fragmentTree).forEach(children => {
        children.forEach(child => childFragments.add(child));
      });
      const root = allFragments.find(f => !childFragments.has(f));
      return root || (allFragments.length > 0 ? allFragments[0] : null);
    }
  },
  watch: {
    tree: {
      handler() {
        if (this.viewMode === 'graph') {
          this.$nextTick(() => this.renderDAG());
        }
      },
      deep: true,
      immediate: true
    },
    viewMode(newMode) {
      if (newMode === 'graph') {
        this.$nextTick(() => this.renderDAG());
      }
    }
  },
  mounted() {
    if (this.viewMode === 'graph') {
      this.updateSvgSize();
      this.renderDAG();
    }
    // 监听窗口大小变化
    window.addEventListener('resize', this.handleResize);
  },
  beforeUnmount() {
    window.removeEventListener('resize', this.handleResize);
  },
  methods: {
    // 更新 SVG 尺寸以适应容器
    updateSvgSize() {
      if (this.$refs.svgCanvas) {
        const container = this.$refs.svgCanvas.parentElement;
        if (container) {
          this.svgWidth = container.clientWidth || 1200;
          this.svgHeight = container.clientHeight || 600;
        }
      }
    },
    // 处理窗口大小变化
    handleResize() {
      this.updateSvgSize();
      if (this.viewMode === 'graph' && this.hasNodes) {
        this.renderDAG();
      }
    },
    // Graph view methods
    renderDAG() {
      if (!this.hasNodes) {
        this.renderedNodes = [];
        this.links = [];
        return;
      }

      const nodeMap = new Map();
      const nodesByDepth = new Map();
      
      this.tree.nodes.forEach(node => {
        nodeMap.set(node.id, node);
        const depth = node.depth || 0;
        if (!nodesByDepth.has(depth)) nodesByDepth.set(depth, []);
        nodesByDepth.get(depth).push(node);
      });

      // 步骤1: 识别需要合并的节点对
      const mergedNodeIds = new Set(); // 被合并掉的 SINK 节点 ID
      const mergeMap = new Map(); // originalId -> mergedNode
      
      // 1.1 合并 LOCAL_EXCHANGE_SINK + LOCAL_EXCHANGE
      this.tree.nodes.forEach(sinkNode => {
        if (sinkNode.operator_name && sinkNode.operator_name.includes('LOCAL_EXCHANGE_SINK')) {
          const exchangeNode = this.tree.nodes.find(n => 
            n.operator_name && n.operator_name.includes('LOCAL_EXCHANGE_OPERATOR') &&
            !n.operator_name.includes('SINK') &&
            n.fragment_id === sinkNode.fragment_id &&
            n.plan_node_id === sinkNode.plan_node_id
          );
          if (exchangeNode) {
            const mergedNode = this.createMergedNode(exchangeNode, sinkNode, 'LOCAL_EXCHANGE');
            mergeMap.set(exchangeNode.id, mergedNode);
            mergeMap.set(sinkNode.id, mergedNode);
            mergedNodeIds.add(sinkNode.id);
          }
        }
      });

      // 1.2 合并 DATA_STREAM_SINK + EXCHANGE
      this.tree.nodes.forEach(sinkNode => {
        if (sinkNode.operator_name && sinkNode.operator_name.includes('DATA_STREAM_SINK')) {
          const destId = sinkNode.unique_metrics?.dest_id;
          if (destId) {
            const exchangeNode = this.tree.nodes.find(n => 
              n.operator_name && n.operator_name.includes('EXCHANGE_OPERATOR') &&
              !n.operator_name.includes('SINK') &&
              !n.operator_name.includes('LOCAL') &&
              n.plan_node_id === parseInt(destId)
            );
            if (exchangeNode) {
              const mergedNode = this.createMergedNode(exchangeNode, sinkNode, 'DATA_STREAM');
              mergeMap.set(exchangeNode.id, mergedNode);
              mergeMap.set(sinkNode.id, mergedNode);
              mergedNodeIds.add(sinkNode.id);
            }
          }
        }
      });

      // 1.3 合并 AGGREGATION_SINK + AGGREGATION
      this.tree.nodes.forEach(sinkNode => {
        if (sinkNode.operator_name && sinkNode.operator_name.includes('AGGREGATION_SINK')) {
          const aggNode = this.tree.nodes.find(n => 
            n.operator_name && n.operator_name.includes('AGGREGATION_OPERATOR') &&
            !n.operator_name.includes('SINK') &&
            n.fragment_id === sinkNode.fragment_id &&
            n.plan_node_id === sinkNode.plan_node_id
          );
          if (aggNode) {
            const mergedNode = this.createMergedNode(aggNode, sinkNode, 'AGGREGATION');
            mergeMap.set(aggNode.id, mergedNode);
            mergeMap.set(sinkNode.id, mergedNode);
            mergedNodeIds.add(sinkNode.id);
          }
        }
      });

      // 步骤2: 创建可见节点列表（过滤掉被合并的 SINK 节点）
      const visibleNodes = [];
      const processedIds = new Set();
      
      this.tree.nodes.forEach(node => {
        if (mergedNodeIds.has(node.id)) {
          // 跳过被合并的 SINK 节点
          return;
        }
        
        if (mergeMap.has(node.id)) {
          const mergedNode = mergeMap.get(node.id);
          if (!processedIds.has(mergedNode.id)) {
            visibleNodes.push(mergedNode);
            processedIds.add(mergedNode.id);
          }
        } else {
          // 创建节点副本，避免修改原始数据
          visibleNodes.push({
            ...node,
            children: node.children ? [...node.children] : []
          });
        }
      });

      // 步骤3: 更新 children 引用
      // 关键修复：不要将指向 SINK 的引用重定向到合并节点，而是直接移除
      // 因为合并节点已经包含了 SINK 的所有 children
      visibleNodes.forEach(node => {
        if (node.children && node.children.length > 0) {
          node.children = node.children.filter(childId => {
            // 如果 child 是被合并掉的 SINK 节点，移除这个引用
            if (mergedNodeIds.has(childId)) {
              console.log(`移除 ${node.operator_name} -> 被合并的SINK(${childId}) 的连接`);
              return false;
            }
            // 避免自引用
            if (childId === node.id) {
              return false;
            }
            return true;
          });
        }
      });

      // 重新构建深度映射
      const visibleNodesByDepth = new Map();
      visibleNodes.forEach(node => {
        const depth = node.depth || 0;
        if (!visibleNodesByDepth.has(depth)) visibleNodesByDepth.set(depth, []);
        visibleNodesByDepth.get(depth).push(node);
      });

      this.maxTime = Math.max(...visibleNodes.map(n => this.getNodeTime(n)), 1);

      // 更新 SVG 尺寸以适应容器
      this.updateSvgSize();
      
      // 保存容器宽度，确保 SVG 不超出
      const containerWidth = this.svgWidth;
      
      // 优化布局：使用重心启发式算法减少连线交叉
      const LEVEL_HEIGHT = 200;  // 垂直间距
      const LEVEL_WIDTH = 350;   // 横向间距
      let maxDepth = Math.max(...visibleNodes.map(n => n.depth || 0));
      const calculatedHeight = (maxDepth + 1) * LEVEL_HEIGHT + 150;
      // SVG 高度取容器高度和计算高度的较大值（垂直方向可以滚动）
      this.svgHeight = Math.max(this.svgHeight, calculatedHeight);

      // 构建节点映射
      const nodeIdMap = new Map(visibleNodes.map(n => [n.id, n]));
      
      // 第一步：为每个节点分配初始水平位置（基于重心）
      const nodePositions = new Map(); // nodeId -> x position
      
      // 从最深层开始向上遍历（自底向上）
      const depths = Array.from(visibleNodesByDepth.keys()).sort((a, b) => b - a);
      
      depths.forEach(depth => {
        const levelNodes = visibleNodesByDepth.get(depth);
        
        // 计算每个节点的"理想位置"（基于其子节点或父节点的位置）
        const nodeOrder = levelNodes.map(node => {
          let targetPos = 0;
          let count = 0;
          
          // 如果有子节点，基于子节点的平均位置
          if (node.children && node.children.length > 0) {
            node.children.forEach(childId => {
              if (nodePositions.has(childId)) {
                targetPos += nodePositions.get(childId);
                count++;
              }
            });
          }
          
          // 如果没有子节点（叶子节点），或者还没有位置信息，使用默认排序
          if (count === 0) {
            // 查找父节点
            visibleNodes.forEach(potentialParent => {
              if (potentialParent.children && potentialParent.children.includes(node.id)) {
                if (nodePositions.has(potentialParent.id)) {
                  targetPos += nodePositions.get(potentialParent.id);
                  count++;
                }
              }
            });
          }
          
          const avgPos = count > 0 ? targetPos / count : 0;
          
          return {
            node,
            targetPos: avgPos,
            // 备用排序键：Fragment > Pipeline > Plan Node ID
            sortKey: `${node.fragment_id || 'F0'}-${node.pipeline_id || 'P0'}-${node.plan_node_id || 0}`
          };
        });
        
        // 按目标位置排序，如果目标位置相同则按备用键排序
        nodeOrder.sort((a, b) => {
          if (Math.abs(a.targetPos - b.targetPos) > 0.1) {
            return a.targetPos - b.targetPos;
          }
          return a.sortKey.localeCompare(b.sortKey);
        });
        
        // 分配实际位置
        nodeOrder.forEach((item, index) => {
          const x = index * LEVEL_WIDTH;
          nodePositions.set(item.node.id, x);
        });
        
        // 更新层级节点列表（按新顺序）
        visibleNodesByDepth.set(depth, nodeOrder.map(item => item.node));
      });
      
      // 第二步：计算节点布局宽度
      let maxNodesInLevel = 0;
      visibleNodesByDepth.forEach(nodes => {
        maxNodesInLevel = Math.max(maxNodesInLevel, nodes.length);
      });
      const totalWidth = maxNodesInLevel * LEVEL_WIDTH;
      
      // SVG 宽度始终等于容器宽度（不超出容器）
      // 如果内容宽度大于容器，用户可以通过缩放和平移查看
      this.svgWidth = containerWidth;
      const offsetX = Math.max(50, (this.svgWidth - totalWidth) / 2);

      // 第三步：生成最终节点位置
      this.renderedNodes = visibleNodes.map(node => {
        const depth = node.depth || 0;
        const y = depth * LEVEL_HEIGHT + 80;
        const x = nodePositions.get(node.id) + offsetX;
        return { ...node, x, y };
      });

      // 步骤4: 构建连接线
      this.links = [];
      const renderedNodeMap = new Map(this.renderedNodes.map(n => [n.id, n]));
      
      visibleNodes.forEach(sourceNode => {
        if (!sourceNode.children) return;
        sourceNode.children.forEach((childId, idx) => {
          const targetNode = renderedNodeMap.get(childId);
          if (!targetNode) return;
          
          const source = renderedNodeMap.get(sourceNode.id);
          if (!source) return;
          
          if (source.id !== targetNode.id) {
            const startX = targetNode.x + this.NODE_WIDTH / 2;
            const startY = targetNode.y;
            const endX = source.x + this.NODE_WIDTH / 2;
            const endY = source.y + this.NODE_HEIGHT + 8;
            const controlY = (startY + endY) / 2;
            const path = `M ${startX} ${startY} C ${startX} ${controlY}, ${endX} ${controlY}, ${endX} ${endY}`;
            
            // 获取原始节点的行数（用于显示）
            const originalChild = nodeMap.get(childId) || targetNode;
            const rows = this.getNodeRows(originalChild);
            let label = `Rows: ${this.formatRowsSimple(rows)}`;
            
            if (sourceNode.operator_name && sourceNode.operator_name.includes('JOIN')) {
              label += idx === 0 ? ' (PROBE)' : ' (BUILD)';
            }
            
            this.links.push({
              id: `${source.id}-${targetNode.id}`,
              path,
              labelX: (startX + endX) / 2,
              labelY: controlY - 8,
              label,
              isHotspot: source.is_hotspot || targetNode.is_hotspot,
              strokeWidth: Math.min(5, Math.max(1, Math.log10(rows + 1) / 2))
            });
          }
        });
      });
    },
    
    // 创建合并节点
    createMergedNode(primaryNode, secondaryNode, type) {
      // primaryNode: EXCHANGE/AGGREGATION 等主节点
      // secondaryNode: 对应的 SINK 节点
      
      // 合并 children
      const mergedChildren = [...(primaryNode.children || [])];
      if (secondaryNode.children) {
        secondaryNode.children.forEach(childId => {
          if (!mergedChildren.includes(childId)) {
            mergedChildren.push(childId);
          }
        });
      }
      
      // 合并时间
      const primaryTime = this.getNodeTime(primaryNode);
      const secondaryTime = this.getNodeTime(secondaryNode);
      const totalTime = primaryTime + secondaryTime;
      
      // 合并百分比
      const totalPct = (primaryNode.time_percentage || 0) + (secondaryNode.time_percentage || 0);
      
      return {
        ...primaryNode, // 保留主节点的基本信息
        id: primaryNode.id,
        operator_name: type, // 使用简化的名称
        children: mergedChildren,
        isMerged: true,
        mergedType: type,
        primaryNode: primaryNode,
        secondaryNode: secondaryNode,
        time_percentage: totalPct,
        is_hotspot: primaryNode.is_hotspot || secondaryNode.is_hotspot,
        metrics: {
          ...primaryNode.metrics,
          operator_total_time: totalTime
        }
      };
    },
    getNodeTime(node) {
      if (!node?.metrics) return 0;
      const time = node.metrics.operator_total_time;
      if (typeof time === 'number') return time;
      if (typeof time === 'object' && time !== null) {
        return (time.secs || 0) * 1_000_000_000 + (time.nanos || 0);
      }
      return 0;
    },
    getNodeRows(node) {
      return node?.metrics?.rows_returned || 0;
    },
    formatRowsSimple(rows) {
      if (rows === 0) return '0';
      if (rows < 1000) return String(rows);
      if (rows < 1_000_000) return `${(rows / 1000).toFixed(1)}K`;
      if (rows < 1_000_000_000) return `${(rows / 1_000_000).toFixed(1)}M`;
      return `${(rows / 1_000_000_000).toFixed(1)}B`;
    },
    formatOperatorName(name) {
      if (!name) return 'UNKNOWN';
      return name.replace(/_OPERATOR$/, '').replace(/_/g, ' ');
    },
    formatGraphTime(node) {
      if (!node?.metrics) return 'N/A';
      const time = this.getNodeTime(node);
      if (time === 0) return '0ns';
      const us = time / 1000;
      if (us < 1000) return `${us.toFixed(2)}us`;
      const ms = us / 1000;
      if (ms < 1000) return `${ms.toFixed(2)}ms`;
      return `${(ms / 1000).toFixed(2)}s`;
    },
    getNodeColorClass(node) {
      if (!node) return 'default';
      const name = node.operator_name || '';
      if (name.includes('SCAN')) return 'scan';
      if (name.includes('JOIN')) return 'join';
      if (name.includes('AGGREGATE') || name.includes('AGGREGATION')) return 'aggregate';
      if (name.includes('EXCHANGE')) return 'exchange';
      if (name.includes('SORT')) return 'sort';
      if (name.includes('PROJECT')) return 'project';
      return 'default';
    },
    getProgressWidth(node) {
      if (!node?.time_percentage) return 0;
      return (Math.min(100, node.time_percentage) / 100) * this.NODE_WIDTH;
    },
    getProgressColor(node) {
      const pct = node?.time_percentage || 0;
      if (pct > 30) return '#E57373';
      if (pct > 10) return '#FFB74D';
      return '#81C784';
    },
    selectNode(node) {
      this.selectedNodeId = node.id;
      this.selectedNode = node;
    },
    deselectNode() {
      this.selectedNodeId = null;
      this.selectedNode = null;
    },
    zoomIn() { 
      this.zoomAtPoint(this.svgWidth / 2, this.svgHeight / 2, 1.2);
    },
    zoomOut() { 
      this.zoomAtPoint(this.svgWidth / 2, this.svgHeight / 2, 1 / 1.2);
    },
    // 在指定点进行缩放
    zoomAtPoint(pointX, pointY, factor) {
      // 计算缩放前，该点在内容坐标系中的位置
      const beforeZoomX = (pointX - this.panX) / this.zoom;
      const beforeZoomY = (pointY - this.panY) / this.zoom;
      
      // 应用缩放
      const newZoom = Math.min(3, Math.max(0.3, this.zoom * factor));
      this.zoom = newZoom;
      
      // 调整 pan 偏移，使该点保持不变
      this.panX = pointX - beforeZoomX * newZoom;
      this.panY = pointY - beforeZoomY * newZoom;
    },
    fitToScreen() { this.zoom = 0.8; this.panX = 50; this.panY = 50; },
    resetView() { this.zoom = 1; this.panX = 50; this.panY = 50; this.deselectNode(); },
    handleWheel(event) {
      // 获取 SVG 元素和鼠标位置
      const svg = this.$refs.svgCanvas;
      if (!svg) return;
      
      const rect = svg.getBoundingClientRect();
      // 鼠标在 SVG 中的位置
      const mouseX = event.clientX - rect.left;
      const mouseY = event.clientY - rect.top;
      
      // 计算缩放前，鼠标点在内容坐标系中的位置
      const beforeZoomX = (mouseX - this.panX) / this.zoom;
      const beforeZoomY = (mouseY - this.panY) / this.zoom;
      
      // 应用缩放
      const delta = event.deltaY;
      const zoomSensitivity = 0.001; // 降低敏感度
      const zoomChange = -delta * zoomSensitivity;
      const oldZoom = this.zoom;
      const newZoom = Math.min(3, Math.max(0.3, this.zoom * (1 + zoomChange)));
      this.zoom = newZoom;
      
      // 计算缩放后，为了保持鼠标点指向的内容不变，需要调整的 pan 偏移
      // 新的 pan 位置 = 鼠标位置 - (内容坐标 * 新缩放比例)
      this.panX = mouseX - beforeZoomX * newZoom;
      this.panY = mouseY - beforeZoomY * newZoom;
    },
    startPan(event) {
      this.isPanning = true;
      this.panStartX = event.clientX - this.panX;
      this.panStartY = event.clientY - this.panY;
    },
    doPan(event) {
      if (!this.isPanning) return;
      this.panX = event.clientX - this.panStartX;
      this.panY = event.clientY - this.panStartY;
    },
    endPan() { this.isPanning = false; },
    
    // Common methods
    formatPct(pct) {
      if (pct === null || pct === undefined) return '0.00%';
      return `${pct.toFixed(2)}%`;
    },
    formatNumber(num) {
      if (!num) return '0';
      return num.toLocaleString();
    },
    getShortId(id) {
      if (!id) return '-';
      return id.replace('Fragment ', 'F').replace('Pipeline ', 'P');
    },
    getNodeColor(node) {
      if (!node) return '#999';
      if (node.is_hotspot) return '#e74c3c';
      if (node.time_percentage > 15) return '#f39c12';
      const name = node.operator_name || '';
      if (name.includes('SCAN')) return '#9b59b6';
      if (name.includes('JOIN')) return '#e67e22';
      if (name.includes('AGGREGATE') || name.includes('AGGREGATION')) return '#1abc9c';
      if (name.includes('EXCHANGE')) return '#3498db';
      return '#95a5a6';
    },
    getPctClass(node) {
      if (!node.time_percentage) return '';
      if (node.time_percentage > 30) return 'high-pct';
      if (node.time_percentage > 15) return 'medium-pct';
      return '';
    },
    getFragmentStats(fragId) {
      const nodes = this.nodesByFragment[fragId] || [];
      return `${nodes.length} operators`;
    },
    getPipelineIds(fragId) {
      const nodes = this.nodesByFragment[fragId] || [];
      const ids = new Set(nodes.map(n => n.pipeline_id).filter(Boolean));
      return Array.from(ids).sort((a, b) => {
        const numA = parseInt(a.replace('Pipeline ', ''));
        const numB = parseInt(b.replace('Pipeline ', ''));
        return numA - numB;
      });
    },
    getNodesForPipeline(fragId, pipeId) {
      return (this.nodesByFragment[fragId] || []).filter(n => n.pipeline_id === pipeId);
    }
  }
};
</script>

<style scoped lang="scss">
.dag-visualization {
  width: 100%;
  height: 100%;
  min-height: 500px;
  background: white;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.view-controls {
  display: flex;
  gap: 10px;
  padding: 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
  align-items: center;
  flex-shrink: 0;
}

.view-btn {
  padding: 8px 16px;
  border: 1px solid #d0d0d0;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  
  &:hover {
    background: #f5f5f5;
  }
  
  &.active {
    background: #2196F3;
    color: white;
    border-color: #2196F3;
  }
}

.node-count {
  margin-left: auto;
  color: #666;
  font-size: 13px;
}

.no-data {
  padding: 80px 20px;
  text-align: center;
  color: #999;
}

/* Graph View Styles */
.graph-view-container {
  position: relative;
  flex: 1;
  min-height: 500px;
  background: white;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dag-toolbar {
  position: absolute;
  top: 20px;
  right: 20px;
  z-index: 100;
  display: flex;
  gap: 8px;
  background: white;
  padding: 8px;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.toolbar-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: white;
  border-radius: 4px;
  cursor: pointer;
  color: #666;
  transition: all 0.2s;
  
  &:hover {
    background: #f5f5f5;
    color: #333;
  }
}

.svg-wrapper {
  flex: 1;
  width: 100%;
  max-width: 100%; // 确保不超出父容器
  overflow: hidden;
  position: relative;
}

.dag-svg {
  display: block;
  user-select: none;
  max-width: 100%; // 确保 SVG 不超出容器宽度
}

.connection-line {
  fill: none;
  stroke: #BDBDBD;
  transition: stroke 0.3s;
  
  &.hotspot-link {
    stroke: #E57373;
  }
}

.link-label {
  font-size: 11px;
  fill: #666;
  text-anchor: middle;
  pointer-events: none;
}

.node-group {
  cursor: pointer;
  transition: all 0.2s;
  
  &:hover .node-border { stroke: #2196F3; stroke-width: 2; }
  &.selected .node-border { stroke: #1976D2; stroke-width: 2; }
  &.hotspot .node-border { stroke: #E57373; stroke-width: 2; }
}

.node-header {
  fill: #757575;
  &.header-scan { fill: #FF9800; }
  &.header-join { fill: #FF9800; }
  &.header-aggregate { fill: #9C27B0; }
  &.header-exchange { fill: #607D8B; }
  &.header-sort { fill: #00BCD4; }
  &.header-project { fill: #9E9E9E; }
}

.node-body { fill: white; }
.progress-bg { fill: #F5F5F5; }
.node-border { fill: none; stroke: #E0E0E0; stroke-width: 1; }
.node-title { font-size: 13px; font-weight: 600; fill: white; pointer-events: none; }
.node-detail { font-size: 11px; fill: #666; pointer-events: none; }
.node-detail-small { font-size: 10px; fill: #999; pointer-events: none; }
.node-percentage { font-size: 12px; font-weight: 600; fill: #333; pointer-events: none; }
.merged-badge { fill: #FFD700; font-size: 12px; }

.detail-panel {
  position: absolute;
  right: 0;
  top: 0;
  width: 320px;
  height: 100%;
  background: white;
  border-left: 1px solid #e0e0e0;
  overflow-y: auto;
  box-shadow: -2px 0 8px rgba(0,0,0,0.1);
}

.detail-header {
  padding: 20px;
  background: #f5f5f5;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  
  h3 { margin: 0; font-size: 14px; }
  .close-btn { background: none; border: none; cursor: pointer; font-size: 16px; color: #666; }
}

.detail-content { padding: 20px; }
.detail-section { margin-bottom: 20px; h4 { margin: 0 0 10px; font-size: 12px; color: #666; text-transform: uppercase; } }

.merged-indicator {
  background: #fff8dc;
  padding: 12px;
  border-radius: 6px;
  border-left: 3px solid #FFD700;
  
  h4 { color: #ff8c00; }
}

.sub-node-section {
  background: #f8f9fa;
  padding: 12px;
  border-radius: 6px;
  border-left: 3px solid #2196F3;
  
  h4 { 
    color: #2196F3; 
    font-size: 11px;
    text-transform: none;
  }
}

.detail-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #f5f5f5;
  font-size: 12px;
  
  .label { color: #666; }
  .value { color: #333; font-weight: 500; }
}

.slide-enter-active, .slide-leave-active { transition: transform 0.3s ease; }
.slide-enter-from, .slide-leave-to { transform: translateX(100%); }

/* Table View Styles */
.table-view {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.table-header, .table-row {
  display: grid;
  grid-template-columns: 80px 80px 2fr 120px 80px 100px;
  gap: 12px;
  padding: 12px;
  font-size: 13px;
}

.table-header {
  font-weight: 600;
  color: #666;
  border-bottom: 2px solid #e0e0e0;
  position: sticky;
  top: 0;
  background: white;
  z-index: 10;
}

.table-row {
  border-bottom: 1px solid #f5f5f5;
  align-items: center;
  transition: background 0.2s;
  
  &:hover { background: #f8f9fa; }
  &.hotspot { background: #ffebee; }
}

.operator-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
  margin-right: 8px;
}

.col-pct {
  &.high-pct { color: #e74c3c; font-weight: 600; }
  &.medium-pct { color: #f39c12; font-weight: 600; }
}

/* Tree View Styles */
.tree-view {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.tree-legend {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 6px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #666;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  display: inline-block;
}

.fragment-node {
  margin-bottom: 8px;
}

.fragment-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px;
  background: #f8f9fa;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
  
  &:hover { background: #e9ecef; }
}

.toggle-icon {
  font-size: 10px;
  color: #666;
}

.fragment-icon {
  font-size: 16px;
}

.fragment-name {
  font-weight: 600;
  font-size: 14px;
  color: #333;
}

.fragment-node-count {
  margin-left: auto;
  font-size: 12px;
  color: #999;
}

.fragment-nodes {
  margin: 8px 0 8px 32px;
}

.node-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin: 4px 0;
  border-radius: 4px;
  background: white;
  border: 1px solid #e0e0e0;
  
  &.hotspot {
    background: #ffebee;
    border-color: #e74c3c;
  }
}

.node-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.node-op-name {
  flex: 1;
  font-size: 13px;
  color: #333;
}

.node-pct {
  font-size: 12px;
  color: #e74c3c;
  font-weight: 600;
}

/* Pipeline View Styles */
.pipeline-view {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.fragment-section {
  margin-bottom: 30px;
}

.fragment-title {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: #2c3e50;
  color: white;
  border-radius: 6px;
  font-weight: 600;
  font-size: 14px;
}

.frag-icon {
  font-size: 18px;
}

.frag-stats {
  margin-left: auto;
  font-size: 12px;
  opacity: 0.8;
}

.pipelines-container {
  margin-top: 10px;
  margin-left: 20px;
}

.pipeline-section {
  margin-bottom: 16px;
}

.pipeline-title {
  font-weight: 600;
  font-size: 13px;
  color: #34495e;
  padding: 8px 12px;
  background: #ecf0f1;
  border-radius: 4px;
}

.operators-list {
  margin-top: 8px;
  margin-left: 20px;
}

.operator-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  margin: 4px 0;
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 12px;
  
  &.hotspot {
    background: #ffebee;
    border-color: #e74c3c;
  }
}

.op-name {
  flex: 1;
  color: #333;
}

.op-pct {
  color: #e74c3c;
  font-weight: 600;
  min-width: 50px;
  text-align: right;
}

.op-time {
  color: #999;
  min-width: 80px;
  text-align: right;
}
</style>
