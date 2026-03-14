<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Refresh, FullScreen, Download, Search } from "@element-plus/icons-vue";
import type { TopologyGroupKind, TopologyNode } from "@/types";
import { TOPOLOGY_ENV_OPTIONS } from "@/constants/options";

const props = defineProps<{
  nodes: TopologyNode[];
}>();

const emit = defineEmits<{
  (e: "search", payload: { matchIds: string[]; focusId?: string }): void;
  (e: "filter", payload: { nodeKinds: TopologyGroupKind[]; env: "prod" | "test" | "dev" }): void;
  (e: "layout-change", type: "force" | "dagre"): void;
  (e: "export", type: "png" | "svg"): void;
  (e: "refresh"): void;
  (e: "fullscreen"): void;
}>();

const searchText = ref("");
const matchIds = ref<string[]>([]);
const currentMatchIndex = ref(0);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

const searchCountText = computed(() => {
  if (!searchText.value) return "";
  return `${matchIds.value.length > 0 ? currentMatchIndex.value + 1 : 0}/${matchIds.value.length}`;
});

function doSearch() {
  const keyword = searchText.value.trim().toLowerCase();
  if (!keyword) {
    matchIds.value = [];
    currentMatchIndex.value = 0;
    emit("search", { matchIds: [] });
    return;
  }

  matchIds.value = props.nodes
    .filter((node) => {
      const name = node.name.toLowerCase();
      const nodeType = (node.nodeType || "").toLowerCase();
      const extra = node.extra || {};
      const address = ((extra.address as string) || "").toLowerCase();
      const ip = ((extra.ip as string) || "").toLowerCase();
      return (
        name.includes(keyword) ||
        nodeType.includes(keyword) ||
        address.includes(keyword) ||
        ip.includes(keyword)
      );
    })
    .map((node) => node.id);

  currentMatchIndex.value = 0;
  emitSearchResult();
}

function emitSearchResult() {
  const focusId = matchIds.value.length > 0 ? matchIds.value[currentMatchIndex.value] : undefined;
  emit("search", { matchIds: matchIds.value, focusId });
}

function handleSearchInput() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(doSearch, 300);
}

function handleSearchEnter() {
  if (matchIds.value.length === 0) return;
  currentMatchIndex.value = (currentMatchIndex.value + 1) % matchIds.value.length;
  emitSearchResult();
}

watch(
  () => props.nodes,
  () => {
    if (searchText.value) doSearch();
  },
);

const typeOptions = [
  { label: "应用服务", value: "service" },
  { label: "中间件", value: "middleware" },
  { label: "Nginx", value: "nginx" },
];
const selectedTypes = ref<TopologyGroupKind[]>([]);

const envOptions = TOPOLOGY_ENV_OPTIONS;
const selectedEnv = ref<"prod" | "test" | "dev">("prod");

const selectedLayout = ref<"force" | "dagre">("dagre");

function emitFilter() {
  emit("filter", { nodeKinds: selectedTypes.value, env: selectedEnv.value });
}

watch([selectedTypes, selectedEnv], emitFilter, { deep: true, immediate: true });
watch(
  selectedLayout,
  (layout) => {
    emit("layout-change", layout);
  },
  { immediate: true },
);

function handleExport(type: "png" | "svg") {
  emit("export", type);
}
</script>

<template>
  <div class="topology-toolbar">
    <div class="toolbar-left">
      <el-input
        v-model="searchText"
        :prefix-icon="Search"
        placeholder="搜索节点名称/IP..."
        clearable
        size="small"
        class="w-220"
        @input="handleSearchInput"
        @keydown.enter="handleSearchEnter"
        @clear="doSearch"
      >
        <template #append v-if="searchCountText">
          <span class="search-count">{{ searchCountText }}</span>
        </template>
      </el-input>

      <el-divider direction="vertical" />

      <el-checkbox-group v-model="selectedTypes" size="small">
        <el-checkbox-button v-for="opt in typeOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </el-checkbox-button>
      </el-checkbox-group>

      <el-divider direction="vertical" />

      <el-select v-model="selectedEnv" size="small" class="w-96" fit-input-width>
        <el-option
          v-for="opt in envOptions"
          :key="opt.value"
          :label="opt.label"
          :value="opt.value"
        />
      </el-select>
    </div>

    <div class="toolbar-right">
      <el-radio-group v-model="selectedLayout" size="small" class="layout-group">
        <el-radio-button value="force">力导向</el-radio-button>
        <el-radio-button value="dagre">层级</el-radio-button>
      </el-radio-group>

      <el-divider direction="vertical" />

      <el-dropdown size="small" @command="handleExport">
        <el-button size="small" :icon="Download">导出</el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="png">PNG</el-dropdown-item>
            <el-dropdown-item command="svg">SVG</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <el-button size="small" :icon="Refresh" @click="$emit('refresh')">刷新</el-button>
      <el-button size="small" :icon="FullScreen" @click="$emit('fullscreen')" />
    </div>
  </div>
</template>

<style scoped>
.topology-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--im-surface-0);
  border-bottom: 1px solid var(--im-border-subtle);
  flex-shrink: 0;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-count {
  font-size: 12px;
  color: var(--im-text-secondary);
  min-width: 32px;
  text-align: center;
}

.layout-group :deep(.el-radio-button__inner) {
  min-height: 32px;
  border-color: var(--im-border-subtle);
  background: var(--im-surface-1);
  color: var(--im-text-secondary);
}

.layout-group :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: var(--im-accent-soft);
  border-color: var(--im-border-active);
  color: var(--im-accent);
  box-shadow: none;
}

.w-96 {
  width: 96px;
}
</style>
