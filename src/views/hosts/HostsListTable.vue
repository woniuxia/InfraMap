<script setup lang="ts">
import { ref } from "vue";
import type { Host } from "@/types";
import {
  buildHardwareSummary,
  parseHostTags,
  statusLabel,
  statusTagType,
  summarizeIpDisplay,
} from "@/views/hosts/hostDisplay";

interface Props {
  data: Host[];
  loading: boolean;
  total: number;
  page: number;
  pageSize: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  edit: [row: Host];
  copy: [row: Host];
  delete: [payload: { id: string; hostname: string }];
  "page-change": [page: number];
  "page-size-change": [size: number];
}>();

const expandedRowKeys = ref<string[]>([]);

function ipSummary(row: Host) {
  return summarizeIpDisplay(row.ip_display);
}

function ipDisplayText(row: Host): string {
  const ips = ipSummary(row).all;
  return ips.length > 0 ? ips.join(", ") : "-";
}

function detailTags(row: Host) {
  return parseHostTags(row.tags);
}

function hasDetailDescription(row: Host): boolean {
  return Boolean(row.description && row.description.trim().length > 0);
}

function isExpanded(rowId: string): boolean {
  return expandedRowKeys.value.includes(rowId);
}

function toggleHostDetail(row: Host) {
  if (isExpanded(row.id)) {
    expandedRowKeys.value = expandedRowKeys.value.filter((id) => id !== row.id);
    return;
  }
  expandedRowKeys.value = [...expandedRowKeys.value, row.id];
}

function handleExpandChange(_row: Host, expandedRows: Host[]) {
  expandedRowKeys.value = expandedRows.map((row) => row.id);
}
</script>

<template>
  <div>
    <el-table
      :data="props.data"
      v-loading="props.loading"
      border
      stripe
      row-key="id"
      :expand-row-keys="expandedRowKeys"
      class="w-full im-table-fixed-ops"
      @expand-change="handleExpandChange"
    >
      <el-table-column type="expand" width="56">
        <template #default="{ row }">
          <div class="host-expand">
            <div class="host-expand-grid">
              <div class="host-expand-item">
                <span class="host-expand-label">完整 IP 列表</span>
                <div class="host-expand-value host-expand-ip-list">
                  <template v-if="ipSummary(row).all.length > 0">
                    <el-tag
                      v-for="ip in ipSummary(row).all"
                      :key="`expand-ip-${row.id}-${ip}`"
                      size="small"
                      effect="plain"
                    >
                      {{ ip }}
                    </el-tag>
                  </template>
                  <span v-else>-</span>
                </div>
              </div>
              <div class="host-expand-item">
                <span class="host-expand-label">操作系统</span>
                <span class="host-expand-value">{{ row.os_type || "-" }}</span>
              </div>
              <div class="host-expand-item">
                <span class="host-expand-label">硬件规格</span>
                <span class="host-expand-value">{{ buildHardwareSummary(row) }}</span>
              </div>
              <div class="host-expand-item">
                <span class="host-expand-label">标签</span>
                <div class="host-expand-value host-expand-tags">
                  <template v-if="detailTags(row).length > 0">
                    <el-tag
                      v-for="tag in detailTags(row)"
                      :key="`expand-tag-${row.id}-${tag}`"
                      size="small"
                      effect="plain"
                    >
                      {{ tag }}
                    </el-tag>
                  </template>
                  <span v-else>-</span>
                </div>
              </div>
              <div class="host-expand-item host-expand-item-full">
                <span class="host-expand-label">备注描述</span>
                <span class="host-expand-value">
                  {{ hasDetailDescription(row) ? row.description : "-" }}
                </span>
              </div>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="hostname" label="主机名" min-width="180" align="center">
        <template #default="{ row }">
          <button
            type="button"
            class="host-name-toggle"
            :aria-expanded="isExpanded(row.id)"
            @click.stop="toggleHostDetail(row)"
          >
            {{ row.hostname }}
          </button>
        </template>
      </el-table-column>
      <el-table-column label="IP地址" min-width="240" align="center">
        <template #default="{ row }">
          <span class="ip-summary-text">{{ ipDisplayText(row) }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="100" align="center">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)" size="small">
            {{ statusLabel(row.status) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="210" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="emit('edit', row)">编辑</el-button>
          <el-button text type="primary" size="small" @click="emit('copy', row)">复制</el-button>
          <el-button
            text
            type="danger"
            size="small"
            @click="emit('delete', { id: row.id, hostname: row.hostname })"
          >
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrapper">
      <el-pagination
        :current-page="props.page"
        :page-size="props.pageSize"
        :total="props.total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        @current-change="emit('page-change', $event)"
        @size-change="emit('page-size-change', $event)"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.host-name-toggle {
  min-height: 32px;
  width: 100%;
  border: 0;
  padding: 4px 8px;
  background: transparent;
  color: var(--im-text-regular);
  cursor: pointer;
  text-align: center;
  line-height: 1.4;
  font: inherit;
  transition:
    color var(--im-duration-fast) var(--im-ease-standard),
    background-color var(--im-duration-fast) var(--im-ease-standard);
}

.host-name-toggle:hover {
  color: var(--im-accent-light);
  background-color: var(--im-accent-soft);
}

.host-name-toggle:active {
  color: var(--im-accent);
  background-color: var(--im-accent-dim);
}

.host-name-toggle:focus-visible {
  outline: 2px solid var(--im-accent-dim);
  outline-offset: 2px;
  border-radius: var(--im-radius-sm);
}

.ip-summary-text {
  display: inline;
  white-space: normal;
  word-break: break-all;
  line-height: 1.5;
  font-family: var(--im-font-mono);
  font-size: 13px;
}

:deep(.im-table-fixed-ops .el-table__expanded-cell) {
  --im-host-expand-trigger-col-width: 56px;
  padding: 12px 16px 12px var(--im-host-expand-trigger-col-width) !important;
}

.host-expand {
  padding: 12px 14px;
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--im-surface-1) 86%, transparent),
    var(--im-surface-0)
  );
  border: 1px solid var(--im-border-subtle);
  border-radius: var(--im-radius-sm);
}

.host-expand-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px 16px;
}

.host-expand-item {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.host-expand-item-full {
  grid-column: 1 / -1;
}

.host-expand-label {
  font-size: 12px;
  color: var(--im-text-secondary);
}

.host-expand-value {
  color: var(--im-text-regular);
  word-break: break-word;
  line-height: 1.4;
}

.host-expand-ip-list,
.host-expand-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

@media (max-width: 768px) {
  .host-expand-grid {
    grid-template-columns: 1fr;
  }
}
</style>
