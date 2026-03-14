<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import type {
  SearchFieldConfig,
  SearchFieldValue,
  SearchFieldWidth,
  SearchToolbarFilters,
  SearchToolbarQueryPayload,
} from "@/types/searchToolbar";
import {
  buildSearchToolbarQuery,
  createEmptyFilters,
  isEmptyFilterValue,
} from "@/components/filters/searchToolbar.utils";

interface Props {
  searchText: string;
  filters: SearchToolbarFilters;
  fields: SearchFieldConfig[];
  searchLabel?: string;
  searchPlaceholder?: string;
  searchWidth?: SearchFieldWidth;
  debounceMs?: number;
}

const props = withDefaults(defineProps<Props>(), {
  searchLabel: "内容",
  searchPlaceholder: "搜索...",
  searchWidth: "lg",
  debounceMs: 400,
});

const emit = defineEmits<{
  "update:searchText": [value: string];
  "update:filters": [value: SearchToolbarFilters];
  query: [payload: SearchToolbarQueryPayload];
}>();

const advancedExpanded = ref(false);
let queryTimer: ReturnType<typeof setTimeout> | null = null;

const widthPresetMap: Record<SearchFieldWidth, { xl: number; lg: number; md: number; sm: number }> =
  {
    sm: { xl: 2, lg: 3, md: 2, sm: 1 },
    md: { xl: 3, lg: 4, md: 2, sm: 1 },
    lg: { xl: 4, lg: 8, md: 4, sm: 1 },
  };

const basicFields = computed(() =>
  props.fields.filter((field) => (field.section ?? "basic") === "basic"),
);
const advancedFields = computed(() =>
  props.fields.filter((field) => (field.section ?? "basic") === "advanced"),
);
const hasAdvancedFields = computed(() => advancedFields.value.length > 0);
const activeAdvancedCount = computed(
  () =>
    advancedFields.value.filter((field) => !isEmptyFilterValue(field, props.filters[field.key]))
      .length,
);
const advancedToggleText = computed(() => {
  if (activeAdvancedCount.value > 0) {
    return `高级筛选 (${activeAdvancedCount.value})`;
  }
  return "高级筛选";
});

const hasActiveFilters = computed(() => {
  if (props.searchText.trim().length > 0) {
    return true;
  }
  return props.fields.some((field) => !isEmptyFilterValue(field, props.filters[field.key]));
});

function emitQuery(searchText: string, filters: SearchToolbarFilters) {
  emit("query", buildSearchToolbarQuery(searchText, props.fields, filters));
}

function clearQueryTimer() {
  if (queryTimer) {
    clearTimeout(queryTimer);
    queryTimer = null;
  }
}

function scheduleQuery(searchText: string, filters: SearchToolbarFilters) {
  clearQueryTimer();
  const nextSearchText = searchText;
  const nextFilters = filters;
  queryTimer = setTimeout(() => {
    emitQuery(nextSearchText, nextFilters);
    queryTimer = null;
  }, props.debounceMs);
}

function applySearchText(nextSearchText: string) {
  emit("update:searchText", nextSearchText);
  scheduleQuery(nextSearchText, props.filters);
}

function onSearchEnter(event?: KeyboardEvent) {
  clearQueryTimer();
  const target = event?.target as HTMLInputElement | null;
  const nextSearchText = target?.value ?? props.searchText;
  emit("update:searchText", nextSearchText);
  emitQuery(nextSearchText, props.filters);
}

function onSearchClear() {
  clearQueryTimer();
  emit("update:searchText", "");
  emitQuery("", props.filters);
}

function buildNextFilters(fieldKey: string, value: SearchFieldValue): SearchToolbarFilters {
  return {
    ...props.filters,
    [fieldKey]: value,
  };
}

function onSelectFieldChange(field: SearchFieldConfig, value: SearchFieldValue) {
  const nextFilters = buildNextFilters(field.key, value);
  emit("update:filters", nextFilters);
  clearQueryTimer();
  emitQuery(props.searchText, nextFilters);
}

function onTextFieldInput(field: SearchFieldConfig, value: string) {
  const nextFilters = buildNextFilters(field.key, value);
  emit("update:filters", nextFilters);
  scheduleQuery(props.searchText, nextFilters);
}

function onTextFieldEnter(event: KeyboardEvent, field: SearchFieldConfig) {
  clearQueryTimer();
  const target = event.target as HTMLInputElement | null;
  const nextValue = target?.value ?? "";
  const nextFilters = buildNextFilters(field.key, nextValue);
  emit("update:filters", nextFilters);
  emitQuery(props.searchText, nextFilters);
}

function onTextFieldClear(field: SearchFieldConfig) {
  clearQueryTimer();
  const nextFilters = buildNextFilters(field.key, "");
  emit("update:filters", nextFilters);
  emitQuery(props.searchText, nextFilters);
}

function toggleAdvanced() {
  advancedExpanded.value = !advancedExpanded.value;
}

function resetAll() {
  clearQueryTimer();
  const emptyFilters = createEmptyFilters(props.fields);
  emit("update:searchText", "");
  emit("update:filters", emptyFilters);
  emitQuery("", emptyFilters);
}

function fieldSpanStyle(width: SearchFieldWidth) {
  const preset = widthPresetMap[width];
  return {
    "--span-xl": `${preset.xl}`,
    "--span-lg": `${preset.lg}`,
    "--span-md": `${preset.md}`,
    "--span-sm": `${preset.sm}`,
  };
}

function normalizeFieldValue(field: SearchFieldConfig): SearchFieldValue {
  const rawValue = props.filters[field.key];
  if (field.type === "multi-select") {
    return Array.isArray(rawValue) ? rawValue : [];
  }
  return typeof rawValue === "string" ? rawValue : "";
}

function textFieldValue(field: SearchFieldConfig): string {
  const value = normalizeFieldValue(field);
  return typeof value === "string" ? value : "";
}

function selectFieldValue(field: SearchFieldConfig): string | string[] {
  const value = normalizeFieldValue(field);
  if (field.type === "multi-select") {
    return Array.isArray(value) ? value : [];
  }
  return typeof value === "string" ? value : "";
}

onBeforeUnmount(() => {
  clearQueryTimer();
});
</script>

<template>
  <div class="im-search-toolbar">
    <div class="im-search-grid">
      <div class="im-search-field im-search-field-search" :style="fieldSpanStyle(searchWidth)">
        <span class="im-search-label">{{ searchLabel }}</span>
        <el-input
          :model-value="searchText"
          :placeholder="searchPlaceholder"
          clearable
          @update:model-value="applySearchText"
          @keyup.enter="onSearchEnter"
          @clear="onSearchClear"
        />
      </div>

      <div
        v-for="field in basicFields"
        :key="field.key"
        class="im-search-field"
        :style="fieldSpanStyle(field.width ?? 'md')"
      >
        <span class="im-search-label">{{ field.label }}</span>
        <el-input
          v-if="field.type === 'text'"
          :model-value="textFieldValue(field)"
          :placeholder="field.placeholder || `请输入${field.label}`"
          :clearable="field.clearable ?? true"
          @update:model-value="onTextFieldInput(field, String($event))"
          @keyup.enter="onTextFieldEnter($event, field)"
          @clear="() => onTextFieldClear(field)"
        />
        <el-select
          v-else
          :model-value="selectFieldValue(field)"
          :placeholder="field.placeholder || '全部'"
          :clearable="field.clearable ?? true"
          :multiple="field.type === 'multi-select' || field.multiple"
          collapse-tags
          collapse-tags-tooltip
          :max-collapse-tags="field.maxCollapseTags ?? 1"
          @change="onSelectFieldChange(field, $event as SearchFieldValue)"
        >
          <el-option
            v-for="item in field.options || []"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </div>
    </div>

    <div class="im-search-ops" :class="{ 'im-search-ops--actions-only': !hasAdvancedFields }">
      <el-button v-if="hasAdvancedFields" text type="primary" @click="toggleAdvanced">
        {{ advancedToggleText }}
      </el-button>
      <div class="im-search-actions">
        <slot name="actions" :hasActiveFilters="hasActiveFilters" :reset="resetAll">
          <el-button :disabled="!hasActiveFilters" @click="resetAll">重置筛选</el-button>
        </slot>
      </div>
    </div>

    <el-collapse-transition>
      <div v-show="advancedExpanded" class="im-search-advanced">
        <div class="im-search-grid">
          <div
            v-for="field in advancedFields"
            :key="field.key"
            class="im-search-field"
            :style="fieldSpanStyle(field.width ?? 'md')"
          >
            <span class="im-search-label">{{ field.label }}</span>
            <el-input
              v-if="field.type === 'text'"
              :model-value="textFieldValue(field)"
              :placeholder="field.placeholder || `请输入${field.label}`"
              :clearable="field.clearable ?? true"
              @update:model-value="onTextFieldInput(field, String($event))"
              @keyup.enter="onTextFieldEnter($event, field)"
              @clear="() => onTextFieldClear(field)"
            />
            <el-select
              v-else
              :model-value="selectFieldValue(field)"
              :placeholder="field.placeholder || '全部'"
              :clearable="field.clearable ?? true"
              :multiple="field.type === 'multi-select' || field.multiple"
              collapse-tags
              collapse-tags-tooltip
              :max-collapse-tags="field.maxCollapseTags ?? 1"
              @change="onSelectFieldChange(field, $event as SearchFieldValue)"
            >
              <el-option
                v-for="item in field.options || []"
                :key="item.value"
                :label="item.label"
                :value="item.value"
              />
            </el-select>
          </div>
        </div>
      </div>
    </el-collapse-transition>
  </div>
</template>

<style scoped>
.im-search-toolbar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px;
  border: 1px solid var(--im-border-light);
  border-radius: var(--im-radius-md);
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--im-surface-1) 82%, transparent),
    var(--im-surface-0)
  );
}

.im-search-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: 12px;
  align-items: center;
}

.im-search-field {
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  min-width: 0;
  grid-column: span var(--span-xl, 3);
}

.im-search-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
  text-align: right;
  white-space: nowrap;
}

.im-search-ops {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.im-search-ops--actions-only {
  justify-content: flex-end;
}

.im-search-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.im-search-advanced {
  border-top: 1px solid var(--im-border-subtle);
  padding-top: 12px;
}

@media (max-width: 1279px) {
  .im-search-grid {
    grid-template-columns: repeat(8, minmax(0, 1fr));
  }

  .im-search-field {
    grid-column: span var(--span-lg, 4);
  }
}

@media (max-width: 959px) {
  .im-search-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .im-search-field {
    grid-column: span var(--span-md, 2);
  }
}

@media (max-width: 767px) {
  .im-search-grid {
    grid-template-columns: 1fr;
  }

  .im-search-field {
    grid-column: span var(--span-sm, 1);
  }

  .im-search-ops {
    flex-direction: column;
    align-items: flex-start;
  }

  .im-search-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
