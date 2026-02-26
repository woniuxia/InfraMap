<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import type { Middleware } from "@/types";
import { listMiddlewares, saveMiddleware, softDeleteMiddleware } from "@/api/middlewares";
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
} = useResourceList<Middleware>({
  listFn: listMiddlewares,
  deleteFn: softDeleteMiddleware,
  entityLabel: "中间件",
});

const searchText = ref("");
const drawerVisible = ref(false);
const editingMw = ref<Partial<Middleware>>({});
const isEditing = ref(false);
const saveLoading = ref(false);

function onSearch() {
  handleSearch(searchText.value);
}

function openAdd() {
  editingMw.value = { env: "prod", category: "database" };
  isEditing.value = false;
  drawerVisible.value = true;
}

function openEdit(row: Middleware) {
  editingMw.value = { ...row };
  isEditing.value = true;
  drawerVisible.value = true;
}

async function handleSave() {
  saveLoading.value = true;
  try {
    await saveMiddleware(editingMw.value);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    drawerVisible.value = false;
    fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

function categoryLabel(category: string) {
  return (
    ({
      database: "数据库",
      message_queue: "消息队列",
      cache: "缓存",
      search_engine: "搜索引擎",
      config_center: "配置中心",
      other: "其他",
    } as Record<string, string>)[category] || category
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
        placeholder="搜索名称/地址..."
        clearable
        style="width: 250px"
        @clear="onSearch"
        @keyup.enter="onSearch"
      />
      <el-select
        placeholder="分类"
        clearable
        style="width: 120px"
        @change="(v: string) => handleFilter('category', v)"
      >
        <el-option label="数据库" value="database" />
        <el-option label="消息队列" value="message_queue" />
        <el-option label="缓存" value="cache" />
        <el-option label="搜索引擎" value="search_engine" />
        <el-option label="配置中心" value="config_center" />
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
      <el-button type="primary" @click="openAdd">新增中间件</el-button>
    </div>

    <el-table :data="data" v-loading="loading" border stripe style="width: 100%">
      <el-table-column prop="name" label="实例名称" min-width="150" />
      <el-table-column prop="category" label="分类" width="100" align="center">
        <template #default="{ row }">{{ categoryLabel(row.category) }}</template>
      </el-table-column>
      <el-table-column prop="type" label="类型" width="100" />
      <el-table-column label="地址" min-width="180">
        <template #default="{ row }">
          {{ row.address || "-" }}{{ row.port ? ":" + row.port : "" }}
        </template>
      </el-table-column>
      <el-table-column prop="version" label="版本" width="100" />
      <el-table-column prop="env" label="环境" width="80" align="center">
        <template #default="{ row }">
          <el-tag :type="envTagType(row.env)" size="small">{{ envLabel(row.env) }}</el-tag>
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
      :title="isEditing ? '编辑中间件' : '新增中间件'"
      size="500px"
    >
      <el-form :model="editingMw" label-width="100px">
        <el-form-item label="实例名称" required>
          <el-input v-model="editingMw.name" placeholder="请输入实例名称" />
        </el-form-item>
        <el-form-item label="分类" required>
          <el-select v-model="editingMw.category" style="width: 100%">
            <el-option label="数据库" value="database" />
            <el-option label="消息队列" value="message_queue" />
            <el-option label="缓存" value="cache" />
            <el-option label="搜索引擎" value="search_engine" />
            <el-option label="配置中心" value="config_center" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="类型" required>
          <el-input v-model="editingMw.type" placeholder="如 MySQL/Redis/Kafka" />
        </el-form-item>
        <el-form-item label="连接地址" required>
          <el-input v-model="editingMw.address" placeholder="如 192.168.1.100" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="editingMw.port" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>
        <el-form-item label="版本">
          <el-input v-model="editingMw.version" placeholder="如 8.0.33" />
        </el-form-item>
        <el-form-item label="环境" required>
          <el-select v-model="editingMw.env" style="width: 100%">
            <el-option label="生产" value="prod" />
            <el-option label="开发" value="dev" />
            <el-option label="测试" value="test" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingMw.description" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <DeploymentPanel
        v-if="isEditing && editingMw.id"
        :resource-id="editingMw.id!"
        resource-type="middleware"
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
