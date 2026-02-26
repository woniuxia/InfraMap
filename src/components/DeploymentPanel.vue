<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import type { Deployment, Host } from "@/types";
import { listDeployments, saveDeployment, softDeleteDeployment } from "@/api/deployments";
import { listHosts } from "@/api/hosts";

const props = defineProps<{
  resourceId: string;
  resourceType: "application" | "middleware" | "nginx";
}>();

const deployments = ref<Deployment[]>([]);
const hosts = ref<Host[]>([]);
const loading = ref(false);
const addVisible = ref(false);
const newDeploy = ref<{ host_id: string; port?: number }>({ host_id: "" });
const saveLoading = ref(false);

async function fetchDeployments() {
  if (!props.resourceId) return;
  loading.value = true;
  try {
    const result = await listDeployments({
      page: 1,
      page_size: 100,
      filters: { resource_id: props.resourceId, resource_type: props.resourceType },
    });
    deployments.value = result.data;
  } catch {
    // error shown by tauriInvoke
  } finally {
    loading.value = false;
  }
}

async function fetchHosts() {
  try {
    const result = await listHosts({ page: 1, page_size: 999 });
    hosts.value = result.data;
  } catch {
    // ignore
  }
}

function hostName(hostId: string) {
  const h = hosts.value.find((h) => h.id === hostId);
  return h ? `${h.hostname} (${h.ip_address})` : hostId;
}

function openAdd() {
  newDeploy.value = { host_id: "", port: undefined };
  addVisible.value = true;
}

async function handleAdd() {
  if (!newDeploy.value.host_id) {
    ElMessage.warning("请选择目标服务器");
    return;
  }
  saveLoading.value = true;
  try {
    await saveDeployment({
      id: "",
      resource_id: props.resourceId,
      resource_type: props.resourceType,
      host_id: newDeploy.value.host_id,
      port: newDeploy.value.port,
    });
    ElMessage.success("部署关系添加成功");
    addVisible.value = false;
    fetchDeployments();
  } catch {
    // error shown
  } finally {
    saveLoading.value = false;
  }
}

async function handleRemove(dep: Deployment) {
  try {
    await ElMessageBox.confirm("确认删除此部署关系?", "确认", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      type: "warning",
    });
    await softDeleteDeployment(dep.id);
    ElMessage.success("已删除");
    fetchDeployments();
  } catch {
    // cancelled
  }
}

watch(() => props.resourceId, fetchDeployments);
onMounted(() => {
  fetchDeployments();
  fetchHosts();
});
</script>

<template>
  <div class="deployment-panel">
    <div class="panel-header">
      <span class="panel-title">部署关系</span>
      <el-button text type="primary" size="small" @click="openAdd">添加</el-button>
    </div>

    <el-table :data="deployments" v-loading="loading" size="small" stripe max-height="250">
      <el-table-column label="目标服务器" min-width="180">
        <template #default="{ row }">{{ hostName(row.host_id) }}</template>
      </el-table-column>
      <el-table-column prop="port" label="端口" width="80" align="center">
        <template #default="{ row }">{{ row.port || "-" }}</template>
      </el-table-column>
      <el-table-column label="操作" width="70" align="center">
        <template #default="{ row }">
          <el-button text type="danger" size="small" @click="handleRemove(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty
      v-if="!deployments.length && !loading"
      description="暂无部署关系"
      :image-size="40"
    />

    <el-dialog v-model="addVisible" title="添加部署关系" width="400px" append-to-body>
      <el-form :model="newDeploy" label-width="90px">
        <el-form-item label="目标服务器" required>
          <el-select v-model="newDeploy.host_id" filterable placeholder="选择服务器" style="width: 100%">
            <el-option
              v-for="h in hosts"
              :key="h.id"
              :label="`${h.hostname} (${h.ip_address})`"
              :value="h.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="运行端口">
          <el-input-number v-model="newDeploy.port" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleAdd">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.deployment-panel {
  margin-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  padding-top: 12px;
}
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
</style>
