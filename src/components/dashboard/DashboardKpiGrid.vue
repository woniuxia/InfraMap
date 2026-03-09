<script setup lang="ts">
import type { PropType } from "vue";
import type { DashboardKpiCard } from "@/components/dashboard/types";

defineProps({
  items: {
    type: Array as PropType<DashboardKpiCard[]>,
    default: () => [],
  },
});

const emit = defineEmits<{
  select: [routeName: string];
}>();

function formatInt(value: number) {
  return value.toLocaleString("zh-CN");
}
</script>

<template>
  <div class="section-intro" data-testid="resource-entry-intro">
    <h4 class="section-title">资源入口</h4>
    <p class="section-subtitle">按资源维度查看存量和覆盖情况</p>
  </div>

  <el-row :gutter="16" class="section-row" data-testid="resource-entry-row">
    <el-col v-for="item in items" :key="item.key" :xs="24" :sm="12" :lg="8">
      <el-card class="kpi-card" shadow="hover">
        <button class="kpi-button" type="button" @click="emit('select', item.routeName)">
          <div class="kpi-icon" :class="`kpi-icon-${item.tone}`">
            <el-icon :size="20">
              <component :is="item.icon" />
            </el-icon>
          </div>
          <div class="kpi-content">
            <div class="kpi-title">{{ item.title }}</div>
            <div class="kpi-value">
              <span>{{ formatInt(item.value) }}</span>
              <span v-if="item.key === 'deployment' || item.key === 'relation'" class="kpi-unit">
                %
              </span>
            </div>
            <div class="kpi-subtitle">{{ item.subtitle }}</div>
          </div>
        </button>
      </el-card>
    </el-col>
  </el-row>
</template>

<style scoped lang="scss">
.section-intro {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.section-title {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--im-text-primary);
}

.section-subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--im-text-secondary);
}

.section-row {
  margin: 0;
}

.kpi-card {
  border: 1px solid var(--im-border-light);
  background: var(--im-surface-0);
  transition:
    transform var(--im-duration-base) var(--im-ease-standard),
    box-shadow var(--im-duration-base) var(--im-ease-standard),
    border-color var(--im-duration-base) var(--im-ease-standard);

  :deep(.el-card__body) {
    padding: 0;
  }

  &:hover {
    border-color: var(--im-border-active);
    box-shadow: var(--im-shadow-md);
    transform: translateY(-2px);
  }
}

.kpi-button {
  border: 0;
  width: 100%;
  min-height: 106px;
  cursor: pointer;
  background: transparent;
  color: inherit;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px;
  text-align: left;
}

.kpi-button:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: -2px;
}

.kpi-button:active {
  transform: translateY(1px);
}

.kpi-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.kpi-icon-primary {
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.kpi-icon-success {
  color: var(--el-color-success);
  background: color-mix(in srgb, var(--el-color-success) 20%, transparent);
}

.kpi-icon-warning {
  color: var(--el-color-warning);
  background: color-mix(in srgb, var(--el-color-warning) 22%, transparent);
}

.kpi-icon-danger {
  color: var(--el-color-danger);
  background: color-mix(in srgb, var(--el-color-danger) 20%, transparent);
}

.kpi-icon-info {
  color: var(--el-color-info);
  background: color-mix(in srgb, var(--el-color-info) 20%, transparent);
}

.kpi-content {
  min-width: 0;
  flex: 1;
}

.kpi-title {
  font-size: 13px;
  color: var(--im-text-secondary);
}

.kpi-value {
  margin-top: 2px;
  font-size: 24px;
  line-height: 1.2;
  font-weight: 700;
  color: var(--im-text-primary);
}

.kpi-unit {
  margin-left: 2px;
  font-size: 15px;
  font-weight: 600;
  color: var(--im-text-secondary);
}

.kpi-subtitle {
  margin-top: 6px;
  font-size: 12px;
  color: var(--im-text-muted);
}

@media (max-width: 768px) {
  .section-intro {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
