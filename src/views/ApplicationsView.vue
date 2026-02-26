<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import type { Application } from "@/types";
import { listApplications, saveApplication, softDeleteApplication } from "@/api/applications";
import { useResourceList } from "@/composables/useResourceList";
import DeploymentPanel from "@/components/DeploymentPanel.vue";
import DependencyPanel from "@/components/DependencyPanel.vue";

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
} = useResourceList<Application>({
  listFn: listApplications,
  deleteFn: softDeleteApplication,
  entityLabel: "应用服务",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingApp = ref<Partial<Application>>({});
const isEditing = ref(false);
const saveLoading = ref(false);

function onSearch() {
  handleSearch(searchText.value);
}

function openAdd() {
  editingApp.value = { status: "running", env: "prod", type: "backend" };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: Application) {
  editingApp.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

async function handleSave() {
  saveLoading.value = true;
  try {
    await saveApplication(editingApp.value);
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

function typeLabel(type: string) {
  return (
    ({
      frontend: "前端",
      backend: "后端",
      gateway: "网关",
      batch_job: "批处理",
      microservice: "微服务",
      other: "其他",
    } as Record<string, string>)[type] || type
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
        placeholder="搜索服务名/地址..."
        clearable
        style="width: 250px"
        @clear="onSearch"
        @keyup.enter="onSearch"
      />
      <el-select
        placeholder="类型"
        clearable
        style="width: 120px"
        @change="(v: string) => handleFilter('type', v)"
      >
        <el-option label="前端" value="frontend" />
        <el-option label="后端" value="backend" />
        <el-option label="网关" value="gateway" />
        <el-option label="批处理" value="batch_job" />
        <el-option label="微服务" value="microservice" />
        <el-option label="其他" value="other" />
      </el-select>
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
      <el-button type="primary" @click="openAdd">新增应用</el-button>
    </div>

    <el-table :data="data" v-loading="loading" border stripe style="width: 100%">
      <el-table-column prop="name" label="服务名称" min-width="150" />
      <el-table-column prop="type" label="类型" width="100" align="center">
        <template #default="{ row }">{{ typeLabel(row.type) }}</template>
      </el-table-column>
      <el-table-column label="地址" min-width="180">
        <template #default="{ row }">
          {{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}
        </template>
      </el-table-column>
      <el-table-column prop="env" label="环境" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="tech_stack" label="技术栈" width="120" />
      <el-table-column prop="owner" label="负责人" width="100" />
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
      :title="isEditing ? '编辑应用' : '新增应用'"
      size="500px"
    >
      <el-form :model="editingApp" label-width="100px">
        <el-form-item label="服务名称" required>
          <el-input v-model="editingApp.name" placeholder="请输入服务名称" />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="editingApp.type" style="width: 100%">
            <el-option label="前端" value="frontend" />
            <el-option label="后端" value="backend" />
            <el-option label="网关" value="gateway" />
            <el-option label="批处理" value="batch_job" />
            <el-option label="微服务" value="microservice" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="访问地址">
          <el-input v-model="editingApp.address" placeholder="如 192.168.1.100" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="editingApp.port" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>
        <el-form-item label="技术栈">
          <el-input v-model="editingApp.tech_stack" placeholder="如 Java/Spring Boot" />
        </el-form-item>
        <el-form-item label="部署方式">
          <el-input v-model="editingApp.deploy_mode" placeholder="如 Docker/K8s/物理机" />
        </el-form-item>
        <el-form-item label="环境" required>
          <el-select v-model="editingApp.env" style="width: 100%">
            <el-option label="生产" value="prod" />
            <el-option label="开发" value="dev" />
            <el-option label="测试" value="test" />
          </el-select>
        </el-form-item>
        <el-form-item label="Git仓库">
          <el-input v-model="editingApp.git_repo" placeholder="Git仓库地址" />
        </el-form-item>
        <el-form-item label="负责人">
          <el-input v-model="editingApp.owner" placeholder="负责人姓名" />
        </el-form-item>
        <el-form-item label="状态" required>
          <el-select v-model="editingApp.status" style="width: 100%">
            <el-option label="运行中" value="running" />
            <el-option label="已停止" value="stopped" />
            <el-option label="维护中" value="maintenance" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingApp.description" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <DeploymentPanel
        v-if="isEditing && editingApp.id"
        :resource-id="editingApp.id!"
        resource-type="application"
      />
      <DependencyPanel
        v-if="isEditing && editingApp.id"
        :resource-id="editingApp.id!"
        resource-type="application"
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
