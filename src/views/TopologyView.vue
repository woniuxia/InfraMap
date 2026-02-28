<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import TopologyCanvas from '@/components/topology/TopologyCanvas.vue'
import TopologyToolbar from '@/components/topology/TopologyToolbar.vue'
import TopologyDetailPanel from '@/components/topology/TopologyDetailPanel.vue'
import { findPaths, analyzeImpact } from '@/api/topology'
import { useTopologyStore } from '@/stores/topology'
import type { TopologyNode, PathResult, ImpactResult } from '@/types'

// Store
const topologyStore = useTopologyStore()

// Data (from store)
const graphData = computed(() => topologyStore.graphData)
const loading = computed(() => topologyStore.loading)
const canvasRef = ref<InstanceType<typeof TopologyCanvas>>()

// Panel state
const panelMode = ref<'detail' | 'path' | 'impact' | null>(null)
const selectedNode = ref<TopologyNode | null>(null)
const pathResult = ref<PathResult | null>(null)
const impactResult = ref<ImpactResult | null>(null)

// Path tracing state
const pathTraceMode = ref(false)
const pathSource = ref<string | null>(null)
const pathTraceHint = ref('')

// Context menu
const contextMenuVisible = ref(false)
const contextMenuPos = ref({ x: 0, y: 0 })
const contextMenuNode = ref<TopologyNode | null>(null)

// Fullscreen
const isFullscreen = ref(false)
const containerRef = ref<HTMLDivElement>()

// Node name map for detail panel
const nodeNameMap = computed(() => {
  const map: Record<string, string> = {}
  if (graphData.value) {
    graphData.value.nodes.forEach((n) => (map[n.id] = n.name))
    graphData.value.combos.forEach((c) => (map[c.id] = c.label))
  }
  return map
})

async function loadData() {
  await topologyStore.fetchGraph(true)
}

// Node click handler
function handleNodeClick(node: TopologyNode) {
  if (pathTraceMode.value) {
    if (!pathSource.value) {
      pathSource.value = node.id
      pathTraceHint.value = `已选起点: ${node.name}，请 Ctrl+点击终点`
      ElMessage.info(pathTraceHint.value)
    } else {
      // Second click -> find paths
      handleFindPaths(pathSource.value, node.id)
      pathTraceMode.value = false
      pathSource.value = null
      pathTraceHint.value = ''
    }
    return
  }

  selectedNode.value = node
  panelMode.value = 'detail'
}

// Ctrl+click to enter path trace mode
function handleCanvasNodeClick(node: TopologyNode) {
  // We'll check for ctrl key on the canvas event
  // For now, regular clicks just show detail
  handleNodeClick(node)
}

// Right-click context menu
function handleContextMenu(payload: { node: TopologyNode; x: number; y: number }) {
  contextMenuNode.value = payload.node
  contextMenuPos.value = { x: payload.x, y: payload.y }
  contextMenuVisible.value = true
}

function closeContextMenu() {
  contextMenuVisible.value = false
}

// Path tracing
function startPathTrace() {
  closeContextMenu()
  pathTraceMode.value = true
  pathSource.value = contextMenuNode.value?.id || null
  if (pathSource.value) {
    pathTraceHint.value = `已选起点: ${contextMenuNode.value?.name}，请点击终点`
    ElMessage.info(pathTraceHint.value)
  }
}

async function handleFindPaths(sourceId: string, targetId: string) {
  try {
    pathResult.value = await findPaths(sourceId, targetId)
    panelMode.value = 'path'
    if (pathResult.value.paths.length > 0) {
      canvasRef.value?.highlightPaths(pathResult.value.paths)
    } else {
      ElMessage.warning('未找到连接路径')
    }
  } catch {
    // error shown by tauriInvoke
  }
}

// Impact analysis
async function handleAnalyzeImpact() {
  const node = contextMenuNode.value
  closeContextMenu()
  if (!node) return

  try {
    impactResult.value = await analyzeImpact(node.id)
    selectedNode.value = node
    panelMode.value = 'impact'
    canvasRef.value?.highlightImpact(node.id, impactResult.value)
  } catch {
    // error shown by tauriInvoke
  }
}

// Search
function handleSearch(payload: { matchIds: string[]; focusId?: string }) {
  if (payload.matchIds.length === 0) {
    canvasRef.value?.clearHighlight()
    return
  }
  canvasRef.value?.highlightSearch(payload.matchIds, payload.focusId)
}

// Filter
function handleFilter(payload: { types: string[]; env: string }) {
  canvasRef.value?.applyFilter(payload)
}

// Layout
function handleLayoutChange(type: 'force' | 'dagre') {
  canvasRef.value?.setLayout(type)
}

// Export
async function handleExport(type: 'png' | 'svg') {
  const dataURL = await canvasRef.value?.exportImage(type)
  if (!dataURL) return

  const link = document.createElement('a')
  link.download = `topology.${type}`
  link.href = dataURL
  link.click()
  ElMessage.success('导出成功')
}

// Panel close
function handlePanelClose() {
  panelMode.value = null
  selectedNode.value = null
  pathResult.value = null
  impactResult.value = null
  canvasRef.value?.clearHighlight()
}

// Fullscreen toggle
function toggleFullscreen() {
  if (!containerRef.value) return
  if (!isFullscreen.value) {
    containerRef.value.requestFullscreen?.()
    isFullscreen.value = true
  } else {
    document.exitFullscreen?.()
    isFullscreen.value = false
  }
}

// Listen for fullscreen change
function handleFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement
}

onMounted(() => {
  loadData()
  document.addEventListener('fullscreenchange', handleFullscreenChange)
})
</script>

<template>
  <div ref="containerRef" class="topology-view" v-loading="loading" @click="closeContextMenu">
    <!-- Toolbar -->
    <TopologyToolbar
      :nodes="graphData?.nodes || []"
      @search="handleSearch"
      @filter="handleFilter"
      @layout-change="handleLayoutChange"
      @export="handleExport"
      @refresh="loadData"
      @fullscreen="toggleFullscreen"
    />

    <!-- Path trace hint -->
    <div v-if="pathTraceMode" class="path-trace-bar">
      <span>{{ pathTraceHint || '请点击起始节点' }}</span>
      <el-button
        size="small"
        type="info"
        text
        @click="pathTraceMode = false; pathSource = null; pathTraceHint = ''"
      >
        取消
      </el-button>
    </div>

    <!-- Content area -->
    <div class="topology-content">
      <!-- Canvas -->
      <div class="canvas-wrapper">
        <TopologyCanvas
          ref="canvasRef"
          :graph-data="graphData"
          @node-click="handleCanvasNodeClick"
          @node-contextmenu="handleContextMenu"
        />

        <!-- Empty state -->
        <div v-if="graphData && graphData.nodes.length === 0 && !loading" class="empty-overlay">
          <el-empty description="暂无拓扑数据，请先添加资源和关系" />
        </div>
      </div>

      <!-- Detail panel -->
      <TopologyDetailPanel
        :mode="panelMode"
        :selected-node="selectedNode"
        :path-result="pathResult"
        :impact-result="impactResult"
        :node-name-map="nodeNameMap"
        @close="handlePanelClose"
      />
    </div>

    <!-- Context menu -->
    <teleport to="body">
      <div
        v-if="contextMenuVisible"
        class="context-menu"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
        @click.stop
      >
        <div class="context-menu-item" @click="startPathTrace">
          路径追踪（从此节点出发）
        </div>
        <div class="context-menu-item" @click="handleAnalyzeImpact">
          影响分析
        </div>
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.topology-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--im-surface-1);
  position: relative;
}
.path-trace-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  background: var(--im-accent-soft);
  border-bottom: 1px solid var(--im-border-active);
  font-size: 13px;
  color: var(--im-accent);
  flex-shrink: 0;
}
.topology-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}
.canvas-wrapper {
  flex: 1;
  position: relative;
  min-width: 0;
}
.empty-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(10, 15, 26, 0.45);
  pointer-events: none;
}
.context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--im-surface-1);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-sm);
  box-shadow: var(--im-shadow-md);
  padding: 4px 0;
  min-width: 160px;
}
.context-menu-item {
  padding: 8px 16px;
  font-size: 13px;
  cursor: pointer;
  transition: background var(--im-duration-fast) var(--im-ease-standard);
}
.context-menu-item:hover {
  background: var(--im-surface-2);
  color: var(--im-accent);
}
</style>
