<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import type { NginxConfig } from "@/types";
import { listNginxConfigs, saveNginxConfig, softDeleteNginxConfig } from "@/api/nginx-configs";
import { useResourceList } from "@/composables/useResourceList";
import DeploymentPanel from "@/components/DeploymentPanel.vue";

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
} = useResourceList<NginxConfig>({
  listFn: listNginxConfigs,
  deleteFn: softDeleteNginxConfig,
  entityLabel: "负载均衡",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingNc = ref<Partial<NginxConfig>>({});
const isEditing = ref(false);
const saveLoading = ref(false);

function onSearch() {
  handleSearch(searchText.value);
}

function openAdd() {
  editingNc.value = { status: "running", env: "prod", strategy: "roundrobin" };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: NginxConfig) {
  editingNc.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

async function handleSave() {
  saveLoading.value = true;
  try {
    await saveNginxConfig(editingNc.value);
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

function strategyLabel(strategy: string) {
  return (
    ({ roundrobin: "轮询", ip_hash: "IP哈希" } as Record<string, string>)[strategy] || strategy || "-"
  );
}

function envLabel(env: string) {
  return (
    ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env
  );
}

function envTagType(env: string) {
  return (
    ({ prod: "danger", dev: "", test: "warning" } as Record<string, string>)[env] || "info"
  );
}

onMounted(() => fetchData());
</script>

<template>
  <div class="resource-view">
    <div class="filter-bar">
      <el-input
        v-model="searchText"
        placeholder="搜索配置名称..."
        clearable
        style="width: 250px"
        @clear="onSearch"
        @keyup.enter="onSearch"
      />
      <el-select
        placeholder="环境"
        clearable
        style="width: 100px"
        @change="(v: string) => handleFilter('env', v)"
      >
        <el-option label="生产" value="prod" />
        <el-option label="开发" value="dev" />
        <el-option label="测试" value="test" />
      </el-select>
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
      <el-select
        placeholder="策略"
        clearable
        style="width: 120px"
        @change="(v: string) => handleFilter('strategy', v)"
      >
        <el-option label="轮询" value="roundrobin" />
        <el-option label="IP哈希" value="ip_hash" />
      </el-select>
      <el-button type="primary" @click="openAdd">新增配置</el-button>
    </div>

    <el-table :data="data" v-loading="loading" border stripe style="width: 100%">
      <el-table-column prop="name" label="配置名称" min-width="180" />
      <el-table-column prop="listen_port" label="监听端口" width="100" align="center">
        <template #default="{ row }">{{ row.listen_port || "-" }}</template>
      </el-table-column>
      <el-table-column prop="strategy" label="负载策略" width="100" align="center">
        <template #default="{ row }">{{ strategyLabel(row.strategy) }}</template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
        </template>
      </el-table-column>
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
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.name)"
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
      :title="isEditing ? '编辑负载均衡' : '新增负载均衡'"
      size="500px"
    >
      <el-form :model="editingNc" label-width="110px">
        <el-form-item label="配置名称" required>
          <el-input v-model="editingNc.name" placeholder="请输入配置名称" />
        </el-form-item>
        <el-form-item label="监听端口">
          <el-input-number
            v-model="editingNc.listen_port"
            :min="1"
            :max="65535"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="负载策略">
          <el-select v-model="editingNc.strategy" style="width: 100%">
            <el-option label="轮询 (roundrobin)" value="roundrobin" />
            <el-option label="IP哈希 (ip_hash)" value="ip_hash" />
          </el-select>
        </el-form-item>
        <el-form-item label="上游服务器">
          <el-input
            v-model="editingNc.upstream_servers"
            type="textarea"
            :rows="4"
            placeholder='JSON数组，如 ["192.168.1.10:8080","192.168.1.11:8080"]'
          />
        </el-form-item>
        <el-form-item label="环境" required>
          <el-select v-model="editingNc.env" style="width: 100%">
            <el-option label="生产" value="prod" />
            <el-option label="开发" value="dev" />
            <el-option label="测试" value="test" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态" required>
          <el-select v-model="editingNc.status" style="width: 100%">
            <el-option label="运行中" value="running" />
            <el-option label="已停止" value="stopped" />
            <el-option label="维护中" value="maintenance" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingNc.description" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <DeploymentPanel
        v-if="isEditing && editingNc.id"
        :resource-id="editingNc.id!"
        resource-type="nginx"
      />
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
