<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";

type CopyState = "idle" | "copied" | "failed";

const props = withDefaults(
  defineProps<{
    text?: string;
    placeholder?: string;
    copyLabel?: string;
    copiedText?: string;
    retryText?: string;
    resetDelayMs?: number;
    ariaLabel?: string;
  }>(),
  {
    text: "",
    placeholder: "-",
    copyLabel: "复制",
    copiedText: "已复制",
    retryText: "重试",
    resetDelayMs: 1200,
    ariaLabel: "复制文本",
  }
);

const emit = defineEmits<{
  (e: "copied", value: string): void;
  (e: "copy-failed", error: unknown): void;
}>();

const copyState = ref<CopyState>("idle");
let resetTimer: ReturnType<typeof setTimeout> | null = null;

const normalizedText = computed(() => props.text.trim());
const canCopy = computed(() => normalizedText.value.length > 0);
const displayText = computed(() => (canCopy.value ? normalizedText.value : props.placeholder));
const buttonText = computed(() => {
  if (copyState.value === "copied") return props.copiedText;
  if (copyState.value === "failed") return props.retryText;
  return props.copyLabel;
});

function clearResetTimer() {
  if (!resetTimer) return;
  clearTimeout(resetTimer);
  resetTimer = null;
}

function scheduleReset() {
  clearResetTimer();
  resetTimer = setTimeout(() => {
    copyState.value = "idle";
    resetTimer = null;
  }, props.resetDelayMs);
}

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
    copyState.value = "copied";
    emit("copied", normalizedText.value);
  } catch (error) {
    copyState.value = "failed";
    emit("copy-failed", error);
  } finally {
    scheduleReset();
  }
}

onBeforeUnmount(() => {
  clearResetTimer();
});
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
      {{ buttonText }}
    </el-button>
  </div>
</template>

<style scoped lang="scss">
.im-copyable-cell {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  max-width: 100%;
}

.im-copyable-cell__text {
  font-family: var(--im-font-mono);
  color: var(--im-text-regular);
  word-break: break-all;
}

.im-copyable-cell__btn {
  opacity: 0;
  pointer-events: none;
  transform: translateY(1px);
  transition:
    opacity var(--im-duration-fast) var(--im-ease-standard),
    transform var(--im-duration-fast) var(--im-ease-standard);
}

.im-copyable-cell:hover .im-copyable-cell__btn,
.im-copyable-cell:focus-within .im-copyable-cell__btn {
  opacity: 1;
  pointer-events: auto;
  transform: translateY(0);
}

.im-copyable-cell__btn:focus-visible {
  outline: 2px solid var(--im-accent-dim);
  outline-offset: 1px;
  border-radius: var(--im-radius-sm);
}
</style>
