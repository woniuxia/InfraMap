<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { save, open } from "@tauri-apps/plugin-dialog";
import type {
  StorageProfile,
  UpdateStoragePathInput,
  UpdateSettingsInput,
  BackupEntry,
  DbPreviewSummary,
} from "@/types";
import {
  getStorageProfile,
  updateStoragePath,
  restartApp,
  getSettings,
  updateSettings,
  createBackup,
  listBackups,
  deleteBackup,
  previewRestore,
  restoreBackup,
} from "@/api/settings";
import { exportSnapshotV2, importSnapshotV2, previewSnapshotV2 } from "@/api/snapshots";

// --- Storage Path ---
const storageLoading = ref(false);
const storageSaving = ref(false);
const storageProfile = ref<StorageProfile | null>(null);
const storageForm = ref<UpdateStoragePathInput>({
  root_path: "",
});

async function loadStorageProfile() {
  storageLoading.value = true;
  try {
    const profile = await getStorageProfile();
    storageProfile.value = profile;
    storageForm.value.root_path = profile.active_root_path;
  } catch {
    // error shown by tauriInvoke
  } finally {
    storageLoading.value = false;
  }
}

async function handleSelectStoragePath() {
  try {
    const selected = await open({
      title: "选择新的数据根目录",
      directory: true,
      multiple: false,
    });
    if (!selected || Array.isArray(selected)) return;
    storageForm.value.root_path = selected;
  } catch {
    // cancelled or error shown by tauriInvoke
  }
}

async function handleSaveStoragePath() {
  const nextRoot = storageForm.value.root_path?.trim();
  if (!nextRoot) {
    ElMessage.warning("请先选择存储根目录");
    return;
  }

  try {
    await ElMessageBox.confirm(
      "修改存储根目录后，应用会迁移数据库和备份目录。部分场景下需要重启应用才能生效，是否继续？",
      "确认修改存储路径",
      { type: "warning" }
    );
  } catch {
    return;
  }

  storageSaving.value = true;
  try {
    const result = await updateStoragePath({ root_path: nextRoot });
    if (!result.restart_required) {
      ElMessage.success("存储路径已更新");
      await loadStorageProfile();
      return;
    }

    ElMessage.success("存储路径已更新，应用即将重启");
    await restartApp();
  } catch {
    // error shown by tauriInvoke
  } finally {
    storageSaving.value = false;
  }
}

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
    ElMessage.success("备份设置已保存");
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
    ElMessage.success(`备份已创建：${filename}`);
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
    await ElMessageBox.confirm(`确认删除备份文件 ${filename} 吗？`, "删除备份", {
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
      "恢复备份会覆盖当前数据库内容。系统会先自动创建恢复前备份，确认继续吗？",
      "确认恢复备份",
      { type: "warning" }
    );
    await restoreBackup(filename);
    ElMessage.success("备份恢复完成");
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
      title: "导出 Snapshot v2 数据",
      defaultPath: `inframap_snapshot_v2_${new Date().toISOString().slice(0, 10)}.json`,
      filters: [{ name: "Snapshot v2", extensions: ["json"] }],
    });
    if (!filepath) return;

    exportLoading.value = true;
    const result = await exportSnapshotV2(filepath);
    ElMessage.success(`Snapshot v2 已导出，共 ${result.total_rows} 行`);
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
      title: "导入 Snapshot v2 数据",
      filters: [{ name: "Snapshot v2", extensions: ["json"] }],
      multiple: false,
      directory: false,
    });
    if (!filepath || Array.isArray(filepath)) return;

    const preview = await previewSnapshotV2(filepath);
    const previewLines = [
      `格式版本：v${preview.manifest.format_version}`,
      `Schema 版本：${preview.manifest.schema_version}`,
      `总记录数：${preview.total_rows}`,
      `兼容性：${preview.compatible ? "兼容" : "不兼容"}`,
      `快照表统计：${preview.snapshot_counts.map((item) => `${item.table}=${item.count}`).join(" / ") || "无"}`,
    ];

    if (preview.warnings.length > 0) {
      previewLines.push(`警告：${preview.warnings.join("；")}`);
    }

    await ElMessageBox.confirm(
      `导入前预检完成：\n\n${previewLines.join("\n")}\n\n导入将覆盖当前数据，执行前会自动创建安全备份。确定继续？`,
      "导入确认",
      { type: "warning" }
    );

    importLoading.value = true;
    const result = await importSnapshotV2(filepath);
    ElMessage.success("Snapshot v2 导入成功");
    ElMessageBox.alert(
      `导入完成：\n- 自动备份：${result.backup_filename}\n- 总记录数：${result.total_rows}\n- 表统计：${result.table_counts.map((item) => `${item.table}=${item.count}`).join(" / ") || "无"}`,
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
  loadStorageProfile();
  loadSettings();
  loadBackups();
});
</script>

<template>
  <div class="settings-view">
    <el-card shadow="never" class="settings-card">
      <template #header>
        <span class="card-title">存储路径</span>
      </template>
      <el-form :model="storageForm" label-width="140px" v-loading="storageLoading">
        <el-form-item label="当前根目录">
          <span class="text-secondary">{{ storageProfile?.active_root_path || "--" }}</span>
        </el-form-item>
        <el-form-item label="数据库文件">
          <span class="text-secondary">{{ storageProfile?.db_path || "--" }}</span>
        </el-form-item>
        <el-form-item label="备份目录">
          <span class="text-secondary">{{ storageProfile?.backup_dir || "--" }}</span>
        </el-form-item>
        <el-form-item label="新的根目录">
          <div class="storage-path-row">
            <el-input
              v-model="storageForm.root_path"
              clearable
              placeholder="请选择或输入新的数据根目录"
            />
            <el-button @click="handleSelectStoragePath">选择目录</el-button>
          </div>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :loading="storageSaving" @click="handleSaveStoragePath">
            保存存储路径
          </el-button>
        </el-form-item>
        <el-form-item>
          <p class="path-tip">
            修改后，系统会在新的根目录下维护 `inframap.db` 和 `backups` 目录。
            为避免数据丢失，请确保目标目录具备读写权限且空间充足。
          </p>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never" class="settings-card">
      <template #header>
        <span class="card-title">备份策略</span>
      </template>
      <el-form :model="settingsForm" label-width="120px" v-loading="settingsLoading">
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
        <el-form-item label="最大保留数">
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
          <el-button type="primary" :loading="settingsSaving" @click="handleSaveSettings">
            保存备份设置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never" class="settings-card">
      <template #header>
        <div class="card-header-row">
          <span class="card-title">备份列表</span>
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
        class="w-full im-table-fixed-ops"
        empty-text="暂无备份记录"
      >
        <el-table-column prop="filename" label="文件名" min-width="280" />
        <el-table-column label="大小" width="100" align="center">
          <template #default="{ row }">
            {{ formatSize(row.file_size) }}
          </template>
        </el-table-column>
        <el-table-column label="创建时间" width="180" align="center">
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

    <el-card shadow="never" class="settings-card">
      <template #header>
        <span class="card-title">Snapshot v2 导入导出</span>
      </template>
      <div class="import-export-actions">
        <div class="action-item">
          <div class="action-desc">
            <strong>导出 Snapshot v2</strong>
            <p>将当前有效数据导出为 Snapshot v2 文件，用于迁移、备份与后续任务追踪。</p>
          </div>
          <el-button type="primary" :loading="exportLoading" @click="handleExport">
            导出 Snapshot v2
          </el-button>
        </div>
        <el-divider />
        <div class="action-item">
          <div class="action-desc">
            <strong>导入 Snapshot v2</strong>
            <p>导入前先执行预检，再由你确认后导入；导入前会自动创建安全备份。</p>
          </div>
          <el-button type="warning" :loading="importLoading" @click="handleImport">
            导入 Snapshot v2
          </el-button>
        </div>
      </div>
    </el-card>

    <el-dialog v-model="previewVisible" :title="`备份预览 - ${previewFilename}`" width="500px">
      <template v-if="previewData">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="主机">{{ previewData.hosts }}</el-descriptions-item>
          <el-descriptions-item label="应用">{{ previewData.applications }}</el-descriptions-item>
          <el-descriptions-item label="中间件">{{ previewData.middlewares }}</el-descriptions-item>
          <el-descriptions-item label="Nginx">{{ previewData.nginx_configs }}</el-descriptions-item>
          <el-descriptions-item label="部署">{{ previewData.deployments }}</el-descriptions-item>
          <el-descriptions-item label="调用关系">{{ previewData.call_relations }}</el-descriptions-item>
          <el-descriptions-item label="Schema 版本">
            {{ previewData.schema_version }}
          </el-descriptions-item>
          <el-descriptions-item label="兼容性">
            <el-tag :type="previewData.is_compatible ? 'success' : 'danger'" size="small">
              {{ previewData.is_compatible ? "兼容" : "不兼容" }}
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>
        <div v-if="!previewData.is_compatible" class="preview-warning">
          当前备份的 schema 版本高于本地应用支持范围，直接恢复可能失败，请先升级应用版本。
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

.storage-path-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;

  :deep(.el-input) {
    flex: 1;
  }
}

.path-tip {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
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
