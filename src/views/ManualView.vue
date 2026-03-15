<script setup lang="ts">
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import ManualSidebar from "@/components/manual/ManualSidebar.vue";
import ManualContent from "@/components/manual/ManualContent.vue";

const route = useRoute();
const router = useRouter();

const activeSection = ref<string>((route.query.section as string) || "overview-intro");

function handleSectionChange(sectionId: string) {
  activeSection.value = sectionId;
  router.push({ query: { section: sectionId } });
}

watch(
  () => route.query.section,
  (newSection) => {
    if (newSection && typeof newSection === "string") {
      activeSection.value = newSection;
    }
  },
);
</script>

<template>
  <div class="manual-view">
    <ManualSidebar :active-section="activeSection" @section-change="handleSectionChange" />
    <ManualContent :active-section="activeSection" />
  </div>
</template>

<style scoped lang="scss">
.manual-view {
  display: flex;
  height: 100%;
  background: var(--im-bg);
}
</style>
