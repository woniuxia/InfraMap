<script setup lang="ts">
import { computed, ref } from "vue";
import type {
  TopologyFilterState,
  TopologyEdgeType,
} from "@/components/topology/topologyGraph.utils";
import { TOPOLOGY_ENV_LABELS, TOPOLOGY_ENV_ORDER } from "@/components/topology/topologyGraph.utils";
import type { TopologyEnv, TopologyGroupKind, TopologyLegendStats } from "@/types";

const props = defineProps<{
  stats: TopologyLegendStats | null;
  filter: TopologyFilterState;
}>();

const emit = defineEmits<{
  (e: "change", payload: Partial<TopologyFilterState>): void;
}>();

const collapsed = ref(false);

const NODE_KIND_LABELS: Record<TopologyGroupKind, string> = {
  application_service: "应用服务",
  middleware: "中间件",
  nginx: "Nginx",
};

const EDGE_LABELS: Record<TopologyEdgeType, string> = {
  http_call: "HTTP 调用",
  tcp: "TCP",
  mq_produce: "MQ 生产",
  mq_consume: "MQ 消费",
  grpc_call: "gRPC",
  db_query: "数据库查询",
  cache_access: "缓存访问",
};

const currentEnvCount = computed(() => {
  const env = props.filter.env;
  return props.stats?.env_counts.find((item) => item.env === env)?.count || 0;
});

const currentEnvAppCount = computed(() => {
  const env = props.filter.env;
  return props.stats?.env_counts.find((item) => item.env === env)?.app_count || 0;
});

const nodeItems = computed(() => {
  const source = props.stats?.node_type_counts || [];
  return source
    .map((item) => ({
      kind: item.kind === "application" ? "application_service" : item.kind,
      count: item.count,
    }))
    .filter((item): item is { kind: TopologyGroupKind; count: number } => {
      return item.kind === "application_service" || item.kind === "middleware" || item.kind === "nginx";
    });
});

const edgeItems = computed(() => props.stats?.edge_type_counts || []);

function switchEnv(env: TopologyEnv) {
  if (props.filter.env === env) return;
  emit("change", { env });
}

function toggleNodeKind(kind: TopologyGroupKind) {
  const set = new Set(props.filter.nodeKinds);
  if (set.has(kind)) {
    set.delete(kind);
  } else {
    set.add(kind);
  }
  emit("change", { nodeKinds: Array.from(set) });
}

function toggleEdgeKind(kind: string) {
  const typed = kind as TopologyEdgeType;
  const set = new Set(props.filter.edgeTypes);
  if (set.has(typed)) {
    set.delete(typed);
  } else {
    set.add(typed);
  }
  emit("change", { edgeTypes: Array.from(set) });
}

function updateShowAllEdges(value: boolean | string | number) {
  emit("change", { showAllEdges: Boolean(value) });
}
</script>

<template>
  <aside class="topology-legend" :class="{ collapsed }">
    <header class="legend-header">
      <strong class="legend-title">图例与筛选</strong>
      <el-button
        text
        size="small"
        class="legend-collapse-btn"
        @click="collapsed = !collapsed"
      >
        {{ collapsed ? "展开" : "收起" }}
      </el-button>
    </header>

    <div v-if="!collapsed" class="legend-content">
      <section class="legend-group">
        <div class="legend-group-title">
          环境上下文
          <span class="legend-emphasis">{{ TOPOLOGY_ENV_LABELS[filter.env] }}</span>
        </div>

        <div class="env-switch-row" role="group" aria-label="选择环境">
          <button
            v-for="env in TOPOLOGY_ENV_ORDER"
            :key="env"
            type="button"
            class="env-switch-btn"
            :class="[`env-${env}`, { active: filter.env === env }]"
            @click="switchEnv(env)"
          >
            {{ TOPOLOGY_ENV_LABELS[env] }}
          </button>
        </div>

        <div class="metric-list">
          <div class="metric-item">
            <span class="metric-label">当前节点</span>
            <span class="metric-value">{{ currentEnvCount }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">应用服务</span>
            <span class="metric-value">{{ currentEnvAppCount }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">跨环境节点</span>
            <span class="metric-value">{{ stats?.external_node_count || 0 }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">跨环境关系</span>
            <span class="metric-value">{{ stats?.cross_env_edge_count || 0 }}</span>
          </div>
        </div>
      </section>

      <section class="legend-group">
        <div class="legend-group-title">节点类型</div>
        <button
          v-for="item in nodeItems"
          :key="item.kind"
          type="button"
          class="legend-item"
          :class="{ active: filter.nodeKinds.includes(item.kind) }"
          @click="toggleNodeKind(item.kind)"
        >
          <span class="item-label">
            {{ NODE_KIND_LABELS[item.kind] || item.kind }}
          </span>
          <span class="item-count">{{ item.count }}</span>
        </button>
      </section>

      <section class="legend-group">
        <div class="legend-group-title">关系类型</div>
        <button
          v-for="item in edgeItems"
          :key="item.kind"
          type="button"
          class="legend-item"
          :class="{ active: filter.edgeTypes.includes(item.kind as TopologyEdgeType) }"
          @click="toggleEdgeKind(item.kind)"
        >
          <span class="item-label">{{ EDGE_LABELS[item.kind as TopologyEdgeType] || item.kind }}</span>
          <span class="item-count">{{ item.count }}</span>
        </button>

        <div class="switch-row">
          <span>显示全部关系线</span>
          <el-switch
            :model-value="filter.showAllEdges"
            inline-prompt
            active-text="开"
            inactive-text="关"
            @change="updateShowAllEdges"
          />
        </div>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.topology-legend {
  width: 280px;
  border-left: 1px solid var(--im-border-subtle);
  background: linear-gradient(180deg, var(--im-surface-0) 0%, var(--im-surface-1) 100%);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

.topology-legend.collapsed {
  width: 96px;
}

.legend-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--im-border-subtle);
  padding: 10px 12px;
}

.legend-title {
  font-size: 13px;
  color: var(--im-text-primary);
}

.legend-collapse-btn {
  min-height: 32px;
}

.legend-content {
  padding: 10px 12px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.legend-group {
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
  padding: 8px;
  background: var(--im-surface-0);
}

.legend-group-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--im-text-secondary);
  margin-bottom: 8px;
}

.legend-emphasis {
  color: var(--im-accent);
  font-weight: 600;
}

.env-switch-row {
  display: flex;
  gap: 6px;
  margin-bottom: 10px;
}

.env-switch-btn {
  flex: 1;
  min-height: 32px;
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-1);
  color: var(--im-text-secondary);
  cursor: pointer;
  font-size: 12px;
  transition:
    border-color var(--im-duration-fast) var(--im-ease-standard),
    background var(--im-duration-fast) var(--im-ease-standard),
    color var(--im-duration-fast) var(--im-ease-standard);
}

.env-switch-btn:hover {
  background: var(--im-surface-2);
}

.env-switch-btn:active {
  border-color: var(--im-border-active);
}

.env-switch-btn:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}

.env-switch-btn.active {
  color: var(--im-text-primary);
  border-color: var(--im-border-active);
  background: var(--im-accent-soft);
}

.metric-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

.metric-item {
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
  padding: 6px 8px;
  background: var(--im-surface-1);
}

.metric-label {
  display: block;
  font-size: 11px;
  color: var(--im-text-secondary);
}

.metric-value {
  display: block;
  margin-top: 2px;
  font-size: 13px;
  color: var(--im-text-primary);
  font-weight: 600;
}

.legend-item {
  width: 100%;
  min-height: 32px;
  border: 1px solid transparent;
  border-radius: var(--im-radius-sm);
  background: transparent;
  color: var(--im-text-primary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  margin-bottom: 4px;
  cursor: pointer;
  transition:
    background var(--im-duration-fast) var(--im-ease-standard),
    border-color var(--im-duration-fast) var(--im-ease-standard),
    color var(--im-duration-fast) var(--im-ease-standard);
}

.legend-item:last-child {
  margin-bottom: 0;
}

.legend-item:hover {
  background: var(--im-surface-2);
}

.legend-item:active {
  border-color: var(--im-border-active);
}

.legend-item:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}

.legend-item.active {
  border-color: var(--im-border-active);
  background: var(--im-accent-soft);
}

.item-label {
  font-size: 12px;
  text-align: left;
}

.item-count {
  font-size: 12px;
  color: var(--im-text-secondary);
}

.switch-row {
  margin-top: 6px;
  border-top: 1px dashed var(--im-border-subtle);
  padding-top: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--im-text-secondary);
  font-size: 12px;
}

@media (max-width: 1080px) {
  .topology-legend {
    width: 100%;
    border-left: none;
    border-top: 1px solid var(--im-border-subtle);
  }

  .topology-legend.collapsed {
    width: 100%;
  }
}
</style>
