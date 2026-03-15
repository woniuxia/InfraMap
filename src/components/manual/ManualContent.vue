<script setup lang="ts">
import { computed, defineAsyncComponent } from "vue";
import { findSectionById } from "@/docs/manual-content";

const props = defineProps<{
  activeSection: string;
}>();

const currentSection = computed(() => findSectionById(props.activeSection));

const currentComponent = computed(() => {
  const componentName = currentSection.value?.component;
  if (!componentName) return null;

  return defineAsyncComponent(() => import(`@/components/manual/sections/${componentName}.vue`));
});
</script>

<template>
  <div class="manual-content">
    <div class="content-wrapper">
      <component :is="currentComponent" v-if="currentComponent" />
      <div v-else class="empty-state">
        <el-icon :size="48" color="var(--im-text-muted)">
          <Document />
        </el-icon>
        <p>请从左侧选择章节</p>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.manual-content {
  flex: 1;
  overflow-y: auto;
  background: var(--im-bg);
}

.content-wrapper {
  max-width: 900px;
  margin: 0 auto;
  padding: 40px 60px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 400px;
  color: var(--im-text-muted);

  p {
    margin-top: 16px;
    font-size: 14px;
  }
}
</style>
