<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { useEnvStore } from "@/stores/env";
import { ElMessage } from "element-plus";
import type { TopologyEnv } from "@/types";
import {
  Monitor,
  Menu as IconMenu,
  Connection,
  Collection,
  SetUp,
  DataAnalysis,
  Files,
  Share,
  UploadFilled,
  Setting,
  Fold,
  Expand,
  User,
  More,
} from "@element-plus/icons-vue";
import { markRaw, type Component, ref, computed, onMounted, onUnmounted, nextTick } from "vue";

const route = useRoute();
const router = useRouter();
const appStore = useAppStore();
const envStore = useEnvStore();

const ENV_OPTIONS: Array<{
  value: TopologyEnv;
  label: string;
  type: "danger" | "warning" | "info";
}> = [
  { value: "prod", label: "生产", type: "danger" },
  { value: "test", label: "测试", type: "warning" },
  { value: "dev", label: "开发", type: "info" },
];

const envHints: Record<TopologyEnv, string> = {
  prod: "线上正式环境",
  test: "预发布验证",
  dev: "开发调试",
};

const isDropdownOpen = ref(false);
const selectorRef = ref<HTMLButtonElement | null>(null);
const dropdownRef = ref<HTMLDivElement | null>(null);
const dropdownPosition = ref({ top: "0px", left: "0px", width: "0px" });

const currentEnvLabel = computed(() => {
  const option = ENV_OPTIONS.find((opt) => opt.value === envStore.currentEnv);
  return appStore.sidebarCollapsed ? option?.label.charAt(0) : option?.label;
});

function updateDropdownPosition() {
  if (!selectorRef.value) return;
  const rect = selectorRef.value.getBoundingClientRect();
  dropdownPosition.value = {
    top: `${rect.bottom + 6}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
  };
}

function toggleDropdown() {
  isDropdownOpen.value = !isDropdownOpen.value;
  if (isDropdownOpen.value) {
    nextTick(() => {
      updateDropdownPosition();
    });
  }
}

function selectEnv(env: TopologyEnv) {
  envStore.setEnv(env);
  isDropdownOpen.value = false;
  const envLabel = ENV_OPTIONS.find((opt) => opt.value === env)?.label || env;
  ElMessage.success(`已切换到${envLabel}环境`);
}

function handleClickOutside(event: MouseEvent) {
  const target = event.target as Node;
  if (dropdownRef.value?.contains(target) || selectorRef.value?.contains(target)) {
    return;
  }
  isDropdownOpen.value = false;
}

function handleResize() {
  if (isDropdownOpen.value) {
    updateDropdownPosition();
  }
}

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  window.addEventListener("resize", handleResize);
  window.addEventListener("scroll", handleResize, true);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  window.removeEventListener("resize", handleResize);
  window.removeEventListener("scroll", handleResize, true);
});

// 主菜单 7 项
const mainMenuItems: { path: string; name: string; icon: Component }[] = [
  { path: "/", name: "仪表盘", icon: markRaw(DataAnalysis) },
  { path: "/hosts", name: "服务器", icon: markRaw(Monitor) },
  { path: "/systems", name: "系统", icon: markRaw(Collection) },
  { path: "/services", name: "服务", icon: markRaw(IconMenu) },
  { path: "/middlewares", name: "中间件", icon: markRaw(Connection) },
  { path: "/nginx-configs", name: "网关", icon: markRaw(SetUp) },
  { path: "/topology", name: "拓扑图", icon: markRaw(Share) },
];

// "更多"子菜单分组
interface MenuGroup {
  title: string;
  items: { path: string; name: string; icon: Component }[];
}

const moreMenuGroups: MenuGroup[] = [
  {
    title: "资源管理",
    items: [
      { path: "/ip-addresses", name: "IP地址", icon: markRaw(Connection) },
      { path: "/contacts", name: "联系人", icon: markRaw(User) },
    ],
  },
  {
    title: "工具箱",
    items: [
      { path: "/import-workbench", name: "批量录入", icon: markRaw(UploadFilled) },
      { path: "/jobs", name: "任务中心", icon: markRaw(Files) },
      { path: "/integrity-center", name: "数据健康", icon: markRaw(DataAnalysis) },
    ],
  },
];

// 底部独立项
const settingsItem = { path: "/settings", name: "系统设置", icon: markRaw(Setting) };

// 判断系统设置是否激活
const isSettingsActive = computed(() => route.path === settingsItem.path);

// 导航到系统设置
function navigateToSettings() {
  router.push(settingsItem.path);
}
</script>

<template>
  <div class="sidebar-container">
    <div class="sidebar-header">
      <span v-if="!appStore.sidebarCollapsed" class="logo-text">InfraMap</span>
      <span v-else class="logo-icon">IM</span>
    </div>
    <div class="env-selector-wrapper">
      <button
        class="env-selector"
        :class="[`env-${envStore.currentEnv}`, { 'is-collapsed': appStore.sidebarCollapsed }]"
        @click="toggleDropdown"
        ref="selectorRef"
      >
        <span class="env-indicator"></span>
        <span class="env-label">{{ currentEnvLabel }}</span>
        <svg
          class="env-chevron"
          :class="{ 'is-open': isDropdownOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path d="M6 9l6 6 6-6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <Teleport to="body">
        <Transition name="env-dropdown">
          <div
            v-if="isDropdownOpen"
            class="env-dropdown-portal"
            :style="dropdownPosition"
            ref="dropdownRef"
          >
            <div class="env-dropdown-header">
              <span>选择环境</span>
            </div>
            <div class="env-options">
              <button
                v-for="option in ENV_OPTIONS"
                :key="option.value"
                class="env-option"
                :class="{ 'is-active': envStore.currentEnv === option.value }"
                @click="selectEnv(option.value)"
              >
                <span class="env-option-indicator" :class="`env-${option.value}`"></span>
                <span class="env-option-label">{{ option.label }}</span>
                <span class="env-option-hint">{{ envHints[option.value] }}</span>
                <svg
                  v-if="envStore.currentEnv === option.value"
                  class="env-check"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                >
                  <path
                    d="M20 6L9 17l-5-5"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </button>
            </div>
          </div>
        </Transition>
      </Teleport>
    </div>
    <el-menu
      :default-active="route.path"
      :collapse="appStore.sidebarCollapsed"
      :collapse-transition="false"
      router
      class="main-menu"
    >
      <!-- 资源类菜单项 -->
      <el-menu-item v-for="item in mainMenuItems.slice(0, 6)" :key="item.path" :index="item.path">
        <el-icon><component :is="item.icon" /></el-icon>
        <template #title>{{ item.name }}</template>
      </el-menu-item>

      <!-- 视图类菜单项（拓扑图） -->
      <el-menu-item v-for="item in mainMenuItems.slice(6)" :key="item.path" :index="item.path">
        <el-icon><component :is="item.icon" /></el-icon>
        <template #title>{{ item.name }}</template>
      </el-menu-item>

      <!-- "更多" 子菜单 -->
      <el-sub-menu index="more" popper-class="sidebar-submenu-popper">
        <template #title>
          <el-icon><More /></el-icon>
          <span>更多</span>
        </template>

        <el-menu-item-group v-for="group in moreMenuGroups" :key="group.title" :title="group.title">
          <el-menu-item v-for="item in group.items" :key="item.path" :index="item.path">
            <el-icon><component :is="item.icon" /></el-icon>
            <template #title>{{ item.name }}</template>
          </el-menu-item>
        </el-menu-item-group>
      </el-sub-menu>
    </el-menu>

    <!-- 底部区域：系统设置 + 折叠按钮（单行横向布局） -->
    <div class="sidebar-bottom" :class="{ 'is-collapsed': appStore.sidebarCollapsed }">
      <div
        class="settings-item"
        :class="{ 'is-active': isSettingsActive }"
        @click="navigateToSettings"
      >
        <el-icon><Setting /></el-icon>
        <span v-if="!appStore.sidebarCollapsed" class="settings-text">
          {{ settingsItem.name }}
        </span>
      </div>
      <div class="collapse-btn" @click="appStore.toggleSidebar">
        <el-icon size="18">
          <Fold v-if="!appStore.sidebarCollapsed" />
          <Expand v-else />
        </el-icon>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--im-surface-0);
}
.sidebar-header {
  height: 56px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--im-border);
  font-weight: 600;
}
.logo-text {
  font-family: var(--im-font-display);
  font-size: 18px;
  color: var(--im-accent);
}
.logo-icon {
  font-size: 16px;
  font-weight: 700;
  color: var(--im-accent);
}
.main-menu {
  flex: 1;
  border-right: none;
  overflow-y: auto;

  :deep(.el-menu-item) {
    margin: 3px 8px;
    height: 44px;
    line-height: 44px;
    border-radius: var(--im-radius-sm);
    transition: all var(--im-duration-base) var(--im-ease-standard);
  }

  :deep(.el-menu-item.is-active) {
    background: var(--im-accent-dim);
    color: var(--im-accent);
    position: relative;

    &::before {
      content: "";
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      width: 3px;
      height: 20px;
      border-radius: 0 2px 2px 0;
      background: var(--im-accent);
    }
  }

  // 子菜单标题样式
  :deep(.el-sub-menu__title) {
    margin: 3px 8px;
    height: 44px;
    line-height: 44px;
    border-radius: var(--im-radius-sm);

    .el-sub-menu__icon-arrow {
      display: none;
    }
  }
}

// 底部区域容器
.sidebar-bottom {
  border-top: 1px solid var(--im-border);
  background: var(--im-surface-0);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  height: 52px;
  padding: 0 4px;
  gap: 4px;

  &.is-collapsed {
    flex-direction: column;
    height: auto;
    padding: 4px 0;
    gap: 0;
  }
}

// 系统设置独立项样式
.settings-item {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  height: 44px;
  padding: 0 12px;
  border-radius: var(--im-radius-sm);
  cursor: pointer;
  transition: all var(--im-duration-base) var(--im-ease-standard);
  color: var(--im-text-primary);
  overflow: hidden;

  &:hover {
    background: var(--im-surface-1);
  }

  &.is-active {
    background: var(--im-accent-dim);
    color: var(--im-accent);
    position: relative;

    &::before {
      content: "";
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      width: 3px;
      height: 20px;
      border-radius: 0 2px 2px 0;
      background: var(--im-accent);
    }
  }

  .is-collapsed & {
    flex: none;
    width: 44px;
    justify-content: center;
    padding: 0;
    margin: 0 auto;
  }

  .el-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  &:not(.is-active) .el-icon {
    margin-right: 10px;
  }

  .is-collapsed & .el-icon {
    margin-right: 0;
  }
}

.settings-text {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

// 折叠按钮
.collapse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  border-radius: var(--im-radius-sm);
  cursor: pointer;
  color: var(--im-text-secondary);
  transition: all var(--im-duration-base) var(--im-ease-standard);

  &:hover {
    background: var(--im-surface-1);
    color: var(--im-text-primary);
  }

  .is-collapsed & {
    width: 44px;
    margin: 0 auto;
  }
}

// 子菜单弹出层样式
:deep(.sidebar-submenu-popper) {
  .el-menu-item-group__title {
    font-size: 12px;
    color: var(--im-text-tertiary);
    padding: 8px 16px;
  }

  .el-menu-item {
    height: 40px;
    line-height: 40px;
  }
}

// ===== 环境选择器 - 工业精致风格 =====
.env-selector-wrapper {
  padding: 12px;
  border-bottom: 1px solid var(--im-border-light);
  display: flex;
  justify-content: center;
}

.env-selector {
  position: relative;
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  border: none;
  border-radius: 10px;
  background: var(--im-surface-0);
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  outline: none;

  // 微光边框效果
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.06),
    0 1px 2px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);

  &:hover {
    transform: translateY(-1px);
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.08),
      0 4px 12px rgba(0, 0, 0, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.8);
  }

  &:active {
    transform: translateY(0);
  }

  // 收缩状态
  &.is-collapsed {
    width: 40px;
    padding: 0;
    justify-content: center;

    .env-label {
      display: none;
    }

    .env-chevron {
      display: none;
    }

    .env-indicator {
      position: static;
      transform: none;
    }
  }
}

// 环境指示器 - 脉动效果
.env-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  position: relative;

  &::before {
    content: "";
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    animation: pulse 2s ease-out infinite;
  }
}

@keyframes pulse {
  0% {
    transform: scale(0.5);
    opacity: 0.6;
  }
  50% {
    opacity: 0.3;
  }
  100% {
    transform: scale(2);
    opacity: 0;
  }
}

.env-prod .env-indicator {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  box-shadow:
    0 0 0 1px rgba(239, 68, 68, 0.3),
    0 0 8px rgba(239, 68, 68, 0.4);

  &::before {
    background: rgba(239, 68, 68, 0.4);
  }
}

.env-test .env-indicator {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  box-shadow:
    0 0 0 1px rgba(245, 158, 11, 0.3),
    0 0 8px rgba(245, 158, 11, 0.4);

  &::before {
    background: rgba(245, 158, 11, 0.4);
  }
}

.env-dev .env-indicator {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  box-shadow:
    0 0 0 1px rgba(59, 130, 246, 0.3),
    0 0 8px rgba(59, 130, 246, 0.4);

  &::before {
    background: rgba(59, 130, 246, 0.4);
  }
}

.env-label {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.3px;
  color: var(--im-text-primary);
  text-align: center;
}

.env-chevron {
  width: 16px;
  height: 16px;
  color: var(--im-text-tertiary);
  flex-shrink: 0;
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);

  &.is-open {
    transform: rotate(180deg);
  }
}

// ===== 下拉菜单 - Portal 样式 =====
.env-dropdown-portal {
  position: fixed;
  z-index: 9999;
  background: var(--im-surface-0);
  border-radius: 12px;
  padding: 6px;
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.04),
    0 20px 50px rgba(0, 0, 0, 0.15),
    0 8px 20px rgba(0, 0, 0, 0.08);
  backdrop-filter: blur(12px);
  min-width: 180px;
}

.env-dropdown-header {
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 500;
  color: var(--im-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  border-bottom: 1px solid var(--im-border-light);
  margin-bottom: 4px;
}

.env-options {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.env-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
  position: relative;

  &:hover {
    background: var(--im-surface-1);
  }

  &:active {
    transform: scale(0.98);
  }

  &.is-active {
    background: var(--im-accent-dim);
  }
}

.env-option-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;

  &.env-prod {
    background: linear-gradient(135deg, #ef4444, #dc2626);
    box-shadow: 0 0 0 1px rgba(239, 68, 68, 0.2);
  }

  &.env-test {
    background: linear-gradient(135deg, #f59e0b, #d97706);
    box-shadow: 0 0 0 1px rgba(245, 158, 11, 0.2);
  }

  &.env-dev {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    box-shadow: 0 0 0 1px rgba(59, 130, 246, 0.2);
  }
}

.env-option-label {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--im-text-primary);
}

.env-option-hint {
  font-size: 11px;
  color: var(--im-text-tertiary);
  opacity: 0;
  transform: translateX(-4px);
  transition: all 0.2s ease;
}

.env-option:hover .env-option-hint {
  opacity: 1;
  transform: translateX(0);
}

.env-check {
  width: 14px;
  height: 14px;
  color: var(--im-accent);
}

// 动画
.env-dropdown-enter-active,
.env-dropdown-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.env-dropdown-enter-from,
.env-dropdown-leave-to {
  opacity: 0;
  transform: scale(0.96) translateY(-8px);
}

// 暗色模式适配
:root[data-theme="dark"] {
  .env-selector {
    background: rgba(255, 255, 255, 0.03);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.06),
      0 1px 2px rgba(0, 0, 0, 0.2);

    &:hover {
      background: rgba(255, 255, 255, 0.05);
      box-shadow:
        0 0 0 1px rgba(255, 255, 255, 0.08),
        0 4px 12px rgba(0, 0, 0, 0.3);
    }
  }

  .env-dropdown-portal {
    background: rgba(30, 30, 30, 0.95);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.06),
      0 20px 50px rgba(0, 0, 0, 0.4),
      0 8px 20px rgba(0, 0, 0, 0.2);
  }
}
</style>
