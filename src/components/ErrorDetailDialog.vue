<template>
  <button
    v-if="state.collapsedVisible && state.history.length > 0"
    class="im-error-capsule"
    type="button"
    data-testid="error-capsule"
    @click="openInfraErrorDetail(0)"
  >
    {{ state.history.length }} 条错误
  </button>

  <el-dialog v-model="dialogVisible" title="操作失败详情" width="760px" append-to-body>
    <div v-if="detailError" class="im-error-detail-layout">
      <aside class="im-error-list">
        <button
          v-for="(item, index) in state.history"
          :key="`${item.command}-${item.code}-${index}`"
          class="im-error-item"
          :class="{ 'is-active': index === state.activeErrorIndex }"
          type="button"
          data-testid="error-item"
          @click="setActiveError(index)"
        >
          <span class="im-error-item-title">{{ item.message }}</span>
          <span class="im-error-item-meta">{{ item.code }} · {{ item.command }}</span>
        </button>
      </aside>

      <section class="im-error-detail-body">
        <el-alert :title="detailError.message" type="error" :closable="false" show-icon />

        <el-descriptions :column="1" border class="im-error-meta">
          <el-descriptions-item label="错误码">
            <code>{{ detailError.code }}</code>
          </el-descriptions-item>
          <el-descriptions-item label="命令">
            <code>{{ detailError.command }}</code>
          </el-descriptions-item>
          <el-descriptions-item label="是否可重试">
            {{ detailError.retryable ? "是" : "否" }}
          </el-descriptions-item>
        </el-descriptions>

        <el-collapse>
          <el-collapse-item title="技术细节" name="details">
            <pre class="im-error-pre">{{ detailError.details || "无额外技术细节" }}</pre>
          </el-collapse-item>
        </el-collapse>
      </section>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">关闭</el-button>
      <el-button type="primary" plain data-testid="clear-history" @click="clearInfraErrorHistory">
        清空全部
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useErrorPresenter } from "@/composables/useErrorPresenter";

const {
  state,
  openInfraErrorDetail,
  setActiveError,
  closeInfraErrorDetail,
  clearInfraErrorHistory,
} = useErrorPresenter();

const detailError = computed(
  () => state.detailError ?? state.history[state.activeErrorIndex] ?? state.latestError,
);

const dialogVisible = computed({
  get: () => state.detailVisible,
  set: (visible: boolean) => {
    if (!visible) {
      closeInfraErrorDetail();
      return;
    }

    openInfraErrorDetail();
  },
});
</script>

<style scoped>
.im-error-capsule {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 2200;
  min-height: 32px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--im-danger) 42%, transparent);
  background: color-mix(in srgb, var(--im-danger) 16%, var(--im-surface-0));
  color: var(--im-text-primary);
  font-family: var(--im-font-body);
  font-size: 13px;
  line-height: 32px;
  cursor: pointer;
  box-shadow: var(--im-shadow-sm);
  transition:
    transform var(--im-duration-base) var(--im-ease-standard),
    box-shadow var(--im-duration-base) var(--im-ease-standard),
    border-color var(--im-duration-base) var(--im-ease-standard),
    background var(--im-duration-base) var(--im-ease-standard);
}

.im-error-capsule:hover {
  transform: translateY(-1px);
  box-shadow: var(--im-shadow-md);
  border-color: color-mix(in srgb, var(--im-danger) 64%, transparent);
}

.im-error-capsule:active {
  transform: translateY(0);
}

.im-error-capsule:focus-visible {
  outline: 2px solid var(--im-accent-dim);
  outline-offset: 2px;
}

.im-error-detail-layout {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr);
  gap: 16px;
  min-height: 360px;
}

.im-error-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;
  max-height: 420px;
  overflow: auto;
}

.im-error-item {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 6px;
  text-align: left;
  padding: 10px 12px;
  border-radius: var(--im-radius-sm);
  border: 1px solid var(--im-border-light);
  background: var(--im-surface-1);
  color: var(--im-text-regular);
  cursor: pointer;
  transition:
    border-color var(--im-duration-base) var(--im-ease-standard),
    background var(--im-duration-base) var(--im-ease-standard),
    box-shadow var(--im-duration-base) var(--im-ease-standard);
}

.im-error-item:hover {
  border-color: var(--im-border-active);
  background: var(--im-surface-2);
}

.im-error-item:active {
  box-shadow: inset 0 0 0 1px var(--im-border-active);
}

.im-error-item:focus-visible {
  outline: 2px solid var(--im-accent-dim);
  outline-offset: 2px;
}

.im-error-item.is-active {
  border-color: color-mix(in srgb, var(--im-danger) 48%, var(--im-border-active));
  background: color-mix(in srgb, var(--im-danger) 12%, var(--im-surface-1));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--im-danger) 22%, transparent);
}

.im-error-item-title {
  color: var(--im-text-primary);
  line-height: 1.3;
  font-size: 13px;
}

.im-error-item-meta {
  color: var(--im-text-secondary);
  font-size: 12px;
  font-family: var(--im-font-mono);
  line-height: 1.3;
}

.im-error-detail-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.im-error-meta code {
  font-family: var(--im-font-mono);
}

.im-error-pre {
  margin: 0;
  max-height: 220px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
  padding: 12px;
  border-radius: var(--im-radius-sm);
  border: 1px solid var(--im-border-light);
  background: var(--im-surface-1);
  color: var(--im-text-regular);
  font-family: var(--im-font-mono);
  font-size: 12px;
}

@media (max-width: 768px) {
  .im-error-capsule {
    right: 12px;
    left: 12px;
    bottom: 12px;
    width: calc(100% - 24px);
  }

  .im-error-detail-layout {
    grid-template-columns: 1fr;
  }

  .im-error-list {
    max-height: 180px;
    padding-right: 0;
  }
}
</style>
