<template>
  <el-button
    v-if="state.latestError"
    class="im-error-detail-trigger"
    type="danger"
    plain
    size="small"
    @click="openInfraErrorDetail()"
  >
    查看错误详情
  </el-button>

  <el-dialog v-model="dialogVisible" title="操作失败详情" width="640px" append-to-body>
    <div v-if="detailError" class="im-error-detail-body">
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
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">关闭</el-button>
      <el-button type="primary" plain @click="clearInfraError">清除记录</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useErrorPresenter } from "@/composables/useErrorPresenter";

const { state, openInfraErrorDetail, closeInfraErrorDetail, clearInfraError } = useErrorPresenter();

const detailError = computed(() => state.detailError ?? state.latestError);

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
.im-error-detail-trigger {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 2200;
  box-shadow: var(--im-shadow-sm);
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
  .im-error-detail-trigger {
    right: 12px;
    left: 12px;
    bottom: 12px;
    width: calc(100% - 24px);
  }
}
</style>
