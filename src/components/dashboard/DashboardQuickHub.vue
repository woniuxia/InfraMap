<script setup lang="ts">
import type { PropType } from "vue";
import { Refresh } from "@element-plus/icons-vue";
import type { DashboardQuickAction } from "@/components/dashboard/types";

defineProps({
  primaryActions: {
    type: Array as PropType<DashboardQuickAction[]>,
    default: () => [],
  },
  secondaryActions: {
    type: Array as PropType<DashboardQuickAction[]>,
    default: () => [],
  },
});

const emit = defineEmits<{
  quickAction: [action: DashboardQuickAction];
  refresh: [];
}>();

function onQuickAction(action: DashboardQuickAction) {
  emit("quickAction", action);
}
</script>

<template>
  <el-card class="hero-card" data-testid="quick-hub">
    <div class="hero-header">
      <div>
        <h3 class="hero-title">快捷操作中心</h3>
        <p class="hero-subtitle">优先进入常用页面，快速完成资产录入与维护</p>
      </div>
      <div class="hero-actions">
        <el-button class="refresh-button" type="primary" @click="emit('refresh')">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <div class="hero-primary-actions" data-testid="primary-quick-actions">
      <button
        v-for="action in primaryActions"
        :key="action.key"
        type="button"
        class="quick-action-button quick-action-button-primary"
        :data-testid="`quick-action-${action.key}`"
        @click="onQuickAction(action)"
      >
        <el-icon :size="20">
          <component :is="action.icon" />
        </el-icon>
        <div class="quick-action-text">
          <div class="quick-action-title">{{ action.title }}</div>
          <div class="quick-action-desc">{{ action.desc }}</div>
        </div>
        <el-tag v-if="action.badge" class="quick-action-badge" size="small" effect="plain">
          {{ action.badge }}
        </el-tag>
      </button>
    </div>

    <div class="hero-secondary-actions" data-testid="secondary-quick-actions">
      <button
        v-for="action in secondaryActions"
        :key="action.key"
        type="button"
        class="quick-action-button quick-action-button-secondary"
        :data-testid="`quick-action-${action.key}`"
        @click="onQuickAction(action)"
      >
        <el-icon :size="16">
          <component :is="action.icon" />
        </el-icon>
        <span class="quick-action-secondary-title">{{ action.title }}</span>
        <span class="quick-action-secondary-desc">{{ action.desc }}</span>
      </button>
    </div>
  </el-card>
</template>

<style scoped lang="scss">
.hero-card {
  border: 1px solid var(--im-border-light);
  background:
    linear-gradient(
      140deg,
      color-mix(in srgb, var(--im-surface-1) 82%, transparent) 0%,
      var(--im-surface-0) 68%
    ),
    radial-gradient(circle at 0% 0%, var(--im-accent-soft) 0%, transparent 52%);

  :deep(.el-card__body) {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
}

.hero-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.hero-title {
  margin: 0;
  font-size: 22px;
  font-family: var(--im-font-display);
  color: var(--im-text-primary);
}

.hero-subtitle {
  margin-top: 6px;
  font-size: 13px;
  color: var(--im-text-secondary);
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.refresh-button {
  min-height: 32px;
}

.hero-primary-actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.hero-secondary-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.quick-action-button {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: color-mix(in srgb, var(--im-accent-soft) 22%, var(--im-surface-1));
  color: inherit;
  cursor: pointer;
  min-height: 60px;
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  text-align: left;
  transition:
    border-color var(--im-duration-base) var(--im-ease-standard),
    background-color var(--im-duration-base) var(--im-ease-standard),
    transform var(--im-duration-base) var(--im-ease-standard);
}

.quick-action-button:hover {
  border-color: var(--im-border-active);
  background: var(--im-surface-2);
  transform: translateY(-1px);
}

.quick-action-button:focus-visible {
  outline: 2px solid var(--im-accent);
  outline-offset: 1px;
}

.quick-action-button:active {
  transform: translateY(0);
}

.quick-action-button-primary {
  align-items: flex-start;
}

.quick-action-button-secondary {
  min-height: 44px;
  background: var(--im-surface-1);
}

.quick-action-secondary-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.quick-action-secondary-desc {
  font-size: 12px;
  color: var(--im-text-muted);
}

.quick-action-text {
  min-width: 0;
  flex: 1;
}

.quick-action-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--im-text-primary);
}

.quick-action-desc {
  margin-top: 2px;
  font-size: 12px;
  color: var(--im-text-muted);
}

.quick-action-badge {
  flex-shrink: 0;
}

@media (max-width: 1280px) {
  .hero-primary-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .hero-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .hero-actions {
    width: 100%;
    justify-content: flex-start;
  }

  .hero-primary-actions,
  .hero-secondary-actions {
    grid-template-columns: 1fr;
  }
}
</style>
