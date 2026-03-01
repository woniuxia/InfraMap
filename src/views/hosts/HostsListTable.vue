<script setup lang="ts">
import type { Host } from "@/types";
import {
  buildHardwareSummary,
  envLabel,
  envTagType,
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

function ipSummary(row: Host) {
  return summarizeIpDisplay(row.ip_display);
}

function detailTags(row: Host) {
  return parseHostTags(row.tags);
}

function hasDetailDescription(row: Host): boolean {
  return Boolean(row.description && row.description.trim().length > 0);
}
</script>

<template>
  <div>
    <el-table :data="props.data" v-loading="props.loading" border stripe class="w-full im-table-fixed-ops">
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
      <el-table-column prop="hostname" label="主机名" min-width="180" align="center" />
      <el-table-column label="IP地址" min-width="240" align="center">
        <template #default="{ row }">
          <span class="ip-summary-text">
            {{ ipSummary(row).primary }}
            <span v-if="ipSummary(row).extraCount > 0" class="ip-summary-extra">
              +{{ ipSummary(row).extraCount }}
            </span>
          </span>
        </template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="90" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
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
        @current-change="(page) => emit('page-change', page)"
        @size-change="(size) => emit('page-size-change', size)"
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

.ip-summary-text {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-family: var(--im-font-mono);
  font-size: 13px;
}

.ip-summary-extra {
  color: var(--im-text-secondary);
  font-size: 12px;
}

.host-expand {
  padding: 12px 16px;
  background:
    linear-gradient(
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
