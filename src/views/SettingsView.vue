<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { save, open } from "@tauri-apps/plugin-dialog";
import type {
  UpdateSettingsInput,
  BackupEntry,
  DbPreviewSummary,
  ImportResult,
} from "@/types";
import {
  getSettings,
  updateSettings,
  createBackup,
  listBackups,
  deleteBackup,
  previewRestore,
  restoreBackup,
  exportJson,
  importJson,
} from "@/api/settings";

// --- Settings ---
const settingsLoading = ref(false);
const settingsSaving = ref(false);
const settingsForm = ref<UpdateSettingsInput>({
  auto_backup_enabled: false,
  backup_interval_hours: 24,
  max_backups: 10,
});
const lastBackupTime = ref<string | undefined>();

async function loadSettings() {
  settingsLoading.value = true;
  try {
    const s = await getSettings();
    settingsForm.value = {
      auto_backup_enabled: s.auto_backup_enabled,
      backup_interval_hours: s.backup_interval_hours,
      max_backups: s.max_backups,
    };
    lastBackupTime.value = s.last_backup_time;
  } catch {
    // error shown by tauriInvoke
  } finally {
    settingsLoading.value = false;
  }
}

async function handleSaveSettings() {
  settingsSaving.value = true;
  try {
    await updateSettings(settingsForm.value);
    ElMessage.success("设置已保存");
  } catch {
    // error shown by tauriInvoke
  } finally {
    settingsSaving.value = false;
  }
}

// --- Backups ---
const backups = ref<BackupEntry[]>([]);
const backupsLoading = ref(false);
const backupCreating = ref(false);

async function loadBackups() {
  backupsLoading.value = true;
  try {
    backups.value = await listBackups();
  } catch {
    // error shown by tauriInvoke
  } finally {
    backupsLoading.value = false;
  }
}

async function handleCreateBackup() {
  backupCreating.value = true;
  try {
    const filename = await createBackup();
    ElMessage.success(`备份已创建: ${filename}`);
    loadBackups();
    loadSettings();
  } catch {
    // error shown by tauriInvoke
  } finally {
    backupCreating.value = false;
  }
}

async function handleDeleteBackup(filename: string) {
  try {
    await ElMessageBox.confirm(`确定要删除备份文件 ${filename} 吗？`, "删除确认", {
      type: "warning",
    });
    await deleteBackup(filename);
    ElMessage.success("备份已删除");
    loadBackups();
  } catch {
    // cancelled or error shown by tauriInvoke
  }
}

// --- Preview ---
const previewVisible = ref(false);
const previewData = ref<DbPreviewSummary | null>(null);
const previewFilename = ref("");

async function handlePreview(filename: string) {
  previewFilename.value = filename;
  try {
    previewData.value = await previewRestore(filename);
    previewVisible.value = true;
  } catch {
    // error shown by tauriInvoke
  }
}

// --- Restore ---
async function handleRestore(filename: string) {
  try {
    await ElMessageBox.confirm(
      "恢复将用备份数据覆盖当前数据库，恢复前会自动创建安全备份。确定继续？",
      "恢复确认",
      { type: "warning" }
    );
    await restoreBackup(filename);
    ElMessage.success("数据库已恢复，建议重启应用");
    loadBackups();
  } catch {
    // cancelled or error shown by tauriInvoke
  }
}

// --- Export ---
const exportLoading = ref(false);

async function handleExport() {
  try {
    const filepath = await save({
      title: "导出 JSON 数据",
      defaultPath: `inframap_export_${new Date().toISOString().slice(0, 10)}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!filepath) return;
    exportLoading.value = true;
    await exportJson(filepath);
    ElMessage.success("数据已导出");
  } catch {
    // error shown by tauriInvoke
  } finally {
    exportLoading.value = false;
  }
}

// --- Import ---
const importLoading = ref(false);

async function handleImport() {
  try {
    const filepath = await open({
      title: "导入 JSON 数据",
      filters: [{ name: "JSON", extensions: ["json"] }],
      multiple: false,
      directory: false,
    });
    if (!filepath) return;

    await ElMessageBox.confirm(
      "导入将清除当前所有数据并替换为导入文件中的数据，导入前会自动创建安全备份。确定继续？",
      "导入确认",
      { type: "warning" }
    );

    importLoading.value = true;
    const result: ImportResult = await importJson(filepath);
    ElMessage.success("数据导入成功");
    ElMessageBox.alert(
      `导入完成：\n- 服务器: ${result.hosts_imported}\n- 应用: ${result.applications_imported}\n- 中间件: ${result.middlewares_imported}\n- Nginx: ${result.nginx_configs_imported}\n- 部署: ${result.deployments_imported}\n- 依赖: ${result.dependencies_imported}`,
      "导入结果"
    );
  } catch {
    // cancelled or error shown by tauriInvoke
  } finally {
    importLoading.value = false;
  }
}

// --- Helpers ---
function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

function formatTime(isoStr: string | undefined): string {
  if (!isoStr) return "--";
  try {
    return new Date(isoStr).toLocaleString("zh-CN");
  } catch {
    return isoStr;
  }
}

const intervalOptions = [
  { label: "4 小时", value: 4 },
  { label: "8 小时", value: 8 },
  { label: "12 小时", value: 12 },
  { label: "24 小时", value: 24 },
  { label: "48 小时", value: 48 },
];

onMounted(() => {
  loadSettings();
  loadBackups();
});
</script>

<template>
  <div class="settings-view">
    <!-- 备份设置 -->
    <el-card shadow="never" class="settings-card">
      <template #header>
        <span class="card-title">备份设置</span>
      </template>
      <el-form
        :model="settingsForm"
        label-width="120px"
        v-loading="settingsLoading"
      >
        <el-form-item label="自动备份">
          <el-switch v-model="settingsForm.auto_backup_enabled" />
        </el-form-item>
        <el-form-item label="备份间隔">
          <el-select
            v-model="settingsForm.backup_interval_hours"
            class="w-200"
            :disabled="!settingsForm.auto_backup_enabled"
          >
            <el-option
              v-for="opt in intervalOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="最大备份数">
          <el-input-number
            v-model="settingsForm.max_backups"
            :min="1"
            :max="100"
            class="w-200"
          />
        </el-form-item>
        <el-form-item label="上次备份时间">
          <span class="text-secondary">{{ formatTime(lastBackupTime) }}</span>
        </el-form-item>
        <el-form-item>
          <el-button
            type="primary"
            :loading="settingsSaving"
            @click="handleSaveSettings"
          >
            保存设置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 备份管理 -->
    <el-card shadow="never" class="settings-card">
      <template #header>
        <div class="card-header-row">
          <span class="card-title">备份管理</span>
          <el-button
            type="primary"
            size="small"
            :loading="backupCreating"
            @click="handleCreateBackup"
          >
            立即备份
          </el-button>
        </div>
      </template>
      <el-table
        :data="backups"
        v-loading="backupsLoading"
        border
        stripe
        class="w-full"
        empty-text="暂无备份"
      >
        <el-table-column prop="filename" label="文件名" min-width="280" />
        <el-table-column label="大小" width="100" align="center">
          <template #default="{ row }">
            {{ formatSize(row.file_size) }}
          </template>
        </el-table-column>
        <el-table-column label="时间" width="180" align="center">
          <template #default="{ row }">
            {{ formatTime(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="类型" width="90" align="center">
          <template #default="{ row }">
            <el-tag :type="row.is_auto ? 'info' : 'success'" size="small">
              {{ row.is_auto ? "自动" : "手动" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right" align="center">
          <template #default="{ row }">
            <el-button text type="primary" size="small" @click="handlePreview(row.filename)">
              预览
            </el-button>
            <el-button text type="warning" size="small" @click="handleRestore(row.filename)">
              恢复
            </el-button>
            <el-button text type="danger" size="small" @click="handleDeleteBackup(row.filename)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 数据导入导出 -->
    <el-card shadow="never" class="settings-card">
      <template #header>
        <span class="card-title">数据导入导出</span>
      </template>
      <div class="import-export-actions">
        <div class="action-item">
          <div class="action-desc">
            <strong>导出 JSON</strong>
            <p>将所有有效数据导出为 JSON 文件，可用于数据迁移或外部备份。</p>
          </div>
          <el-button type="primary" :loading="exportLoading" @click="handleExport">
            导出 JSON
          </el-button>
        </div>
        <el-divider />
        <div class="action-item">
          <div class="action-desc">
            <strong>导入 JSON</strong>
            <p>从 JSON 文件导入数据，将覆盖当前所有数据。导入前会自动创建安全备份。</p>
          </div>
          <el-button type="warning" :loading="importLoading" @click="handleImport">
            导入 JSON
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 预览弹窗 -->
    <el-dialog v-model="previewVisible" :title="`备份预览 - ${previewFilename}`" width="500px">
      <template v-if="previewData">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="服务器">{{ previewData.hosts }}</el-descriptions-item>
          <el-descriptions-item label="应用">{{ previewData.applications }}</el-descriptions-item>
          <el-descriptions-item label="中间件">{{ previewData.middlewares }}</el-descriptions-item>
          <el-descriptions-item label="Nginx 配置">{{ previewData.nginx_configs }}</el-descriptions-item>
          <el-descriptions-item label="部署关系">{{ previewData.deployments }}</el-descriptions-item>
          <el-descriptions-item label="依赖关系">{{ previewData.dependencies }}</el-descriptions-item>
          <el-descriptions-item label="Schema 版本">{{ previewData.schema_version }}</el-descriptions-item>
          <el-descriptions-item label="兼容性">
            <el-tag :type="previewData.is_compatible ? 'success' : 'danger'" size="small">
              {{ previewData.is_compatible ? "兼容" : "不兼容" }}
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>
        <div v-if="!previewData.is_compatible" class="preview-warning">
          该备份的 schema 版本高于当前应用版本，恢复可能导致数据异常。
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.settings-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  :deep(.el-card__header) {
    padding: 12px 20px;
  }
}

.card-title {
  font-size: 16px;
  font-weight: 600;
}

.card-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.text-secondary {
  color: var(--im-text-secondary);
}

.preview-warning {
  margin-top: 12px;
  color: var(--im-danger);
}

.import-export-actions {
  .action-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
  }

  .action-desc {
    flex: 1;

    strong {
      font-size: 14px;
    }

    p {
      margin: 4px 0 0;
      font-size: 13px;
      color: var(--im-text-secondary);
    }
  }
}
</style>
