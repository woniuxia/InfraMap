<script setup lang="ts">
import { useRoute } from "vue-router";
import { useAppStore } from "@/stores/app";
import {
  Monitor,
  Menu as IconMenu,
  Connection,
  SetUp,
  DataAnalysis,
  Share,
  Setting,
  Fold,
  Expand,
} from "@element-plus/icons-vue";
import { markRaw, type Component } from "vue";

const route = useRoute();
const appStore = useAppStore();

const menuItems: { path: string; name: string; icon: Component }[] = [
  { path: "/", name: "仪表盘", icon: markRaw(DataAnalysis) },
  { path: "/hosts", name: "服务器", icon: markRaw(Monitor) },
  { path: "/applications", name: "应用服务", icon: markRaw(IconMenu) },
  { path: "/middlewares", name: "中间件", icon: markRaw(Connection) },
  { path: "/nginx-configs", name: "负载均衡", icon: markRaw(SetUp) },
  { path: "/topology", name: "拓扑图", icon: markRaw(Share) },
  { path: "/settings", name: "系统设置", icon: markRaw(Setting) },
];
</script>

<template>
  <div class="sidebar-container">
    <div class="sidebar-header">
      <span v-if="!appStore.sidebarCollapsed" class="logo-text">InfraMap</span>
      <span v-else class="logo-icon">IM</span>
    </div>
    <el-menu
      :default-active="route.path"
      :collapse="appStore.sidebarCollapsed"
      :collapse-transition="false"
      router
    >
      <el-menu-item
        v-for="item in menuItems"
        :key="item.path"
        :index="item.path"
      >
        <el-icon><component :is="item.icon" /></el-icon>
        <template #title>{{ item.name }}</template>
      </el-menu-item>
    </el-menu>
    <div class="sidebar-footer">
      <el-button text @click="appStore.toggleSidebar">
        <el-icon size="18">
          <Fold v-if="!appStore.sidebarCollapsed" />
          <Expand v-else />
        </el-icon>
      </el-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.sidebar-header {
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--el-border-color-light);
  font-weight: 600;
}
.logo-text {
  font-size: 18px;
  color: var(--el-color-primary);
}
.logo-icon {
  font-size: 16px;
  font-weight: 700;
  color: var(--el-color-primary);
}
.el-menu {
  flex: 1;
  border-right: none;
}
.sidebar-footer {
  padding: 8px;
  display: flex;
  justify-content: center;
  border-top: 1px solid var(--el-border-color-light);
}
</style>
