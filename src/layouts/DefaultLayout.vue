<script setup lang="ts">
import SidebarMenu from "./components/SidebarMenu.vue";
import TopBar from "./components/TopBar.vue";
import { useAppStore } from "@/stores/app";

const appStore = useAppStore();
</script>

<template>
  <el-container class="app-container">
    <el-aside :width="appStore.sidebarCollapsed ? '68px' : '220px'" class="app-sidebar">
      <SidebarMenu />
    </el-aside>
    <el-container>
      <el-header class="app-header" height="56px">
        <TopBar />
      </el-header>
      <el-main class="app-main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<style scoped lang="scss">
.app-container {
  height: 100vh;
}
.app-sidebar {
  transition: width var(--im-duration-slow) var(--im-ease-standard);
  border-right: 1px solid var(--im-border-light);
  background: var(--im-surface-0);
  overflow: hidden;
}
.app-header {
  border-bottom: 1px solid var(--im-border-light);
  background: var(--im-surface-1);
  display: flex;
  align-items: center;
  padding: 0 16px;
}
.app-main {
  background-color: transparent;
  overflow: auto;
}
</style>
