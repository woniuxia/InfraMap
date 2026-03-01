<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { Graph } from '@antv/g6'
import type { GraphData, NodeData, EdgeData, ComboData, IElementEvent } from '@antv/g6'
import type { TopologyGraph, TopologyNode } from '@/types'
import { getMiddlewareIconByType } from '@/utils/middlewareCatalog'

interface FilterConfig {
  types: string[]
  env: string
}

const props = defineProps<{
  graphData: TopologyGraph | null
}>()

const emit = defineEmits<{
  (e: 'node-click', node: TopologyNode): void
  (e: 'node-contextmenu', payload: { node: TopologyNode; x: number; y: number }): void
}>()

const containerRef = ref<HTMLDivElement>()
let graph: Graph | null = null
let resizeObserver: ResizeObserver | null = null
let themeObserver: MutationObserver | null = null

interface GraphTheme {
  statusColors: Record<string, string>
  edgeStyles: Record<string, { stroke: string; lineDash?: number[] }>
  labelPrimary: string
  labelSecondary: string
  labelMuted: string
  labelBg: string
  highlight: string
  impact: string
}

function cssVar(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return value || fallback
}

function withAlpha(hex: string, alphaHex: string): string {
  if (!hex.startsWith('#') || (hex.length !== 7 && hex.length !== 4)) return hex
  if (hex.length === 4) {
    const r = hex[1]
    const g = hex[2]
    const b = hex[3]
    return `#${r}${r}${g}${g}${b}${b}${alphaHex}`
  }
  return `${hex}${alphaHex}`
}

function buildGraphTheme(): GraphTheme {
  const accent = cssVar('--im-accent', '#6fa8ff')
  const success = cssVar('--im-success', '#41c58a')
  const warning = cssVar('--im-warning', '#f2b645')
  const danger = cssVar('--im-danger', '#ef6b73')
  const textPrimary = cssVar('--im-text-primary', '#e6eefc')
  const textSecondary = cssVar('--im-text-secondary', '#93a4c4')
  const textMuted = cssVar('--im-text-muted', '#6f7f9c')
  const surface = cssVar('--im-surface-0', '#0f1728')

  return {
    statusColors: {
      running: success,
      stopped: textMuted,
      maintenance: warning,
    },
    edgeStyles: {
      http_call: { stroke: accent },
      tcp: { stroke: cssVar('--el-color-info', '#5ca3ff') },
      mq_produce: { stroke: warning, lineDash: [4, 4] },
      mq_consume: { stroke: warning, lineDash: [4, 4] },
      grpc_call: { stroke: cssVar('--el-color-primary', '#409eff'), lineDash: [6, 3] },
      db_query: { stroke: cssVar('--el-color-success', '#67c23a') },
      cache_access: { stroke: cssVar('--el-color-warning', '#e6a23c'), lineDash: [2, 4] },
    },
    labelPrimary: textPrimary,
    labelSecondary: textSecondary,
    labelMuted: textMuted,
    labelBg: surface,
    highlight: accent,
    impact: danger,
  }
}

function getNodeType(datum: NodeData, isLargeGraph: boolean): string {
  const nodeType = datum.data?.node_type as string
  if (nodeType === 'middleware') return 'image'
  if (isLargeGraph) return 'circle'
  if (nodeType === 'nginx') return 'hexagon'
  return 'circle' // application
}

function getNodeStyle(datum: NodeData, isLargeGraph: boolean): Record<string, unknown> {
  const theme = buildGraphTheme()
  const nodeType = datum.data?.node_type as string
  const nodeSize = isLargeGraph ? 20 : 32
  const labelFontSize = isLargeGraph ? 9 : 11

  if (nodeType === 'middleware') {
    const extra = (datum.data?.extra as Record<string, unknown> | undefined) ?? {}
    const category = typeof extra.category === 'string' ? extra.category : undefined
    const middlewareType = typeof extra.type === 'string' ? extra.type : undefined
    const icon = getMiddlewareIconByType(middlewareType, category)
    return {
      img: icon.src,
      src: icon.src,
      size: nodeSize,
      labelText: datum.data?.name as string || datum.id,
      labelPlacement: 'bottom',
      labelFontSize,
      labelFill: theme.labelPrimary,
      labelOffsetY: 4,
    }
  }

  const status = datum.data?.status as string
  const fill = theme.statusColors[status] || theme.labelMuted
  return {
    fill,
    stroke: fill,
    lineWidth: 2,
    labelText: datum.data?.name as string || datum.id,
    labelPlacement: 'bottom',
    labelFontSize,
    labelFill: theme.labelPrimary,
    labelOffsetY: 4,
    size: nodeSize,
  }
}

function getEdgeStyle(datum: EdgeData): Record<string, unknown> {
  const theme = buildGraphTheme()
  const edgeType = datum.data?.edge_type as string
  const edgeConf = theme.edgeStyles[edgeType] || theme.edgeStyles.http_call
  return {
    stroke: edgeConf.stroke,
    lineWidth: 1.5,
    lineDash: edgeConf.lineDash || [],
    endArrow: true,
    endArrowSize: 8,
    labelText: datum.data?.label as string || '',
    labelFontSize: 10,
    labelFill: theme.labelSecondary,
    labelBackground: true,
    labelBackgroundFill: withAlpha(theme.labelBg, 'E6'),
    labelBackgroundOpacity: 0.85,
    labelPadding: [2, 4],
  }
}

function getComboStyle(datum: ComboData): Record<string, unknown> {
  const theme = buildGraphTheme()
  const status = datum.data?.status as string
  const color = theme.statusColors[status] || theme.labelMuted
  return {
    fill: color + '10',
    stroke: color,
    lineWidth: 1,
    lineDash: [6, 3],
    collapsedSize: [120, 50],
    labelText: `${datum.data?.label || datum.id}\n${datum.data?.ip || ''}`,
    labelFontSize: 11,
    labelFill: theme.labelSecondary,
  }
}

function transformToG6Data(raw: TopologyGraph): GraphData {
  const nodes: NodeData[] = raw.nodes.map((n) => ({
    id: n.id,
    combo: n.parent_id || undefined,
    data: {
      name: n.name,
      node_type: n.node_type,
      status: n.status,
      env: n.env,
      extra: n.extra,
    },
  }))

  const edges: EdgeData[] = raw.edges.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    data: {
      edge_type: e.edge_type,
      label: e.label,
    },
  }))

  const combos: ComboData[] = raw.combos.map((c) => ({
    id: c.id,
    type: 'rect',
    data: {
      label: c.label,
      ip: c.ip,
      status: c.status,
    },
  }))

  return { nodes, edges, combos }
}

function initGraph() {
  if (!containerRef.value || !props.graphData) return

  const container = containerRef.value
  const { width, height } = container.getBoundingClientRect()
  const theme = buildGraphTheme()

  if (graph) {
    graph.destroy()
    graph = null
  }

  const g6Data = transformToG6Data(props.graphData)
  const nodeCount = g6Data.nodes?.length || 0
  const isLargeGraph = nodeCount > 500

  // Dynamic layout based on node count
  const layoutConfig = isLargeGraph
    ? {
        type: 'fruchterman' as const,
        maxIteration: 300,
        gravity: 5,
        speed: 5,
      }
    : {
        type: 'force' as const,
        preventOverlap: true,
        nodeSize: 50,
        linkDistance: 150,
      }

  graph = new Graph({
    container,
    width: width || 800,
    height: height || 600,
    autoFit: 'view',
    animation: false,
    data: g6Data,
    layout: layoutConfig,
    node: {
      type: (datum: NodeData) => getNodeType(datum, isLargeGraph),
      style: (datum: NodeData) => getNodeStyle(datum, isLargeGraph),
      state: {
        highlight: {
          lineWidth: 3,
          shadowColor: theme.highlight,
          shadowBlur: 10,
        },
        dim: {
          opacity: 0.2,
          labelOpacity: 0.3,
        },
        impact: {
          lineWidth: 3,
          shadowColor: theme.impact,
          shadowBlur: 10,
        },
      },
    },
    edge: {
      type: 'line',
      style: getEdgeStyle,
      state: {
        highlight: {
          lineWidth: 3,
          shadowColor: theme.highlight,
          shadowBlur: 6,
        },
        dim: {
          opacity: 0.15,
        },
      },
    },
    combo: {
      type: 'rect',
      style: getComboStyle,
      state: {
        highlight: {
          lineWidth: 2,
          shadowColor: theme.highlight,
          shadowBlur: 8,
        },
        dim: {
          opacity: 0.2,
        },
      },
    },
    behaviors: ['drag-canvas', 'zoom-canvas', 'drag-element', 'collapse-expand'],
    transforms: ['process-parallel-edges'],
  })

  graph.on('node:click', (evt: IElementEvent) => {
    const id = evt.target?.id as string
    if (!id) return
    const node = props.graphData?.nodes.find((n) => n.id === id)
    if (node) emit('node-click', node)
  })

  graph.on('node:contextmenu', (evt: IElementEvent) => {
    const id = evt.target?.id as string
    if (!id) return
    const node = props.graphData?.nodes.find((n) => n.id === id)
    if (node) {
      emit('node-contextmenu', { node, x: evt.client.x, y: evt.client.y })
    }
  })

  graph.render()
}

watch(
  () => props.graphData,
  () => {
    nextTick(initGraph)
  },
)

onMounted(() => {
  nextTick(initGraph)

  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (graph && containerRef.value) {
        const { width, height } = containerRef.value.getBoundingClientRect()
        graph.resize(width, height)
      }
    })
    resizeObserver.observe(containerRef.value)
  }

  themeObserver = new MutationObserver((mutations) => {
    const changed = mutations.some(
      (m) => m.type === 'attributes' && m.attributeName === 'data-theme',
    )
    if (changed) {
      nextTick(initGraph)
    }
  })
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme'],
  })
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  themeObserver?.disconnect()
  themeObserver = null
  if (graph) {
    graph.destroy()
    graph = null
  }
})

// Exposed methods
function highlightPaths(paths: string[][]) {
  if (!graph) return
  clearHighlight()

  const nodeIds = new Set<string>()
  const edgeIds = new Set<string>()
  paths.forEach((path) => {
    path.forEach((id) => nodeIds.add(id))
    for (let i = 0; i < path.length - 1; i++) {
      // Find matching edge
      const edges = props.graphData?.edges || []
      const edge = edges.find((e) => e.source === path[i] && e.target === path[i + 1])
      if (edge) edgeIds.add(edge.id)
    }
  })

  const allNodes = graph.getNodeData()
  const allEdges = graph.getEdgeData()

  allNodes.forEach((n) => {
    if (nodeIds.has(n.id as string)) {
      graph!.setElementState(n.id, 'highlight')
    } else {
      graph!.setElementState(n.id, 'dim')
    }
  })

  allEdges.forEach((e) => {
    if (e.id && edgeIds.has(e.id as string)) {
      graph!.setElementState(e.id, 'highlight')
    } else if (e.id) {
      graph!.setElementState(e.id, 'dim')
    }
  })
}

function highlightImpact(nodeId: string, result: { affected_nodes: { id: string; depth: number }[] }) {
  if (!graph) return
  clearHighlight()

  const affectedIds = new Set<string>([nodeId])
  result.affected_nodes.forEach((n) => affectedIds.add(n.id))

  const allNodes = graph.getNodeData()
  allNodes.forEach((n) => {
    if (n.id === nodeId) {
      graph!.setElementState(n.id, 'impact')
    } else if (affectedIds.has(n.id as string)) {
      graph!.setElementState(n.id, 'highlight')
    } else {
      graph!.setElementState(n.id, 'dim')
    }
  })

  const allEdges = graph.getEdgeData()
  allEdges.forEach((e) => {
    if (e.id) {
      const srcAffected = affectedIds.has(e.source as string)
      const tgtAffected = affectedIds.has(e.target as string)
      if (srcAffected && tgtAffected) {
        graph!.setElementState(e.id, 'highlight')
      } else {
        graph!.setElementState(e.id, 'dim')
      }
    }
  })
}

function highlightSearch(nodeIds: string[], focusId?: string) {
  if (!graph) return
  clearHighlight()

  const matchSet = new Set(nodeIds)

  const allNodes = graph.getNodeData()
  allNodes.forEach((n) => {
    if (matchSet.has(n.id as string)) {
      graph!.setElementState(n.id, 'highlight')
    } else {
      graph!.setElementState(n.id, 'dim')
    }
  })

  if (focusId) {
    graph.focusElement(focusId, true)
  }
}

function applyFilter(filter: FilterConfig) {
  if (!graph || !props.graphData) return

  const g6Data = transformToG6Data(props.graphData)

  // Filter nodes by type and env
  const filteredNodes = g6Data.nodes!.filter((n) => {
    const nodeType = n.data?.node_type as string
    const category = (n.data?.extra as Record<string, unknown>)?.category as string
    const env = n.data?.env as string

    // Type filter
    if (filter.types.length > 0) {
      let match = false
      if (nodeType === 'application' && filter.types.includes('application')) match = true
      if (nodeType === 'nginx' && filter.types.includes('nginx')) match = true
      if (nodeType === 'middleware') {
        if (filter.types.includes(category || 'other')) match = true
      }
      if (!match) return false
    }

    // Env filter
    if (filter.env && filter.env !== 'all') {
      if (env && env !== filter.env) return false
    }

    return true
  })

  const visibleNodeIds = new Set(filteredNodes.map((n) => n.id))

  // Filter edges where both endpoints are visible
  const filteredEdges = g6Data.edges!.filter(
    (e) => visibleNodeIds.has(e.source as string) && visibleNodeIds.has(e.target as string),
  )

  // Keep combos that have at least one visible node
  const usedCombos = new Set(filteredNodes.map((n) => n.combo).filter(Boolean))
  const filteredCombos = g6Data.combos!.filter((c) => usedCombos.has(c.id))

  graph.setData({ nodes: filteredNodes, edges: filteredEdges, combos: filteredCombos })
  graph.render()
}

function setLayout(type: 'force' | 'dagre') {
  if (!graph) return
  if (type === 'dagre') {
    graph.setLayout({
      type: 'antv-dagre',
      rankdir: 'LR',
      nodesep: 40,
      ranksep: 80,
    })
  } else {
    graph.setLayout({
      type: 'force',
      preventOverlap: true,
      nodeSize: 50,
      linkDistance: 150,
    })
  }
  graph.layout()
}

async function exportImage(_type: 'png' | 'svg'): Promise<string | undefined> {
  if (!graph) return
  // G6 5.x toDataURL only supports raster formats; always export as PNG
  const dataURL = await graph.toDataURL({ type: 'image/png' })
  return dataURL
}

function clearHighlight() {
  if (!graph) return
  const allNodes = graph.getNodeData()
  const allEdges = graph.getEdgeData()
  const allCombos = graph.getComboData()

  allNodes.forEach((n) => graph!.setElementState(n.id, []))
  allEdges.forEach((e) => { if (e.id) graph!.setElementState(e.id, []) })
  allCombos.forEach((c) => graph!.setElementState(c.id, []))
}

defineExpose({
  highlightPaths,
  highlightImpact,
  highlightSearch,
  applyFilter,
  setLayout,
  exportImage,
  clearHighlight,
})
</script>

<template>
  <div ref="containerRef" class="topology-canvas" />
</template>

<style scoped>
.topology-canvas {
  width: 100%;
  height: 100%;
  min-height: 400px;
}
</style>
