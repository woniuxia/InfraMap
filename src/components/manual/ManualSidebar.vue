<script setup lang="ts">
import { ref } from "vue";
import { manualSections, type ManualSection } from "@/docs/manual-content";
import {
  InfoFilled,
  Share,
  FolderOpened,
  Tools,
  Setting,
  QuestionFilled,
} from "@element-plus/icons-vue";
import type { Component } from "vue";

defineProps<{
  activeSection: string;
}>();

const emit = defineEmits<{
  (e: "section-change", sectionId: string): void;
}>();

// 默认展开的分组
const expandedGroups = ref<Set<string>>(new Set(["overview", "topology"]));

const iconMap: Record<string, Component> = {
  InfoFilled,
  Share,
  FolderOpened,
  Tools,
  Setting,
  QuestionFilled,
};

function toggleGroup(groupId: string) {
  if (expandedGroups.value.has(groupId)) {
    expandedGroups.value.delete(groupId);
  } else {
    expandedGroups.value.add(groupId);
  }
}

function selectSection(section: ManualSection) {
  if (section.component) {
    emit("section-change", section.id);
  } else if (section.children && section.children.length > 0) {
    // 如果是分组,展开并选择第一个子项
    expandedGroups.value.add(section.id);
    emit("section-change", section.children[0].id);
  }
}
</script>

<template>
  <div class="manual-sidebar">
    <div class="sidebar-header">
      <h2>使用手册</h2>
    </div>
    <div class="sidebar-content">
      <div v-for="section in manualSections" :key="section.id" class="section-group">
        <!-- 分组标题 -->
        <div
          v-if="section.children"
          class="group-title"
          :class="{ 'is-expanded': expandedGroups.has(section.id) }"
          @click="toggleGroup(section.id)"
        >
          <el-icon v-if="section.icon" class="group-icon">
            <component :is="iconMap[section.icon]" />
          </el-icon>
          <span class="group-label">{{ section.title }}</span>
          <el-icon class="expand-icon">
            <ArrowRight />
          </el-icon>
        </div>

        <!-- 单独章节(无子项) -->
        <div
          v-else
          class="section-item"
          :class="{ 'is-active': activeSection === section.id }"
          @click="selectSection(section)"
        >
          <el-icon v-if="section.icon" class="section-icon">
            <component :is="iconMap[section.icon]" />
          </el-icon>
          <span class="section-label">{{ section.title }}</span>
        </div>

        <!-- 子章节列表 -->
        <Transition name="expand">
          <div v-if="section.children && expandedGroups.has(section.id)" class="children-list">
            <div
              v-for="child in section.children"
              :key="child.id"
              class="section-item"
              :class="{ 'is-active': activeSection === child.id }"
              @click="selectSection(child)"
            >
              <span class="section-label">{{ child.title }}</span>
            </div>
          </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.manual-sidebar {
  width: 260px;
  flex-shrink: 0;
  background: var(--im-surface-0);
  border-right: 1px solid var(--im-border);
  display: flex;
  flex-direction: column;
  height: 100%;
}

.sidebar-header {
  height: 56px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid var(--im-border);

  h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--im-text-primary);
    margin: 0;
  }
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 8px;
}

.section-group {
  margin-bottom: 4px;
}

.group-title {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 12px;
  border-radius: var(--im-radius-sm);
  cursor: pointer;
  transition: all var(--im-duration-base) var(--im-ease-standard);
  color: var(--im-text-primary);
  font-weight: 500;
  font-size: 14px;

  &:hover {
    background: var(--im-surface-1);
  }

  .group-icon {
    font-size: 16px;
    margin-right: 8px;
    color: var(--im-accent);
  }

  .group-label {
    flex: 1;
  }

  .expand-icon {
    font-size: 14px;
    color: var(--im-text-secondary);
    transition: transform var(--im-duration-base) var(--im-ease-standard);
  }

  &.is-expanded .expand-icon {
    transform: rotate(90deg);
  }
}

.children-list {
  padding-left: 12px;
  margin-top: 2px;
}

.section-item {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 12px;
  border-radius: var(--im-radius-sm);
  cursor: pointer;
  transition: all var(--im-duration-base) var(--im-ease-standard);
  color: var(--im-text-secondary);
  font-size: 14px;
  position: relative;

  &:hover {
    background: var(--im-surface-1);
    color: var(--im-text-primary);
  }

  &.is-active {
    background: var(--im-accent-dim);
    color: var(--im-accent);

    &::before {
      content: "";
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      width: 3px;
      height: 16px;
      border-radius: 0 2px 2px 0;
      background: var(--im-accent);
    }
  }

  .section-icon {
    font-size: 16px;
    margin-right: 8px;
  }

  .section-label {
    flex: 1;
  }
}

// 展开/收起动画
.expand-enter-active,
.expand-leave-active {
  transition: all var(--im-duration-base) var(--im-ease-standard);
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 500px;
}
</style>
