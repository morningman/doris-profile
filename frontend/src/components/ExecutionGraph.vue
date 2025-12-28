<template>
  <div class="execution-graph">
    <!-- View Mode Toggle -->
    <div class="view-controls">
      <!-- 第一排：视图切换和统计 -->
      <div class="controls-row-top">
      <div class="view-tabs">
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
          :class="['view-btn', { active: viewMode === 'pipeline' }]"
          @click="viewMode = 'pipeline'"
        >
          📦 Pipeline View
        </button>
        <span class="node-count">{{ nodeCount }} nodes | {{ fragmentCount }} fragments</span>
      </div>
      
      <!-- 工具栏按钮 -->
      <div v-if="viewMode === 'graph'" class="view-toolbar">
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
      </div>
      
      <!-- 第二排：搜索栏 -->
      <div v-if="viewMode === 'graph'" class="controls-row-search">
        <div class="search-box">
          <input 
            v-model="searchText" 
            @input="handleSearch"
            @keydown.enter="searchNext"
            type="text" 
            placeholder="Search nodes (e.g., HASH_JOIN, F8-P9)"
            class="search-input"
          />
          <span v-if="searchResults.length > 0" class="search-count">
            {{ currentSearchIndex + 1 }} / {{ searchResults.length }}
          </span>
        </div>
        
        <!-- 搜索导航按钮 -->
        <button 
          @click="searchPrev" 
          :disabled="searchResults.length === 0"
          class="toolbar-btn" 
          title="上一个匹配"
        >
          <i class="fas fa-chevron-up"></i>
        </button>
        <button 
          @click="searchNext" 
          :disabled="searchResults.length === 0"
          class="toolbar-btn" 
          title="下一个匹配"
        >
          <i class="fas fa-chevron-down"></i>
        </button>
      </div>
      
      <!-- 第三排：Pipeline 过滤器 -->
      <div v-if="viewMode === 'graph'" class="controls-row-filter">
        <label>Fragment:</label>
        <el-select 
          v-model="selectedFragmentId" 
          @change="onFragmentChange"
          placeholder="Select Fragment"
          clearable
          size="small"
          style="width: 150px"
        >
          <el-option 
            v-for="fragId in fragmentIds" 
            :key="fragId" 
            :label="fragId" 
            :value="fragId"
          />
        </el-select>
        
        <label>Pipelines:</label>
        <el-select 
          v-model="selectedPipelineIds" 
          @change="onPipelineSelectionChange"
          placeholder="Select Pipelines"
          multiple
          collapse-tags
          size="small"
          style="width: 250px"
          :disabled="!selectedFragmentId"
        >
          <el-option 
            v-for="pipeId in availablePipelines" 
            :key="pipeId" 
            :label="pipeId" 
            :value="pipeId"
          >
            <span 
              class="pipeline-color-dot" 
              :style="{ background: pipelineColorMap[pipeId] || '#ccc' }"
            ></span>
            {{ pipeId }}
          </el-option>
        </el-select>
        
        <button 
          v-if="selectedFragmentId"
          @click="selectAllPipelines" 
          class="toolbar-btn"
          title="Select All Pipelines"
        >
          Select All
        </button>
        
        <button 
          v-if="hasActivePipelineFilter"
          @click="clearPipelineFilter" 
          class="toolbar-btn"
          title="Clear Filter"
        >
          <i class="fas fa-times"></i> Clear
        </button>
      </div>
    </div>

    <div v-if="!hasNodes" class="no-data">
      <p>No execution tree data available</p>
    </div>
    
    <!-- Graph View (SVG DAG Visualization) -->
    <div v-else-if="viewMode === 'graph'" class="graph-view-container">
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
            <marker id="arrow" markerWidth="8" markerHeight="8" refX="5" refY="4" orient="auto">
              <polygon points="0 0, 6 4, 0 8" fill="#616161" />
            </marker>
            <marker id="arrow-hotspot" markerWidth="8" markerHeight="8" refX="5" refY="4" orient="auto">
              <polygon points="0 0, 6 4, 0 8" fill="#E57373" />
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
                :data-node-id="node.id"
                class="node-group"
                :class="{ 
                  selected: selectedNodeId === node.id, 
                  hotspot: node.is_hotspot,
                  'top-time-consuming': isTopThreeNode(node.id),
                  'pipeline-highlight': hasActivePipelineFilter && node.pipeline_id && node.fragment_id === selectedFragmentId && selectedPipelineIds.includes(node.pipeline_id),
                  'pipeline-dimmed': hasActivePipelineFilter && (!node.pipeline_id || node.fragment_id !== selectedFragmentId || !selectedPipelineIds.includes(node.pipeline_id))
                }"
                @click.stop="selectNode(node)"
              >
                <rect 
                  class="node-header" 
                  :class="hasActivePipelineFilter ? '' : `header-${getNodeColorClass(node)}`" 
                  :fill="hasActivePipelineFilter ? getNodeColor(node) : ''"
                  :width="NODE_WIDTH" 
                  :height="getNodeHeaderHeight(node)" 
                  rx="3" 
                />
                <rect class="node-body" :width="NODE_WIDTH" :y="getNodeHeaderHeight(node)" :height="NODE_BODY_HEIGHT" />
                <rect class="progress-bg" :y="getNodeHeaderHeight(node) + NODE_BODY_HEIGHT" :width="NODE_WIDTH" :height="NODE_PROGRESS_HEIGHT" />
                <rect v-if="node.time_percentage" class="progress-fill" :y="getNodeHeaderHeight(node) + NODE_BODY_HEIGHT" :width="getProgressWidth(node)" :height="NODE_PROGRESS_HEIGHT" :fill="getProgressColor(node)" />
                <rect class="node-border" :width="NODE_WIDTH" :height="getNodeTotalHeight(node)" rx="3" />
                
                <!-- 节点标题（包含 ID 信息） -->
                <text class="node-title" x="10" :y="19">
                  {{ formatNodeTitle(node) }}
                  <tspan v-if="node.isMerged" class="merged-badge" dx="5" style="font-size: 11px; fill: #FFD700;">⚡</tspan>
                </text>
                
                <!-- JOIN 类型信息 (仅 HASH_JOIN 节点，显示在标题栏内) -->
                <text v-if="getJoinType(node)" class="node-join-type" x="10" :y="33" style="font-size: 9px; fill: rgba(255, 255, 255, 0.9); font-weight: 500;">
                  {{ getJoinType(node) }}
                </text>
                
                <!-- TABLE 名称信息 (仅 SCAN 节点，显示在标题栏内) -->
                <text v-if="getTableName(node)" class="node-table-name" x="10" :y="33" style="font-size: 9px; fill: rgba(255, 255, 255, 0.9); font-weight: 500;">
                  Table: {{ getTableName(node) }}
                </text>
                
                <!-- 节点详情 -->
                <template v-if="node.isMerged">
                  <!-- 合并节点显示两个节点的简化信息 -->
                  <text class="node-detail-small" x="10" :y="getNodeHeaderHeight(node) + 12" style="font-size: 10px;">
                    {{ node.primaryNode.operator_name }}
                  </text>
                  <text class="node-detail-small" x="10" :y="getNodeHeaderHeight(node) + 24" style="font-size: 10px;">
                    + {{ node.secondaryNode.operator_name }}
                  </text>
                  <text class="node-detail" x="10" :y="getNodeHeaderHeight(node) + 40">
                    Total Cost: {{ formatGraphTime(node) }}
                  </text>
                  <text class="node-percentage" :x="NODE_WIDTH - 10" :y="getNodeHeaderHeight(node) + 40" text-anchor="end">
                    {{ formatPct(node.time_percentage) }}
                  </text>
                </template>
                <template v-else>
                  <!-- 普通节点 -->
                  <text class="node-detail" x="10" :y="getNodeHeaderHeight(node) + 20">
                    Cost: {{ formatGraphTime(node) }}
                  </text>
                  <text class="node-percentage" :x="NODE_WIDTH - 10" :y="getNodeHeaderHeight(node) + 20" text-anchor="end">
                    {{ formatPct(node.time_percentage) }}
                  </text>
                  <!-- Max/Min 时间 -->
                  <text v-if="node.metrics?.operator_max_time_raw" class="node-detail-small" x="10" :y="getNodeHeaderHeight(node) + 35" style="font-size: 9px; fill: #666;">
                    max: {{ node.metrics.operator_max_time_raw }} | min: {{ node.metrics.operator_min_time_raw || 'N/A' }}
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
                  <span class="label">Plan Node ID:</span>
                  <span class="value">{{ selectedNode.primaryNode.plan_node_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Fragment:</span>
                  <span class="value">{{ selectedNode.primaryNode.fragment_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Pipeline:</span>
                  <span class="value">{{ selectedNode.primaryNode.pipeline_id }}</span>
                </div>
                
                <!-- PlanInfo -->
                <div v-if="selectedNode.primaryNode.plan_info && selectedNode.primaryNode.plan_info.length > 0" style="margin-top: 12px;">
                  <div style="font-weight: 600; margin-bottom: 6px; color: #555;">📋 PlanInfo</div>
                  <div v-for="(item, index) in selectedNode.primaryNode.plan_info" :key="index" class="metric-item">
                    <div class="metric-label">{{ item.key }}</div>
                    <div class="metric-value">{{ item.value }}</div>
                    <div v-if="item.children && item.children.length > 0" class="metric-children">
                      <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                        <div class="metric-label">{{ child.key }}</div>
                        <div class="metric-value">{{ child.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- CommonCounters -->
                <div v-if="selectedNode.primaryNode.common_counters && selectedNode.primaryNode.common_counters.length > 0" style="margin-top: 12px;">
                  <div style="font-weight: 600; margin-bottom: 6px; color: #555;">📊 Common Counters</div>
                  <div v-for="(item, index) in selectedNode.primaryNode.common_counters" :key="index" class="metric-item">
                    <div class="metric-label">{{ item.key }}</div>
                    <div class="metric-value">{{ item.value }}</div>
                    <div v-if="item.children && item.children.length > 0" class="metric-children">
                      <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                        <div class="metric-label">{{ child.key }}</div>
                        <div class="metric-value">{{ child.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- CustomCounters -->
                <div v-if="selectedNode.primaryNode.custom_counters && selectedNode.primaryNode.custom_counters.length > 0" style="margin-top: 12px;">
                  <div style="font-weight: 600; margin-bottom: 6px; color: #555;">⚙️ Custom Counters</div>
                  <div v-for="(item, index) in selectedNode.primaryNode.custom_counters" :key="index" class="metric-item">
                    <div class="metric-label">{{ item.key }}</div>
                    <div class="metric-value">{{ item.value }}</div>
                    <div v-if="item.children && item.children.length > 0" class="metric-children">
                      <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                        <div class="metric-label">{{ child.key }}</div>
                        <div class="metric-value">{{ child.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 次节点信息 (SINK) -->
              <div class="detail-section sub-node-section">
                <h4>🔹 {{ selectedNode.secondaryNode.operator_name }}</h4>
                <div class="detail-item">
                  <span class="label">Plan Node ID:</span>
                  <span class="value">{{ selectedNode.secondaryNode.plan_node_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Fragment:</span>
                  <span class="value">{{ selectedNode.secondaryNode.fragment_id }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Pipeline:</span>
                  <span class="value">{{ selectedNode.secondaryNode.pipeline_id }}</span>
                </div>
                
                <!-- PlanInfo -->
                <div v-if="selectedNode.secondaryNode.plan_info && selectedNode.secondaryNode.plan_info.length > 0" style="margin-top: 12px;">
                  <div style="font-weight: 600; margin-bottom: 6px; color: #555;">📋 PlanInfo</div>
                  <div v-for="(item, index) in selectedNode.secondaryNode.plan_info" :key="index" class="metric-item">
                    <div class="metric-label">{{ item.key }}</div>
                    <div class="metric-value">{{ item.value }}</div>
                    <div v-if="item.children && item.children.length > 0" class="metric-children">
                      <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                        <div class="metric-label">{{ child.key }}</div>
                        <div class="metric-value">{{ child.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- CommonCounters -->
                <div v-if="selectedNode.secondaryNode.common_counters && selectedNode.secondaryNode.common_counters.length > 0" style="margin-top: 12px;">
                  <div style="font-weight: 600; margin-bottom: 6px; color: #555;">📊 Common Counters</div>
                  <div v-for="(item, index) in selectedNode.secondaryNode.common_counters" :key="index" class="metric-item">
                    <div class="metric-label">{{ item.key }}</div>
                    <div class="metric-value">{{ item.value }}</div>
                    <div v-if="item.children && item.children.length > 0" class="metric-children">
                      <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                        <div class="metric-label">{{ child.key }}</div>
                        <div class="metric-value">{{ child.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- CustomCounters -->
                <div v-if="selectedNode.secondaryNode.custom_counters && selectedNode.secondaryNode.custom_counters.length > 0" style="margin-top: 12px;">
                  <div style="font-weight: 600; margin-bottom: 6px; color: #555;">⚙️ Custom Counters</div>
                  <div v-for="(item, index) in selectedNode.secondaryNode.custom_counters" :key="index" class="metric-item">
                    <div class="metric-label">{{ item.key }}</div>
                    <div class="metric-value">{{ item.value }}</div>
                    <div v-if="item.children && item.children.length > 0" class="metric-children">
                      <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                        <div class="metric-label">{{ child.key }}</div>
                        <div class="metric-value">{{ child.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 普通节点显示 -->
            <div v-else>
              <div class="detail-section">
                <h4>基本信息</h4>
                <div class="detail-item"><span class="label">Operator:</span><span class="value">{{ selectedNode.operator_name }}</span></div>
                <div class="detail-item"><span class="label">Plan Node ID:</span><span class="value">{{ selectedNode.plan_node_id }}</span></div>
                <div class="detail-item"><span class="label">Fragment:</span><span class="value">{{ selectedNode.fragment_id }}</span></div>
                <div class="detail-item"><span class="label">Pipeline:</span><span class="value">{{ selectedNode.pipeline_id }}</span></div>
              </div>
              
              <!-- PlanInfo -->
              <div v-if="selectedNode.plan_info && selectedNode.plan_info.length > 0" class="detail-section">
                <h4>📋 PlanInfo</h4>
                <div v-for="(item, index) in selectedNode.plan_info" :key="index" class="metric-item">
                  <div class="metric-label">{{ item.key }}</div>
                  <div class="metric-value">{{ item.value }}</div>
                  <div v-if="item.children && item.children.length > 0" class="metric-children">
                    <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                      <div class="metric-label">{{ child.key }}</div>
                      <div class="metric-value">{{ child.value }}</div>
                    </div>
                  </div>
                </div>
              </div>
              
              <!-- CommonCounters -->
              <div v-if="selectedNode.common_counters && selectedNode.common_counters.length > 0" class="detail-section">
                <h4>📊 Common Counters</h4>
                <div v-for="(item, index) in selectedNode.common_counters" :key="index" class="metric-item">
                  <div class="metric-label">{{ item.key }}</div>
                  <div class="metric-value">{{ item.value }}</div>
                  <div v-if="item.children && item.children.length > 0" class="metric-children">
                    <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                      <div class="metric-label">{{ child.key }}</div>
                      <div class="metric-value">{{ child.value }}</div>
                    </div>
                  </div>
                </div>
              </div>
              
              <!-- CustomCounters -->
              <div v-if="selectedNode.custom_counters && selectedNode.custom_counters.length > 0" class="detail-section">
                <h4>⚙️ Custom Counters</h4>
                <div v-for="(item, index) in selectedNode.custom_counters" :key="index" class="metric-item">
                  <div class="metric-label">{{ item.key }}</div>
                  <div class="metric-value">{{ item.value }}</div>
                  <div v-if="item.children && item.children.length > 0" class="metric-children">
                    <div v-for="(child, childIndex) in item.children" :key="childIndex" class="metric-child">
                      <div class="metric-label">{{ child.key }}</div>
                      <div class="metric-value">{{ child.value }}</div>
                    </div>
                  </div>
                </div>
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
        <span class="col-nodeid">NodeId</span>
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
          <span class="col-nodeid">{{ node.plan_node_id !== undefined && node.plan_node_id !== null ? node.plan_node_id : '-' }}</span>
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

    <!-- Pipeline View -->
    <div v-else-if="viewMode === 'pipeline'" class="pipeline-view">
      <div v-for="fragId in fragmentIds" :key="fragId" class="fragment-section">
        <div class="fragment-title" @click="toggleFragmentCollapse(fragId)">
          <span class="collapse-icon">{{ isFragmentCollapsed(fragId) ? '▶' : '▼' }}</span>
          <span class="frag-icon">📁</span> {{ fragId }}
          <span class="frag-stats">{{ getFragmentStats(fragId) }}</span>
        </div>
        <div v-show="!isFragmentCollapsed(fragId)" class="pipelines-container">
          <div v-for="pipeId in getPipelineIds(fragId)" :key="pipeId" class="pipeline-section">
            <div class="pipeline-title" @click="togglePipelineCollapse(fragId, pipeId)">
              <span class="collapse-icon">{{ isPipelineCollapsed(fragId, pipeId) ? '▶' : '▼' }}</span>
              <span class="pipe-icon">📦</span> {{ pipeId }}
            </div>
            <div v-show="!isPipelineCollapsed(fragId, pipeId)" class="operators-list">
              <div 
                v-for="node in getNodesForPipeline(fragId, pipeId)" 
                :key="node.id"
                :class="['operator-item', { hotspot: node.is_hotspot }]"
              >
                <span class="operator-dot" :style="{ background: getNodeColor(node) }"></span>
                <span class="op-name">
                  {{ node.operator_name }}
                  <span v-if="node.plan_node_id !== undefined && node.plan_node_id !== null" class="op-node-id">({{ node.plan_node_id }})</span>
                </span>
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
export default {
  name: 'ExecutionGraph',
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
      NODE_BODY_HEIGHT: 68,  // 从56增加到68，以容纳max/min行
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
      needsAutoFit: true, // 标记是否需要自动适应
      topThreeNodeIds: [], // 存储最耗时的三个节点 ID
      
      // 搜索相关状态
      searchText: '',
      searchResults: [],
      currentSearchIndex: -1,
      
      // Pipeline 过滤器状态
      selectedFragmentId: null,        // 当前选中的 Fragment ID
      selectedPipelineIds: [],         // 当前选中的多个 Pipeline IDs
      pipelineColorMap: {},            // Pipeline ID -> 颜色的映射
      
      // Pipeline View 折叠状态
      collapsedFragments: new Set(),   // 折叠的 Fragment IDs
      collapsedPipelines: new Set(),   // 折叠的 Pipeline IDs (格式: "fragId|pipeId")
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
    // 获取选中 fragment 下的 pipeline 列表（排序）
    availablePipelines() {
      if (!this.selectedFragmentId) return [];
      const nodes = this.nodesByFragment[this.selectedFragmentId] || [];
      const pipelineIds = new Set(nodes.map(n => n.pipeline_id).filter(Boolean));
      return Array.from(pipelineIds).sort((a, b) => {
        const numA = parseInt(a.replace(/\D/g, ''));
        const numB = parseInt(b.replace(/\D/g, ''));
        return numA - numB;
      });
    },
    // 检查是否有 pipeline 过滤器激活
    hasActivePipelineFilter() {
      return this.selectedPipelineIds.length > 0;
    },
  },
  watch: {
    tree: {
      handler() {
        if (this.viewMode === 'graph') {
          this.needsAutoFit = true; // 数据变化时需要自动适应
          this.$nextTick(() => this.renderDAG());
        }
        // 初始化折叠状态：默认全部折叠
        this.initializeCollapseState();
      },
      deep: true,
      immediate: true
    },
    viewMode(newMode) {
      if (newMode === 'graph') {
        this.needsAutoFit = true; // 切换视图时需要自动适应
        this.$nextTick(() => this.renderDAG());
      } else if (newMode === 'pipeline') {
        // 切换到 Pipeline View 时初始化折叠状态
        this.initializeCollapseState();
      }
    }
  },
  mounted() {
    if (this.viewMode === 'graph') {
      // 延迟一点确保容器尺寸已准备好
      this.$nextTick(() => {
        this.updateSvgSize();
        this.renderDAG();
      });
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
      
      // 计算最耗时的三个节点（基于 time_percentage）
      const sortedByTime = [...visibleNodes]
        .filter(n => n.time_percentage && n.time_percentage > 0)
        .sort((a, b) => (b.time_percentage || 0) - (a.time_percentage || 0));
      
      this.topThreeNodeIds = sortedByTime.slice(0, 3).map(n => n.id);

      // 更新 SVG 尺寸以适应容器
      this.updateSvgSize();
      
      // 保存容器宽度，确保 SVG 不超出
      const containerWidth = this.svgWidth;
      
      // 优化布局：使用改进的树形布局
      const LEVEL_HEIGHT = 180;  // 垂直间距（层与层之间）
      const NODE_WIDTH = 250;    // 单个节点占据的水平空间
      const MIN_SIBLING_GAP = 80; // 兄弟节点之间的最小间距
      const SUBTREE_GAP = 120;   // 不同子树之间的额外间距
      let maxDepth = Math.max(...visibleNodes.map(n => n.depth || 0));
      const calculatedHeight = (maxDepth + 1) * LEVEL_HEIGHT + 150;
      // SVG 高度取容器高度和计算高度的较大值（垂直方向可以滚动）
      this.svgHeight = Math.max(this.svgHeight, calculatedHeight);

      // 构建节点映射
      const nodeIdMap = new Map(visibleNodes.map(n => [n.id, n]));
      
      // 使用改进的树形布局算法：确保规整对齐
      const nodePositions = new Map(); // nodeId -> x position
      const subtreeWidths = new Map(); // nodeId -> subtree width
      
      // 找到根节点（depth = 0）
      const rootNodes = visibleNodesByDepth.get(0) || [];
      
      // 第一步：计算每个子树的宽度（自底向上）
      const calculateSubtreeWidth = (nodeId) => {
        const node = nodeIdMap.get(nodeId);
        if (!node) return NODE_WIDTH;
        
        if (subtreeWidths.has(nodeId)) {
          return subtreeWidths.get(nodeId);
        }
        
        if (!node.children || node.children.length === 0) {
          // 叶子节点宽度
          subtreeWidths.set(nodeId, NODE_WIDTH);
          return NODE_WIDTH;
        }
        
        // 非叶子节点：宽度 = 所有子节点的子树宽度之和 + 子节点间的间距
        let totalWidth = 0;
        node.children.forEach((childId, index) => {
          const childWidth = calculateSubtreeWidth(childId);
          totalWidth += childWidth;
          if (index > 0) {
            totalWidth += MIN_SIBLING_GAP; // 兄弟节点之间的间距
          }
        });
        
        // 确保父节点至少有 NODE_WIDTH 的宽度
        totalWidth = Math.max(totalWidth, NODE_WIDTH);
        subtreeWidths.set(nodeId, totalWidth);
        return totalWidth;
      };
      
      // 为所有节点计算子树宽度
      rootNodes.forEach(rootNode => {
        calculateSubtreeWidth(rootNode.id);
      });
      
      // 第二步：递归布局，分配 x 坐标
      const layoutSubtree = (nodeId, startX) => {
        const node = nodeIdMap.get(nodeId);
        if (!node) return startX;
        
        const subtreeWidth = subtreeWidths.get(nodeId) || NODE_WIDTH;
        
        if (!node.children || node.children.length === 0) {
          // 叶子节点：位于子树空间的中心
          nodePositions.set(nodeId, startX + subtreeWidth / 2);
          return startX + subtreeWidth;
        }
        
        // 有子节点：先布局所有子节点
        let childX = startX;
        const childCenters = [];
        
        node.children.forEach(childId => {
          const childWidth = subtreeWidths.get(childId) || NODE_WIDTH;
          layoutSubtree(childId, childX);
          childCenters.push(childX + childWidth / 2);
          childX += childWidth + MIN_SIBLING_GAP;
        });
        
        // 父节点位置 = 子节点的中心点
        if (childCenters.length > 0) {
          const firstChildCenter = childCenters[0];
          const lastChildCenter = childCenters[childCenters.length - 1];
          nodePositions.set(nodeId, (firstChildCenter + lastChildCenter) / 2);
        } else {
          nodePositions.set(nodeId, startX + subtreeWidth / 2);
        }
        
        return startX + subtreeWidth;
      };
      
      // 从根节点开始布局
      let currentOffset = 0;
      rootNodes.forEach((rootNode, index) => {
        currentOffset = layoutSubtree(rootNode.id, currentOffset);
        if (index < rootNodes.length - 1) {
          currentOffset += SUBTREE_GAP; // 不同根节点之间的额外间距
        }
      });
      
      // 确保所有节点都有位置（处理可能的孤立节点）
      const depths = Array.from(visibleNodesByDepth.keys()).sort((a, b) => a - b);
      depths.forEach(depth => {
        const levelNodes = visibleNodesByDepth.get(depth);
        levelNodes.forEach(node => {
          if (!nodePositions.has(node.id)) {
            nodePositions.set(node.id, currentOffset);
            currentOffset += NODE_WIDTH + MIN_SIBLING_GAP;
          }
        });
      });
      
      // 第三步：计算实际布局宽度
      let minX = Infinity;
      let maxX = -Infinity;
      nodePositions.forEach(x => {
        minX = Math.min(minX, x);
        maxX = Math.max(maxX, x);
      });
      const totalWidth = maxX - minX + NODE_WIDTH;
      
      // SVG 宽度始终等于容器宽度（不超出容器）
      // 如果内容宽度大于容器，用户可以通过缩放和平移查看
      this.svgWidth = containerWidth;
      
      // 计算居中偏移量，同时减去 minX 使布局从 0 开始
      const offsetX = Math.max(50, (this.svgWidth - totalWidth) / 2) - minX;

      // 第四步：生成最终节点位置
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
              strokeWidth: Math.min(6, Math.max(2, Math.log10(rows + 1) * 0.8 + 1.5))
            });
          }
        });
      });
      
      // 首次渲染或数据变化时，自动适应屏幕
      if (this.needsAutoFit && this.renderedNodes.length > 0) {
        // 使用 setTimeout 确保 DOM 完全更新后再执行
        this.$nextTick(() => {
          setTimeout(() => {
            this.fitToScreen();
            this.needsAutoFit = false;
          }, 100);
        });
      }
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
      // 对于 SINK 节点，优先使用 input_rows（因为 SINK 节点接收数据但可能不返回数据）
      if (node?.operator_name && node.operator_name.includes('SINK')) {
        return node?.metrics?.input_rows || node?.metrics?.rows_returned || 0;
      }
      // 对于其他节点，使用 rows_returned
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
    formatNodeTitle(node) {
      if (!node) return 'UNKNOWN';
      const operatorName = this.formatOperatorName(node.operator_name);
      const planNodeId = node.plan_node_id !== undefined && node.plan_node_id !== null ? node.plan_node_id : '?';
      
      // 提取纯数字的 Fragment ID（可能是 "Fragment 2" 或 "2"）
      let fragmentId = '?';
      if (node.fragment_id !== undefined && node.fragment_id !== null) {
        const fid = String(node.fragment_id);
        const match = fid.match(/\d+/);
        fragmentId = match ? match[0] : fid;
      }
      
      // 提取纯数字的 Pipeline ID（可能是 "Pipeline 0" 或 "0"）
      let pipelineId = '?';
      if (node.pipeline_id !== undefined && node.pipeline_id !== null) {
        const pid = String(node.pipeline_id);
        const match = pid.match(/\d+/);
        pipelineId = match ? match[0] : pid;
      }
      
      return `${operatorName}(${planNodeId}-F${fragmentId}-P${pipelineId})`;
    },
    getJoinType(node) {
      if (!node) return null;
      
      // 检查是否是 HASH_JOIN 节点
      const isHashJoin = node.operator_name?.includes('HASH_JOIN') || 
                        node.operator_name?.includes('HASH JOIN');
      
      if (!isHashJoin) return null;
      
      // 尝试从 plan_info 中获取 join op 信息
      let joinOp = null;
      
      if (node.isMerged && node.primaryNode?.plan_info) {
        // 如果是合并节点，从 primaryNode 获取
        const joinOpItem = node.primaryNode.plan_info.find(item => item.key === 'join op');
        joinOp = joinOpItem?.value;
      } else if (node.plan_info) {
        // 普通节点
        const joinOpItem = node.plan_info.find(item => item.key === 'join op');
        joinOp = joinOpItem?.value;
      }
      
      return joinOp || null;
    },
    getTableName(node) {
      if (!node) return null;
      
      // 检查是否是 SCAN 节点
      const isScanNode = node.operator_name?.includes('SCAN_OPERATOR') || 
                        node.operator_name?.includes('OLAP_SCAN') ||
                        node.operator_name?.includes('FILE_SCAN');
      
      if (!isScanNode) return null;
      
      // 尝试从 table_name 字段获取
      let tableName = null;
      
      if (node.isMerged && node.primaryNode?.table_name) {
        // 如果是合并节点，从 primaryNode 获取
        tableName = node.primaryNode.table_name;
      } else if (node.table_name) {
        // 普通节点
        tableName = node.table_name;
      }
      
      return tableName || null;
    },
    getNodeHeaderHeight(node) {
      // 如果有 JOIN 类型信息或 TABLE 名称信息，标题栏高度增加
      return (this.getJoinType(node) || this.getTableName(node)) ? 42 : this.NODE_HEADER_HEIGHT;
    },
    getNodeTotalHeight(node) {
      // 总高度 = 标题高度 + body 高度 + progress 高度
      return this.getNodeHeaderHeight(node) + this.NODE_BODY_HEIGHT + this.NODE_PROGRESS_HEIGHT;
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
      
      // 当有 pipeline 过滤器时，返回特殊 class
      if (this.hasActivePipelineFilter) {
        const nodePipelineId = node.pipeline_id;
        const nodeFragmentId = node.fragment_id;
        // 同时检查 fragment_id 和 pipeline_id
        if (nodePipelineId && 
            nodeFragmentId === this.selectedFragmentId && 
            this.selectedPipelineIds.includes(nodePipelineId)) {
          return 'pipeline-highlight';
        }
        return 'pipeline-dimmed';
      }
      
      // 原有逻辑
      const name = node.operator_name || '';
      if (name.includes('MULTI_CAST')) return 'multicast';  // Check multi-cast first
      if (name.includes('SET_SINK') || name.includes('SET_PROBE') || name.includes('INTERSECT') || name.includes('EXCEPT')) return 'set-op';
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
      if (pct > 30) return '#F5222D';  // Doris 红色 - 高耗时
      if (pct > 10) return '#FA8C16';  // Doris 橙色 - 中等耗时
      return '#52C41A';                // Doris 绿色 - 低耗时
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
      
      // 应用缩放（允许缩小到 5% 以显示完整树形结构）
      const newZoom = Math.min(3, Math.max(0.05, this.zoom * factor));
      this.zoom = newZoom;
      
      // 调整 pan 偏移，使该点保持不变
      this.panX = pointX - beforeZoomX * newZoom;
      this.panY = pointY - beforeZoomY * newZoom;
    },
    fitToScreen() {
      // 自动计算最佳缩放比例以适应屏幕
      if (!this.renderedNodes || this.renderedNodes.length === 0) {
        this.zoom = 0.8;
        this.panX = 50;
        this.panY = 50;
        return;
      }
      
      // 确保 SVG 尺寸已更新
      this.updateSvgSize();
      
      // 验证 SVG 尺寸是否有效
      if (this.svgWidth <= 0 || this.svgHeight <= 0) {
        console.warn('Invalid SVG size, skipping fitToScreen');
        return;
      }
      
      // 计算内容边界
      let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
      this.renderedNodes.forEach(node => {
        minX = Math.min(minX, node.x);
        maxX = Math.max(maxX, node.x + 200); // NODE_WIDTH
        minY = Math.min(minY, node.y);
        maxY = Math.max(maxY, node.y + 90); // NODE_HEIGHT
      });
      
      const contentWidth = maxX - minX;
      const contentHeight = maxY - minY;
      
      // 验证内容尺寸
      if (contentWidth <= 0 || contentHeight <= 0 || !isFinite(contentWidth) || !isFinite(contentHeight)) {
        console.warn('Invalid content dimensions, skipping fitToScreen');
        return;
      }
      
      // 计算缩放比例（留 15% 边距）
      const scaleX = (this.svgWidth * 0.85) / contentWidth;
      const scaleY = (this.svgHeight * 0.85) / contentHeight;
      let newZoom = Math.min(scaleX, scaleY);
      
      // 限制缩放范围（允许缩小到 5% 以显示完整树形结构）
      newZoom = Math.max(0.05, Math.min(newZoom, 2));
      
      if (!isFinite(newZoom)) {
        console.warn('Invalid zoom calculated, using default');
        newZoom = 0.8;
      }
      
      // 居中
      const centerX = (minX + maxX) / 2;
      const centerY = (minY + maxY) / 2;
      
      this.zoom = newZoom;
      this.panX = this.svgWidth / 2 - centerX * newZoom;
      this.panY = this.svgHeight / 2 - centerY * newZoom;
    },
    resetView() { this.zoom = 1; this.panX = 50; this.panY = 50; this.deselectNode(); },
    
    // 搜索相关方法
    handleSearch() {
      const query = this.searchText.trim().toLowerCase();
      if (!query) {
        this.searchResults = [];
        this.currentSearchIndex = -1;
        return;
      }
      
      // 搜索所有匹配的节点
      this.searchResults = this.renderedNodes.filter(node => {
        // 匹配操作符名称
        const operatorMatch = node.operator_name && 
          node.operator_name.toLowerCase().includes(query);
        
        // 匹配 Fragment ID (例如: F8, Fragment 8)
        const fragmentMatch = node.fragment_id && 
          node.fragment_id.toLowerCase().includes(query);
        
        // 匹配 Pipeline ID (例如: P9, Pipeline 9)
        const pipelineMatch = node.pipeline_id && 
          node.pipeline_id.toLowerCase().includes(query);
        
        // 匹配 Node ID (例如: 54)
        const nodeIdMatch = node.plan_node_id && 
          node.plan_node_id.toString().includes(query);
        
        // 匹配组合标识符 (例如: 54-F8-P9, F8-P9)
        const compositeMatch = node.id && 
          node.id.toLowerCase().includes(query);
        
        // 匹配格式化后的标题 (例如: FILE_SCAN(74-F2-P2))
        const formattedTitle = this.formatNodeTitle(node);
        const titleMatch = formattedTitle && 
          formattedTitle.toLowerCase().includes(query);
        
        return operatorMatch || fragmentMatch || pipelineMatch || 
               nodeIdMatch || compositeMatch || titleMatch;
      });
      
      // 如果有结果，跳转到第一个
      if (this.searchResults.length > 0) {
        this.currentSearchIndex = 0;
        this.focusSearchResult();
      } else {
        this.currentSearchIndex = -1;
      }
    },
    
    searchNext() {
      if (this.searchResults.length === 0) return;
      this.currentSearchIndex = (this.currentSearchIndex + 1) % this.searchResults.length;
      this.focusSearchResult();
    },
    
    searchPrev() {
      if (this.searchResults.length === 0) return;
      this.currentSearchIndex = (this.currentSearchIndex - 1 + this.searchResults.length) % this.searchResults.length;
      this.focusSearchResult();
    },
    
    focusSearchResult() {
      if (this.currentSearchIndex < 0 || this.currentSearchIndex >= this.searchResults.length) return;
      
      const node = this.searchResults[this.currentSearchIndex];
      if (node) {
        // 使用现有的 locateAndCenterNode 方法
        this.locateAndCenterNode(node.id);
        
        // 高亮显示当前搜索结果
        this.selectedNodeId = node.id;
        this.selectedNode = node;
      }
    },
    
    // Pipeline 过滤器方法
    assignPipelineColors() {
      // 定义一组鲜艳明亮的颜色（高饱和度、高对比度）
      const colorPalette = [
        '#FF2D55',  // 鲜红色 (亮红)
        '#FF9500',  // 亮橙色 (活力橙)
        '#FFD700',  // 金黄色 (金色)
        '#00C853',  // 鲜绿色 (翠绿)
        '#00BCD4',  // 亮青色 (青蓝)
        '#2196F3',  // 亮蓝色 (天蓝)
        '#9C27B0',  // 鲜紫色 (紫罗兰)
        '#E91E63',  // 洋红色 (玫红)
      ];
      
      this.pipelineColorMap = {};
      this.selectedPipelineIds.forEach((pipeId, idx) => {
        this.pipelineColorMap[pipeId] = colorPalette[idx % colorPalette.length];
      });
    },
    
    // Fragment 选择变化
    onFragmentChange() {
      if (this.selectedFragmentId) {
        // 自动全选该 Fragment 下的所有 pipelines
        this.selectedPipelineIds = [...this.availablePipelines];
        this.assignPipelineColors();
        
        // 居中显示该 Fragment 的所有节点
        this.$nextTick(() => {
          this.centerFragmentNodes(this.selectedFragmentId);
        });
      } else {
        this.selectedPipelineIds = [];
        this.pipelineColorMap = {};
      }
    },
    
    // 居中显示 Fragment 的所有节点
    centerFragmentNodes(fragmentId) {
      const fragmentNodes = this.nodesByFragment[fragmentId] || [];
      if (fragmentNodes.length === 0) return;
      
      // 找到这些节点在 renderedNodes 中的位置
      const nodeIds = new Set(fragmentNodes.map(n => n.id));
      const visibleNodes = this.renderedNodes.filter(n => nodeIds.has(n.id));
      
      if (visibleNodes.length === 0) return;
      
      // 计算边界框
      let minX = Infinity, minY = Infinity;
      let maxX = -Infinity, maxY = -Infinity;
      
      visibleNodes.forEach(node => {
        minX = Math.min(minX, node.x);
        minY = Math.min(minY, node.y);
        maxX = Math.max(maxX, node.x + this.NODE_WIDTH);
        maxY = Math.max(maxY, node.y + this.getNodeTotalHeight(node));
      });
      
      // 计算中心点
      const centerX = (minX + maxX) / 2;
      const centerY = (minY + maxY) / 2;
      
      // 计算需要的缩放比例
      const width = maxX - minX;
      const height = maxY - minY;
      const padding = 100; // 边距
      
      const scaleX = (this.svgWidth - padding * 2) / width;
      const scaleY = (this.svgHeight - padding * 2) / height;
      const newZoom = Math.min(scaleX, scaleY, 1.0); // 最大不超过 1.0
      
      // 设置缩放
      this.zoom = Math.max(0.1, newZoom);
      
      // 计算平移，使中心点位于视图中心
      this.panX = this.svgWidth / 2 - centerX * this.zoom;
      this.panY = this.svgHeight / 2 - centerY * this.zoom;
    },
    
    // Pipeline 多选变化
    onPipelineSelectionChange() {
      this.assignPipelineColors();
    },
    
    // 清除过滤器
    clearPipelineFilter() {
      this.selectedFragmentId = null;
      this.selectedPipelineIds = [];
      this.pipelineColorMap = {};
    },
    
    // 全选当前 fragment 的所有 pipelines
    selectAllPipelines() {
      this.selectedPipelineIds = [...this.availablePipelines];
      this.assignPipelineColors();
    },
    
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
      
      // 应用缩放（允许缩小到 5% 以显示完整树形结构）
      const delta = event.deltaY;
      const zoomSensitivity = 0.001; // 降低敏感度
      const zoomChange = -delta * zoomSensitivity;
      const oldZoom = this.zoom;
      const newZoom = Math.min(3, Math.max(0.05, this.zoom * (1 + zoomChange)));
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
      
      // 如果有 pipeline 过滤器激活
      if (this.hasActivePipelineFilter) {
        const nodePipelineId = node.pipeline_id;
        const nodeFragmentId = node.fragment_id;
        // 同时检查 fragment_id 和 pipeline_id
        if (nodePipelineId && 
            nodeFragmentId === this.selectedFragmentId && 
            this.selectedPipelineIds.includes(nodePipelineId)) {
          // 返回该 pipeline 的专属颜色
          return this.pipelineColorMap[nodePipelineId];
        } else {
          // 未选中的节点返回灰色
          return '#BFBFBF';
        }
      }
      
      // 原有的颜色逻辑（无过滤器时）
      if (node.is_hotspot) return '#F5222D';  // Doris 红色 - 热点
      const name = node.operator_name || '';
      if (name.includes('MULTI_CAST')) return '#FA8C16';   // Doris 橙色 - 广播操作
      if (name.includes('SET_SINK') || name.includes('SET_PROBE') || name.includes('INTERSECT') || name.includes('EXCEPT')) return '#9254DE'; // Doris 紫色 - SET操作
      if (name.includes('SCAN')) return '#52C41A';      // Doris 绿色 - 数据源
      if (name.includes('JOIN')) return '#2F54EB';      // Doris 蓝色 - 核心操作
      if (name.includes('AGGREGATE') || name.includes('AGGREGATION')) return '#722ED1'; // Doris 紫色 - 聚合
      if (name.includes('EXCHANGE') || name.includes('STREAM')) return '#5B8FF9';  // Doris 浅蓝 - 数据交换
      if (name.includes('SORT')) return '#13C2C2';      // Doris 青色 - 排序
      if (name.includes('PROJECT')) return '#8C8C8C';   // 灰色 - 投影
      return '#8C8C8C';
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
    },
    
    // Pipeline View 折叠相关方法
    initializeCollapseState() {
      // 默认折叠所有 Fragment 和 Pipeline
      this.collapsedFragments = new Set(this.fragmentIds);
      this.collapsedPipelines = new Set();
      this.fragmentIds.forEach(fragId => {
        const pipelineIds = this.getPipelineIds(fragId);
        pipelineIds.forEach(pipeId => {
          this.collapsedPipelines.add(`${fragId}|${pipeId}`);
        });
      });
    },
    isFragmentCollapsed(fragId) {
      return this.collapsedFragments.has(fragId);
    },
    toggleFragmentCollapse(fragId) {
      if (this.collapsedFragments.has(fragId)) {
        this.collapsedFragments.delete(fragId);
      } else {
        this.collapsedFragments.add(fragId);
      }
      // 强制触发 Vue 响应式更新
      this.collapsedFragments = new Set(this.collapsedFragments);
    },
    isPipelineCollapsed(fragId, pipeId) {
      return this.collapsedPipelines.has(`${fragId}|${pipeId}`);
    },
    togglePipelineCollapse(fragId, pipeId) {
      const key = `${fragId}|${pipeId}`;
      if (this.collapsedPipelines.has(key)) {
        this.collapsedPipelines.delete(key);
      } else {
        this.collapsedPipelines.add(key);
      }
      // 强制触发 Vue 响应式更新
      this.collapsedPipelines = new Set(this.collapsedPipelines);
    },
    
    // 定位并居中显示指定节点
    locateAndCenterNode(nodeId) {
      // 首先在 renderedNodes 中直接查找节点
      let targetNode = this.renderedNodes.find(n => n.id === nodeId);
      let actualNodeId = nodeId;
      
      // 如果找不到，检查是否是合并节点的子节点
      if (!targetNode) {
        for (const node of this.renderedNodes) {
          if (node.isMerged) {
            // 检查 primaryNode 或 secondaryNode 的 id 是否匹配
            if (node.primaryNode?.id === nodeId || node.secondaryNode?.id === nodeId) {
              targetNode = node;
              actualNodeId = node.id; // 使用合并节点的 id 进行高亮
              console.log(`Found node ${nodeId} in merged node ${node.id}`);
              break;
            }
          }
        }
      }
      
      if (!targetNode || !targetNode.x || !targetNode.y) {
        console.warn(`Node ${nodeId} not found or has no position`);
        return;
      }
      
      // 计算节点中心位置
      const nodeCenterX = targetNode.x + this.NODE_WIDTH / 2;
      const nodeCenterY = targetNode.y + this.getNodeTotalHeight(targetNode) / 2;
      
      // 设置合适的缩放级别（如果当前缩放太小）
      const targetZoom = Math.max(this.zoom, 0.8);
      
      // 计算新的 pan 值，使节点居中
      this.panX = this.svgWidth / 2 - nodeCenterX * targetZoom;
      this.panY = this.svgHeight / 2 - nodeCenterY * targetZoom;
      this.zoom = targetZoom;
      
      // 选中节点
      this.selectNode(targetNode);
      
      // 添加视觉反馈：短暂高亮（使用实际渲染的节点 ID）
      this.$nextTick(() => {
        const element = document.querySelector(`[data-node-id="${actualNodeId}"]`);
        if (element) {
          element.classList.add('node-highlight');
          setTimeout(() => {
            element.classList.remove('node-highlight');
          }, 1000);
        }
      });
    },
    // 判断节点是否是最耗时的三个节点之一
    isTopThreeNode(nodeId) {
      return this.topThreeNodeIds.includes(nodeId);
    }
  }
};
</script>

<style scoped lang="scss">
.execution-graph {
  width: 100%;
  height: 100%;
  background: white;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.view-controls {
  display: flex;
  flex-direction: column;
  padding: 12px 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
  flex-shrink: 0;
  gap: 8px;
}

.controls-row-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.controls-row-search {
  display: flex;
  gap: 8px;
  align-items: center;
  padding-left: 4px;
}

.view-tabs {
  display: flex;
  gap: 10px;
  align-items: center;
}

.view-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  background: white;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  flex: 1;
  max-width: 500px;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 13px;
  padding: 4px 0;
  
  &::placeholder {
    color: #999;
  }
}

.search-count {
  font-size: 11px;
  color: #666;
  white-space: nowrap;
  padding-left: 8px;
  border-left: 1px solid #e0e0e0;
}

.controls-row-filter {
  display: flex;
  gap: 10px;
  align-items: center;
  padding-left: 4px;
  
  label {
    font-weight: 500;
    color: #666;
    font-size: 12px;
  }
}

.pipeline-color-dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin-right: 6px;
  vertical-align: middle;
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background: #d0d0d0;
  margin: 0 4px;
}

.view-btn {
  padding: 6px 14px;
  border: 1px solid #d0d0d0;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
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
  background: white;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}


.toolbar-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 10px;
  border: none;
  background: white;
  border-radius: 4px;
  cursor: pointer;
  color: #666;
  transition: all 0.2s;
  white-space: nowrap;
  font-size: 13px;
  
  &:hover:not(:disabled) {
    background: #f5f5f5;
    color: #333;
  }
  
  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
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
  stroke: #757575;  // 从 #BDBDBD 改为更深的灰色
  stroke-width: 2;  // 设置默认宽度
  transition: stroke 0.3s;
  
  &.hotspot-link {
    stroke: #E57373;
    stroke-width: 3;  // 热点连线更粗
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
  
  /* 最耗时的三个节点：红色标题 + 红色边框 */
  &.top-time-consuming .node-header {
    fill: #F5222D !important;  /* 红色标题背景，覆盖所有类型的颜色 */
  }
  
  &.top-time-consuming .node-border {
    stroke: #F5222D;
    stroke-width: 3;
    stroke-dasharray: none;
    filter: drop-shadow(0 0 4px rgba(245, 34, 45, 0.5));
  }
  
  /* 如果同时是 hotspot 和 top-time-consuming，优先使用 top-time-consuming 样式 */
  &.top-time-consuming.hotspot .node-border {
    stroke: #F5222D;
    stroke-width: 3;
  }
  
  &.node-highlight {
    animation: highlight-pulse 1s ease-out;
  }
  
  /* Pipeline 过滤器样式 */
  &.pipeline-dimmed {
    opacity: 0.3;
  }
  
  &.pipeline-highlight {
    opacity: 1;
  }
}

@keyframes highlight-pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}

.node-header {
  // 默认颜色只在非过滤模式下应用
  .node-group:not(.pipeline-highlight):not(.pipeline-dimmed) & {
    fill: #595959;
  }
  
  &.header-scan { fill: #52C41A; }      // Doris 绿色 - 数据源
  &.header-join { fill: #2F54EB; }      // Doris 蓝色 - 核心操作
  &.header-aggregate { fill: #722ED1; } // Doris 紫色 - 聚合
  &.header-exchange { fill: #5B8FF9; }  // Doris 浅蓝 - 数据交换
  &.header-sort { fill: #13C2C2; }      // Doris 青色 - 排序
  &.header-project { fill: #8C8C8C; }   // 灰色 - 投影
  &.header-multicast { fill: #FA8C16; } // Doris 橙色 - 广播操作
  &.header-set-op { fill: #9254DE; }    // Doris 紫色 - SET操作
}

.node-body { fill: white; }
.progress-bg { fill: #F5F5F5; }
.node-border { fill: none; stroke: #E0E0E0; stroke-width: 1; }
.node-title { font-size: 11px; font-weight: 600; fill: white; pointer-events: none; }
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

.metric-item {
  padding: 8px 0;
  border-bottom: 1px solid #f5f5f5;
  font-size: 12px;
  
  &:last-child {
    border-bottom: none;
  }
  
  .metric-label {
    color: #3498db;
    font-weight: 600;
    margin-bottom: 4px;
    font-size: 11px;
  }
  
  .metric-value {
    color: #555;
    line-height: 1.6;
    word-break: break-word;
    font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
    font-size: 11px;
  }
  
  .metric-children {
    margin-top: 6px;
    margin-left: 16px;
    padding-left: 12px;
    border-left: 2px solid #e0e0e0;
    
    .metric-child {
      margin-bottom: 6px;
      
      &:last-child {
        margin-bottom: 0;
      }
      
      .metric-label {
        color: #e67e22;
        font-weight: 500;
        margin-bottom: 2px;
        font-size: 10px;
      }
      
      .metric-value {
        color: #666;
        font-size: 10px;
      }
    }
  }
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
  grid-template-columns: 80px 80px 70px 2fr 120px 80px 100px;
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

.col-nodeid {
  color: #666;
  text-align: center;
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
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
}

.fragment-title:hover {
  background: #34495e;
}

.collapse-icon {
  font-size: 12px;
  width: 16px;
  display: inline-block;
  text-align: center;
}

.frag-icon {
  font-size: 18px;
}

.pipe-icon {
  font-size: 16px;
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
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 13px;
  color: #34495e;
  padding: 8px 12px;
  background: #ecf0f1;
  border-radius: 4px;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
}

.pipeline-title:hover {
  background: #d5dbdd;
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

.op-node-id {
  color: #999;
  font-size: 11px;
  margin-left: 6px;
  font-weight: normal;
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
