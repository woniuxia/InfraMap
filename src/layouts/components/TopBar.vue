<script setup lang="ts">
import { useRoute } from "vue-router";
import { computed } from "vue";
import { Search } from "@element-plus/icons-vue";

const route = useRoute();
const pageTitle = computed(() => (route.meta?.title as string) || "InfraMap");
</script>

<template>
  <div class="topbar">
    <h2 class="page-title">{{ pageTitle }}</h2>
    <div class="topbar-right">
      <el-input
        placeholder="搜索服务名/IP..."
        :prefix-icon="Search"
        class="global-search"
        clearable
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}
.page-title {
  font-family: var(--im-font-display);
  font-size: 18px;
  font-weight: 600;
  color: var(--im-text-primary);
}
.topbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.global-search {
  width: 300px;
}
</style>
