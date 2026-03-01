<script setup lang="ts">
import { computed } from "vue";
import { CopyDocument } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";

const props = withDefaults(
  defineProps<{
    text?: string;
    placeholder?: string;
    ariaLabel?: string;
  }>(),
  {
    text: "",
    placeholder: "-",
    ariaLabel: "复制文本",
  }
);

const emit = defineEmits<{
  (e: "copied", value: string): void;
  (e: "copy-failed", error: unknown): void;
}>();

const normalizedText = computed(() => props.text.trim());
const canCopy = computed(() => normalizedText.value.length > 0);
const displayText = computed(() => (canCopy.value ? normalizedText.value : props.placeholder));

function fallbackCopyText(value: string): boolean {
  if (typeof document === "undefined" || typeof document.execCommand !== "function") {
    return false;
  }
  const textarea = document.createElement("textarea");
  textarea.value = value;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "fixed";
  textarea.style.top = "-1000px";
  textarea.style.left = "-1000px";
  document.body.appendChild(textarea);
  textarea.focus();
  textarea.select();
  const copied = document.execCommand("copy");
  document.body.removeChild(textarea);
  return copied;
}

async function copyText(value: string) {
  if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(value);
      return;
    } catch {
      // Fallback for WebView contexts where clipboard API may be unavailable.
    }
  }
  if (fallbackCopyText(value)) {
    return;
  }
  throw new Error("copy_failed");
}

async function handleCopy() {
  if (!canCopy.value) return;
  try {
    await copyText(normalizedText.value);
    emit("copied", normalizedText.value);
    ElMessage.success("复制成功");
  } catch (error) {
    emit("copy-failed", error);
    ElMessage.error("复制失败，请重试");
  } finally {
    if (typeof document !== "undefined") {
      // Pointer click should not keep focus, otherwise :focus-within keeps action visible.
      const activeElement = document.activeElement;
      if (activeElement instanceof HTMLElement) {
        activeElement.blur();
      }
    }
  }
}
</script>

<template>
  <div class="im-copyable-cell">
    <span class="im-copyable-cell__text" :title="displayText">{{ displayText }}</span>
    <el-button
      v-if="canCopy"
      link
      type="primary"
      size="small"
      class="im-copyable-cell__btn"
      :aria-label="ariaLabel"
      @click="handleCopy"
    >
      <el-icon class="im-copyable-cell__icon" aria-hidden="true">
        <CopyDocument />
      </el-icon>
    </el-button>
  </div>
</template>

<style scoped lang="scss">
.im-copyable-cell {
  position: relative;
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: center;
  min-height: 24px;
}

.im-copyable-cell__text {
  font-family: var(--im-font-mono);
  color: var(--im-text-regular);
  white-space: nowrap;
}

.im-copyable-cell__btn {
  position: absolute;
  inset-inline-end: 8px;
  inset-block-start: 50%;
  opacity: 0;
  pointer-events: none;
  transform: translateY(calc(-50% + 1px));
  transition:
    opacity var(--im-duration-fast) var(--im-ease-standard),
    transform var(--im-duration-fast) var(--im-ease-standard);
}

.im-copyable-cell:hover .im-copyable-cell__btn,
.im-copyable-cell:focus-within .im-copyable-cell__btn {
  opacity: 1;
  pointer-events: auto;
  transform: translateY(-50%);
}

.im-copyable-cell__btn:focus-visible {
  outline: 2px solid var(--im-accent-dim);
  outline-offset: 1px;
  border-radius: var(--im-radius-sm);
}

.im-copyable-cell__icon {
  font-size: 14px;
}
</style>
