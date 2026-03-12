<script setup lang="ts">
import { computed } from "vue";
import { Phone, Message, Edit, Delete } from "@element-plus/icons-vue";
import type { Contact } from "@/types";
import ContactAvatar from "./ContactAvatar.vue";

const props = defineProps<{
  contact: Contact;
}>();

const emit = defineEmits<{
  edit: [contact: Contact];
  delete: [contact: Contact];
}>();

const hasContactInfo = computed(() => {
  return !!(props.contact.phone || props.contact.email);
});

function onEdit() {
  emit("edit", props.contact);
}

function onDelete() {
  emit("delete", props.contact);
}
</script>

<template>
  <div class="contact-card">
    <div class="contact-card-header">
      <div class="contact-identity">
        <ContactAvatar :name="contact.name" :size="48" />
        <div class="contact-info">
          <div class="contact-name">{{ contact.name }}</div>
          <div v-if="contact.company" class="contact-company">{{ contact.company }}</div>
        </div>
      </div>
      <div class="contact-actions">
        <el-button link type="primary" size="small" @click="onEdit">
          <el-icon><Edit /></el-icon>
        </el-button>
        <el-button link type="danger" size="small" @click="onDelete">
          <el-icon><Delete /></el-icon>
        </el-button>
      </div>
    </div>

    <div class="contact-divider"></div>

    <div class="contact-details">
      <div v-if="contact.phone" class="detail-item">
        <el-icon class="detail-icon"><Phone /></el-icon>
        <span class="detail-text" :title="contact.phone">{{ contact.phone }}</span>
      </div>
      <div v-if="contact.email" class="detail-item">
        <el-icon class="detail-icon"><Message /></el-icon>
        <span class="detail-text" :title="contact.email">{{ contact.email }}</span>
      </div>
      <div v-if="contact.remark" class="detail-item remark">
        <span class="detail-text" :title="contact.remark">{{ contact.remark }}</span>
      </div>
      <div v-if="!hasContactInfo && !contact.remark" class="detail-item empty">
        <span class="detail-text">暂无联系方式</span>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.contact-card {
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-sm);
  background: var(--im-surface-1);
  padding: 16px;
  transition: all var(--im-duration-base) var(--im-ease-standard);

  &:hover {
    border-color: var(--im-border-active);
    transform: translateY(-2px);
    box-shadow: var(--im-shadow-md);

    .contact-actions {
      opacity: 1;
    }
  }
}

.contact-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.contact-identity {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.contact-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.contact-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--im-text-primary);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-company {
  font-size: 12px;
  color: var(--im-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--im-duration-base) var(--im-ease-standard);

  .el-button {
    padding: 4px;
    height: auto;

    .el-icon {
      font-size: 16px;
    }
  }
}

.contact-divider {
  height: 1px;
  background: var(--im-border-light);
  margin: 12px 0;
}

.contact-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--im-text-regular);

  &.remark {
    color: var(--im-text-secondary);
    font-size: 12px;
    line-height: 1.5;

    .detail-text {
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
  }

  &.empty {
    color: var(--im-text-muted);
    font-style: italic;
  }
}

.detail-icon {
  font-size: 14px;
  color: var(--im-text-muted);
  flex-shrink: 0;
}

.detail-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
