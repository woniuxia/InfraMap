<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  name: string;
  size?: number;
}>();

const emit = defineEmits<{
  click: [];
}>();

const initial = computed(() => {
  if (!props.name) return "?";
  return props.name.charAt(0);
});

const bgColor = computed(() => {
  if (!props.name) return "var(--im-text-muted)";

  const colors = ["#5ca3ff", "#41c58a", "#f2b645", "#ef8f62", "#3ec7c5", "#9a7cff", "#ff7cb3"];

  let hash = 0;
  for (let i = 0; i < props.name.length; i++) {
    hash = props.name.charCodeAt(i) + ((hash << 5) - hash);
  }

  const index = Math.abs(hash) % colors.length;
  return colors[index];
});

const avatarSize = computed(() => props.size || 48);

function onClick() {
  emit("click");
}
</script>

<template>
  <div
    class="contact-avatar"
    :style="{
      backgroundColor: bgColor,
      width: `${avatarSize}px`,
      height: `${avatarSize}px`,
      fontSize: `${avatarSize * 0.45}px`,
    }"
    @click="onClick"
  >
    {{ initial }}
  </div>
</template>

<style scoped lang="scss">
.contact-avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  color: #fff;
  font-weight: 600;
  font-family: var(--im-font-display);
  flex-shrink: 0;
  cursor: default;
  user-select: none;
  transition: transform var(--im-duration-base) var(--im-ease-standard);

  &:hover {
    transform: scale(1.05);
  }
}
</style>
