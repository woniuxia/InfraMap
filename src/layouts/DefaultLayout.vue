<script setup lang="ts">
import SidebarMenu from "./components/SidebarMenu.vue";
import TopBar from "./components/TopBar.vue";
import { useAppStore } from "@/stores/app";

const appStore = useAppStore();
</script>

<template>
  <el-container class="app-container">
    <el-aside :width="appStore.sidebarCollapsed ? '64px' : '200px'" class="app-sidebar">
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
  transition: width 0.3s;
  border-right: 1px solid var(--el-border-color-light);
  overflow: hidden;
}
.app-header {
  border-bottom: 1px solid var(--el-border-color-light);
  display: flex;
  align-items: center;
  padding: 0 16px;
}
.app-main {
  background-color: var(--el-bg-color-page);
  overflow: auto;
}
</style>
