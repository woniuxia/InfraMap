<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import type { Host } from "@/types";
import { listHosts, saveHost, softDeleteHost } from "@/api/hosts";
import { useResourceList } from "@/composables/useResourceList";

const {
  loading,
  data,
  total,
  queryParams,
  fetchData,
  handleSearch,
  handleFilter,
  handlePageChange,
  handlePageSizeChange,
  handleDelete,
} = useResourceList<Host>({
  listFn: listHosts,
  deleteFn: softDeleteHost,
  entityLabel: "服务器",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingHost = ref<Partial<Host>>({});
const isEditing = ref(false);
const saveLoading = ref(false);

function onSearch() {
  handleSearch(searchText.value);
}

function openAdd() {
  editingHost.value = { status: "running", hostname: "", ip_address: "" };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: Host) {
  editingHost.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

async function handleSave() {
  saveLoading.value = true;
  try {
    await saveHost(editingHost.value);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    drawerVisible.value = false;
    fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

function statusTagType(status: string) {
  return (
    ({ running: "success", stopped: "danger", maintenance: "warning" } as Record<string, string>)[
      status
    ] || "info"
  );
}

function statusLabel(status: string) {
  return (
    ({ running: "运行中", stopped: "已停止", maintenance: "维护中" } as Record<string, string>)[
      status
    ] || status
  );
}

onMounted(() => fetchData());
</script>

<template>
  <div class="resource-view">
    <div class="filter-bar">
      <el-input
        v-model="searchText"
        placeholder="搜索主机名/IP..."
        clearable
        style="width: 250px"
        @clear="onSearch"
        @keyup.enter="onSearch"
      />
      <el-select
        placeholder="状态"
        clearable
        style="width: 120px"
        @change="(v: string) => handleFilter('status', v)"
      >
        <el-option label="运行中" value="running" />
        <el-option label="已停止" value="stopped" />
        <el-option label="维护中" value="maintenance" />
      </el-select>
      <el-button type="primary" @click="openAdd">新增服务器</el-button>
    </div>

    <el-table :data="data" v-loading="loading" border stripe style="width: 100%">
      <el-table-column prop="hostname" label="主机名" min-width="150" />
      <el-table-column prop="ip_address" label="IP地址" width="150" />
      <el-table-column prop="os_type" label="操作系统" width="120" />
      <el-table-column prop="ram_gb" label="内存(GB)" width="100" align="center" />
      <el-table-column prop="disk_gb" label="磁盘(GB)" width="100" align="center" />
      <el-table-column prop="status" label="状态" width="100" align="center">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)" size="small">{{
            statusLabel(row.status)
          }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="150" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="openEdit(row)">编辑</el-button>
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.hostname)"
            >删除</el-button
          >
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrapper">
      <el-pagination
        v-model:current-page="queryParams.page"
        v-model:page-size="queryParams.page_size"
        :total="total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        @current-change="handlePageChange"
        @size-change="handlePageSizeChange"
      />
    </div>

    <el-drawer
      v-model="drawerVisible"
      :title="isEditing ? '编辑服务器' : '新增服务器'"
      size="500px"
    >
      <el-form :model="editingHost" label-width="100px">
        <el-form-item label="主机名" required>
          <el-input v-model="editingHost.hostname" placeholder="请输入主机名" />
        </el-form-item>
        <el-form-item label="IP地址" required>
          <el-input v-model="editingHost.ip_address" placeholder="如 192.168.1.100" />
        </el-form-item>
        <el-form-item label="操作系统">
          <el-input v-model="editingHost.os_type" placeholder="如 CentOS 7.9" />
        </el-form-item>
        <el-form-item label="CPU信息">
          <el-input v-model="editingHost.cpu_info" placeholder="如 4C 2.5GHz" />
        </el-form-item>
        <el-form-item label="内存(GB)">
          <el-input-number v-model="editingHost.ram_gb" :min="0" style="width: 100%" />
        </el-form-item>
        <el-form-item label="磁盘(GB)">
          <el-input-number v-model="editingHost.disk_gb" :min="0" style="width: 100%" />
        </el-form-item>
        <el-form-item label="状态" required>
          <el-select v-model="editingHost.status" style="width: 100%">
            <el-option label="运行中" value="running" />
            <el-option label="已停止" value="stopped" />
            <el-option label="维护中" value="maintenance" />
          </el-select>
        </el-form-item>
        <el-form-item label="标签">
          <el-input
            v-model="editingHost.tags"
            type="textarea"
            :rows="2"
            placeholder='JSON数组，如 ["web","prod"]'
          />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingHost.description" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="drawerVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
      </template>
    </el-drawer>
  </div>
</template>

<style scoped lang="scss">
.resource-view {
  padding: 0;
}
.filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  align-items: center;
}
.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
