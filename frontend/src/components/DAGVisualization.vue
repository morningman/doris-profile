<template>
  <div class="dag-visualization" ref="containerRef">
    <div v-if="!tree || !tree.nodes || tree.nodes.length === 0" class="no-data">
      <i class="fas fa-project-diagram"></i>
      <p>No execution tree data available</p>
    </div>
    <div v-else class="dag-container">
      <svg ref="svgRef" class="dag-svg"></svg>
      <!-- Node tooltip -->
      <div
        v-if="hoveredNode"
        class="node-tooltip"
        :style="tooltipStyle"
      >
        <div class="tooltip-header">
          <span class="operator-name">{{ hoveredNode.operator_name }}</span>
          <span v-if="hoveredNode.time_percentage" class="time-pct">
            {{ hoveredNode.time_percentage.toFixed(1) }}%
          </span>
        </div>
        <div class="tooltip-content">
          <div v-if="hoveredNode.plan_node_id !== null" class="tooltip-row">
            <span class="label">Plan Node ID:</span>
            <span class="value">{{ hoveredNode.plan_node_id }}</span>
          </div>
          <div v-if="hoveredNode.fragment_id" class="tooltip-row">
            <span class="label">Fragment:</span>
            <span class="value">{{ hoveredNode.fragment_id }}</span>
          </div>
          <div v-if="hoveredNode.metrics?.rows_returned" class="tooltip-row">
            <span class="label">Rows:</span>
            <span class="value">{{ formatNumber(hoveredNode.metrics.rows_returned) }}</span>
          </div>
          <div v-if="hoveredNode.metrics?.operator_total_time_raw" class="tooltip-row">
            <span class="label">Time:</span>
            <span class="value">{{ hoveredNode.metrics.operator_total_time_raw }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import * as d3 from "d3";

export default {
  name: "DAGVisualization",
  props: {
    tree: {
      type: Object,
      default: null,
    },
  },
  setup(props) {
    const containerRef = ref(null);
    const svgRef = ref(null);
    const hoveredNode = ref(null);
    const tooltipPosition = ref({ x: 0, y: 0 });

    const tooltipStyle = computed(() => ({
      left: `${tooltipPosition.value.x}px`,
      top: `${tooltipPosition.value.y}px`,
    }));

    const formatNumber = (num) => {
      if (num === null || num === undefined) return "N/A";
      return num.toLocaleString();
    };

    const getNodeColor = (node) => {
      if (node.is_most_consuming) return "#f56c6c";
      if (node.is_second_most_consuming) return "#e6a23c";
      if (node.is_hotspot) {
        switch (node.hotspot_severity) {
          case "Critical":
            return "#f56c6c";
          case "High":
            return "#e6a23c";
          case "Medium":
            return "#409eff";
          default:
            return "#67c23a";
        }
      }
      return "#409eff";
    };

    const buildHierarchy = (tree) => {
      if (!tree || !tree.nodes || tree.nodes.length === 0) return null;

      const nodeMap = new Map();
      tree.nodes.forEach((node) => {
        nodeMap.set(node.id, { ...node, children: [] });
      });

      // Build parent-child relationships
      tree.nodes.forEach((node) => {
        if (node.children && node.children.length > 0) {
          const parent = nodeMap.get(node.id);
          node.children.forEach((childId) => {
            const child = nodeMap.get(childId);
            if (child) {
              parent.children.push(child);
            }
          });
        }
      });

      // Find root (node with no parent)
      const root = nodeMap.get(tree.root?.id) || nodeMap.values().next().value;
      return root;
    };

    const renderDAG = () => {
      if (!svgRef.value || !containerRef.value || !props.tree) return;

      const container = containerRef.value;
      const width = container.clientWidth || 600;
      const height = Math.max(400, container.clientHeight || 400);

      // Clear existing content
      d3.select(svgRef.value).selectAll("*").remove();

      const svg = d3
        .select(svgRef.value)
        .attr("width", width)
        .attr("height", height);

      const g = svg.append("g").attr("transform", "translate(50, 20)");

      // Build hierarchy
      const hierarchyData = buildHierarchy(props.tree);
      if (!hierarchyData) return;

      const root = d3.hierarchy(hierarchyData);

      // Create tree layout
      const treeLayout = d3
        .tree()
        .size([height - 40, width - 150])
        .separation((a, b) => (a.parent === b.parent ? 1 : 1.5));

      treeLayout(root);

      // Draw links
      g.selectAll(".link")
        .data(root.links())
        .enter()
        .append("path")
        .attr("class", "link")
        .attr("fill", "none")
        .attr("stroke", "#dcdfe6")
        .attr("stroke-width", 2)
        .attr(
          "d",
          d3
            .linkHorizontal()
            .x((d) => d.y)
            .y((d) => d.x)
        );

      // Draw nodes
      const node = g
        .selectAll(".node")
        .data(root.descendants())
        .enter()
        .append("g")
        .attr("class", "node")
        .attr("transform", (d) => `translate(${d.y},${d.x})`)
        .style("cursor", "pointer")
        .on("mouseenter", (event, d) => {
          hoveredNode.value = d.data;
          const rect = containerRef.value.getBoundingClientRect();
          tooltipPosition.value = {
            x: event.clientX - rect.left + 10,
            y: event.clientY - rect.top + 10,
          };
        })
        .on("mousemove", (event) => {
          const rect = containerRef.value.getBoundingClientRect();
          tooltipPosition.value = {
            x: event.clientX - rect.left + 10,
            y: event.clientY - rect.top + 10,
          };
        })
        .on("mouseleave", () => {
          hoveredNode.value = null;
        });

      // Node circles
      node
        .append("circle")
        .attr("r", 8)
        .attr("fill", (d) => getNodeColor(d.data))
        .attr("stroke", "#fff")
        .attr("stroke-width", 2);

      // Node labels
      node
        .append("text")
        .attr("dy", 4)
        .attr("x", (d) => (d.children ? -12 : 12))
        .attr("text-anchor", (d) => (d.children ? "end" : "start"))
        .attr("font-size", "11px")
        .attr("fill", "#303133")
        .text((d) => {
          const name = d.data.operator_name || "Unknown";
          return name.length > 20 ? name.substring(0, 17) + "..." : name;
        });

      // Add percentage labels for hotspots
      node
        .filter((d) => d.data.time_percentage && d.data.time_percentage > 5)
        .append("text")
        .attr("dy", -12)
        .attr("text-anchor", "middle")
        .attr("font-size", "10px")
        .attr("font-weight", "600")
        .attr("fill", (d) => getNodeColor(d.data))
        .text((d) => `${d.data.time_percentage.toFixed(1)}%`);

      // Enable zoom and pan
      const zoom = d3
        .zoom()
        .scaleExtent([0.5, 3])
        .on("zoom", (event) => {
          g.attr("transform", event.transform);
        });

      svg.call(zoom);
    };

    // Watch for tree changes
    watch(
      () => props.tree,
      () => {
        renderDAG();
      },
      { deep: true }
    );

    // Handle resize
    let resizeObserver = null;

    onMounted(() => {
      renderDAG();
      
      if (containerRef.value) {
        resizeObserver = new ResizeObserver(() => {
          renderDAG();
        });
        resizeObserver.observe(containerRef.value);
      }
    });

    onUnmounted(() => {
      if (resizeObserver) {
        resizeObserver.disconnect();
      }
    });

    return {
      containerRef,
      svgRef,
      hoveredNode,
      tooltipStyle,
      formatNumber,
    };
  },
};
</script>

<style lang="scss" scoped>
.dag-visualization {
  width: 100%;
  height: 100%;
  min-height: 400px;
  position: relative;
}

.no-data {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  color: var(--text-tertiary);

  i {
    font-size: 48px;
    margin-bottom: 16px;
  }

  p {
    margin: 0;
  }
}

.dag-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
  border-radius: 8px;
  background: #fafafa;
}

.dag-svg {
  display: block;
}

.node-tooltip {
  position: absolute;
  background: rgba(48, 49, 51, 0.95);
  color: #fff;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 12px;
  z-index: 100;
  pointer-events: none;
  max-width: 300px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);

  .tooltip-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);

    .operator-name {
      font-weight: 600;
    }

    .time-pct {
      background: rgba(245, 108, 108, 0.2);
      color: #f56c6c;
      padding: 2px 8px;
      border-radius: 10px;
      font-size: 11px;
      font-weight: 600;
    }
  }

  .tooltip-content {
    .tooltip-row {
      display: flex;
      justify-content: space-between;
      gap: 16px;
      margin-bottom: 4px;

      &:last-child {
        margin-bottom: 0;
      }

      .label {
        color: rgba(255, 255, 255, 0.7);
      }

      .value {
        font-weight: 500;
      }
    }
  }
}
</style>

