<script setup lang="ts">
import { computed } from 'vue'
import { WarningFilled } from '@element-plus/icons-vue'
import type { TopologyNode, PathResult, ImpactResult } from '@/types'

const props = defineProps<{
  mode: 'detail' | 'path' | 'impact' | null
  selectedNode: TopologyNode | null
  pathResult: PathResult | null
  impactResult: ImpactResult | null
  nodeNameMap: Record<string, string>
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const visible = computed(() => props.mode !== null)

const NODE_TYPE_LABELS: Record<string, string> = {
  application: '应用',
  middleware: '中间件',
  nginx: '负载均衡',
}

const STATUS_LABELS: Record<string, string> = {
  running: '运行中',
  stopped: '已停止',
  maintenance: '维护中',
}

const ENV_LABELS: Record<string, string> = {
  prod: '生产',
  dev: '开发',
  test: '测试',
}

function getNodeName(id: string): string {
  return props.nodeNameMap[id] || id.substring(0, 8)
}

// Impact: group affected nodes by depth
const impactByDepth = computed(() => {
  if (!props.impactResult) return []
  const depthMap: Record<number, { id: string; name: string; node_type: string }[]> = {}
  props.impactResult.affected_nodes.forEach((n) => {
    if (!depthMap[n.depth]) depthMap[n.depth] = []
    depthMap[n.depth].push(n)
  })
  return Object.entries(depthMap)
    .sort(([a], [b]) => Number(a) - Number(b))
    .map(([depth, nodes]) => ({ depth: Number(depth), nodes }))
})

function extraEntries(node: TopologyNode): { key: string; value: string }[] {
  if (!node.extra) return []
  const entries: { key: string; value: string }[] = []
  const LABELS: Record<string, string> = {
    type: '类型',
    category: '分类',
    tech_stack: '技术栈',
    address: '地址',
    port: '端口',
    version: '版本',
    listen_port: '监听端口',
    strategy: '策略',
  }
  for (const [k, v] of Object.entries(node.extra)) {
    if (v !== undefined && v !== null && v !== '') {
      entries.push({ key: LABELS[k] || k, value: String(v) })
    }
  }
  return entries
}
</script>

<template>
  <transition name="slide">
    <div v-if="visible" class="detail-panel">
      <div class="panel-header">
        <span class="panel-title">
          {{ mode === 'detail' ? '节点详情' : mode === 'path' ? '路径追踪' : '影响分析' }}
        </span>
        <el-button text size="small" @click="emit('close')">关闭</el-button>
      </div>

      <div class="panel-body">
        <!-- Detail mode -->
        <template v-if="mode === 'detail' && selectedNode">
          <el-descriptions :column="1" size="small" border>
            <el-descriptions-item label="名称">{{ selectedNode.name }}</el-descriptions-item>
            <el-descriptions-item label="类型">
              <el-tag size="small">{{ NODE_TYPE_LABELS[selectedNode.node_type] || selectedNode.node_type }}</el-tag>
            </el-descriptions-item>
            <el-descriptions-item v-if="selectedNode.status" label="状态">
              <el-tag
                size="small"
                :type="selectedNode.status === 'running' ? 'success' : selectedNode.status === 'stopped' ? 'info' : 'warning'"
              >
                {{ STATUS_LABELS[selectedNode.status] || selectedNode.status }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item v-if="selectedNode.env" label="环境">
              {{ ENV_LABELS[selectedNode.env] || selectedNode.env }}
            </el-descriptions-item>
            <el-descriptions-item
              v-for="entry in extraEntries(selectedNode)"
              :key="entry.key"
              :label="entry.key"
            >
              {{ entry.value }}
            </el-descriptions-item>
          </el-descriptions>
        </template>

        <!-- Path mode -->
        <template v-if="mode === 'path' && pathResult">
          <div v-if="pathResult.paths.length === 0" class="empty-tip">
            <el-empty description="未找到路径" :image-size="60" />
          </div>
          <template v-else>
            <div class="path-summary">
              找到 {{ pathResult.paths.length }} 条路径
              <el-tag v-if="pathResult.truncated" type="warning" size="small" class="tag-gap-left">
                <el-icon><WarningFilled /></el-icon>
                结果已截断
              </el-tag>
            </div>
            <div
              v-for="(path, idx) in pathResult.paths"
              :key="idx"
              class="path-item"
            >
              <div class="path-index">#{{ idx + 1 }}</div>
              <div class="path-chain">
                <template v-for="(nodeId, nIdx) in path" :key="nodeId">
                  <span class="path-node">{{ getNodeName(nodeId) }}</span>
                  <span v-if="nIdx < path.length - 1" class="path-arrow">-></span>
                </template>
              </div>
            </div>
          </template>
        </template>

        <!-- Impact mode -->
        <template v-if="mode === 'impact' && impactResult">
          <div class="impact-summary">
            <el-statistic title="受影响节点" :value="impactResult.total_count" />
            <el-statistic title="最大层级" :value="impactResult.max_depth" />
          </div>
          <div v-if="impactResult.affected_nodes.length === 0" class="empty-tip">
            <el-empty description="无上游依赖" :image-size="60" />
          </div>
          <template v-else>
            <div v-for="group in impactByDepth" :key="group.depth" class="impact-group">
              <div class="impact-depth-label">第 {{ group.depth }} 层</div>
              <div class="impact-nodes">
                <el-tag
                  v-for="node in group.nodes"
                  :key="node.id"
                  size="small"
                  class="impact-tag"
                >
                  {{ node.name }}
                  <span class="impact-node-type">({{ NODE_TYPE_LABELS[node.node_type] || node.node_type }})</span>
                </el-tag>
              </div>
            </div>
          </template>
        </template>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.detail-panel {
  width: 320px;
  border-left: 1px solid var(--im-border-subtle);
  background: var(--im-surface-0);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex-shrink: 0;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--im-border-subtle);
}
.panel-title {
  font-weight: 600;
  font-size: 14px;
}
.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}
.path-summary {
  display: flex;
  align-items: center;
  font-size: 13px;
  color: var(--im-text-secondary);
  margin-bottom: 12px;
}
.tag-gap-left {
  margin-left: 8px;
}
.path-item {
  padding: 8px 12px;
  background: var(--im-surface-1);
  border-radius: 6px;
  margin-bottom: 8px;
}
.path-index {
  font-size: 12px;
  color: var(--im-text-secondary);
  margin-bottom: 4px;
}
.path-chain {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  font-size: 13px;
}
.path-node {
  color: var(--im-accent);
  font-weight: 500;
}
.path-arrow {
  color: var(--im-text-muted);
  margin: 0 2px;
}
.impact-summary {
  display: flex;
  gap: 24px;
  margin-bottom: 16px;
}
.impact-summary :deep(.el-statistic__head) {
  font-size: 12px;
}
.impact-summary :deep(.el-statistic__content) {
  font-size: 20px;
}
.impact-group {
  margin-bottom: 12px;
}
.impact-depth-label {
  font-size: 12px;
  color: var(--im-text-secondary);
  margin-bottom: 6px;
  font-weight: 600;
}
.impact-nodes {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.impact-tag {
  max-width: 100%;
}
.impact-node-type {
  color: var(--im-text-secondary);
  font-size: 11px;
  margin-left: 2px;
}
.empty-tip {
  padding: 20px 0;
}
.slide-enter-active,
.slide-leave-active {
  transition: transform var(--im-duration-base) var(--im-ease-standard);
}
.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
