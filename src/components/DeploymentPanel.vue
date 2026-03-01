<script setup lang="ts">
import { computed, ref, watch, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import type { Deployment, Host, ResourceDeployContext } from "@/types";
import {
  getResourceDeployContext,
  listDeployments,
  saveDeployment,
  softDeleteDeployment,
} from "@/api/deployments";
import { listHosts, saveHost } from "@/api/hosts";
import { listIpAddresses, saveIpAddress } from "@/api/ip-addresses";
import { bindHostIp } from "@/api/host-ip-bindings";

const props = defineProps<{
  resourceId: string;
  resourceType: "application" | "middleware" | "nginx";
  defaultPort?: number;
}>();

const deployments = ref<Deployment[]>([]);
const hosts = ref<Host[]>([]);
const loading = ref(false);
const addVisible = ref(false);
const newDeploy = ref<{ host_id: string; port?: number }>({ host_id: "" });
const saveLoading = ref(false);
const contextLoading = ref(false);
const quickCreateLoading = ref(false);
const hostLocked = ref(false);
const resourceContext = ref<ResourceDeployContext | null>(null);

const hasUnmatchedIp = computed(
  () => Boolean(resourceContext.value?.parsed_ip) && !resourceContext.value?.matched_host_id
);
const parsedIp = computed(() => resourceContext.value?.parsed_ip || "");

function normalizeHostEnv(value?: string | null): Host["env"] {
  if (value === "dev" || value === "test") {
    return value;
  }
  return "prod";
}

function formatTempHostName(date: Date = new Date()) {
  const pad2 = (value: number) => String(value).padStart(2, "0");
  return `temp-host-${date.getFullYear()}${pad2(date.getMonth() + 1)}${pad2(date.getDate())}${pad2(date.getHours())}${pad2(date.getMinutes())}${pad2(date.getSeconds())}`;
}

function splitIpDisplay(value?: string): string[] {
  if (!value) return [];
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

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
  return h ? `${h.hostname} (${h.ip_display || "-"})` : hostId;
}

async function loadResourceDeployContext() {
  contextLoading.value = true;
  try {
    const context = await getResourceDeployContext(props.resourceType, props.resourceId);
    resourceContext.value = context;
    if (context.matched_host_id) {
      newDeploy.value.host_id = context.matched_host_id;
      hostLocked.value = true;
      return;
    }
    if (context.parsed_ip) {
      ElMessage.warning(`连接地址解析到 IP ${context.parsed_ip}，请先选择或快捷新建服务器`);
    }
  } catch {
    // ignore and fallback to manual selection
    resourceContext.value = null;
  } finally {
    contextLoading.value = false;
  }
}

async function openAdd() {
  newDeploy.value = { host_id: "", port: props.defaultPort };
  hostLocked.value = false;
  resourceContext.value = null;
  addVisible.value = true;
  await fetchHosts();
  await loadResourceDeployContext();
}

async function handleQuickCreateHost() {
  const ip = resourceContext.value?.parsed_ip;
  if (!ip) return;

  quickCreateLoading.value = true;
  const normalizedEnv = normalizeHostEnv(resourceContext.value?.resource_env);
  try {
    await fetchHosts();
    const existingHost = hosts.value.find((host) => {
      const hostIps = splitIpDisplay(host.ip_display);
      return host.env === normalizedEnv && hostIps.includes(ip);
    });
    const hostId = existingHost?.id ?? `host-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;

    if (!existingHost) {
      await saveHost({
        id: hostId,
        hostname: formatTempHostName(),
        env: normalizedEnv,
        status: "running",
        is_deleted: 0,
        created_at: "",
        updated_at: "",
      });
    }

    try {
      await saveIpAddress({
        id: "",
        ip_address: ip,
        env: normalizedEnv,
        is_vip: false,
        real_ips: undefined,
        is_deleted: 0,
        created_at: "",
        updated_at: "",
      });
    } catch {
      // ignore duplicate conflicts and reuse existing one
    }

    const ipResult = await listIpAddresses({
      page: 1,
      page_size: 50,
      search: ip,
      filters: { env: normalizedEnv },
    });
    const ipResource = ipResult.data.find((item) => item.ip_address === ip && item.env === normalizedEnv);
    if (!ipResource) {
      ElMessage.warning("快捷创建失败，未找到对应 IP 资源，请手动处理");
      return;
    }

    await bindHostIp({ host_id: hostId, ip_id: ipResource.id });
  } catch {
    // if created concurrently by others, continue to refresh and reuse
  } finally {
    await fetchHosts();
    const matched = hosts.value.find(
      (h) => h.env === normalizedEnv && splitIpDisplay(h.ip_display).includes(ip)
    );
    if (matched) {
      newDeploy.value.host_id = matched.id;
      hostLocked.value = true;
      resourceContext.value = {
        ...(resourceContext.value || {
          resource_type: props.resourceType,
          resource_id: props.resourceId,
          parsed_ip: ip,
        }),
        matched_host_id: matched.id,
        matched_host_name: matched.hostname,
      };
      ElMessage.success("已快捷创建临时服务器，请后续在主机维护完善信息");
    } else {
      ElMessage.warning("快捷创建未返回可用服务器，请手动创建后重试");
    }
    quickCreateLoading.value = false;
  }
}

function unlockHostSelection() {
  hostLocked.value = false;
}

async function handleAdd() {
  if (hasUnmatchedIp.value && !newDeploy.value.host_id) {
    ElMessage.warning("该连接地址尚未匹配服务器，请先快捷新建或手动选择服务器");
    return;
  }
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

    <el-dialog v-model="addVisible" title="添加部署关系" width="360px" append-to-body>
      <el-form :model="newDeploy" label-width="110px">
        <el-form-item label="目标服务器" required>
          <el-select
            v-model="newDeploy.host_id"
            filterable
            placeholder="选择服务器"
            class="w-full"
            :disabled="hostLocked || contextLoading"
          >
            <el-option
              v-for="h in hosts"
              :key="h.id"
              :label="`${h.hostname} (${h.ip_display || '-'})`"
              :value="h.id"
            />
          </el-select>
          <div v-if="hostLocked" class="host-lock-row">
            <span class="lock-tip">已按连接地址自动匹配</span>
            <el-button text size="small" @click="unlockHostSelection">解锁手动选择</el-button>
          </div>
          <div v-if="hasUnmatchedIp" class="ip-warning-row">
            <span class="warning-text">未找到 IP {{ parsedIp }} 对应服务器</span>
            <el-button text type="primary" size="small" :loading="quickCreateLoading" @click="handleQuickCreateHost">
              快捷新建服务器
            </el-button>
          </div>
        </el-form-item>
        <el-form-item label="运行端口">
          <el-input-number v-model="newDeploy.port" :min="1" :max="65535" class="w-full" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" :disabled="!newDeploy.host_id || contextLoading" @click="handleAdd">
          确定
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.deployment-panel {
  margin-top: 16px;
  border-top: 1px solid var(--im-border-subtle);
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
  color: var(--im-text-primary);
}
.host-lock-row,
.ip-warning-row {
  margin-top: 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.lock-tip {
  color: var(--im-text-secondary);
  font-size: 12px;
}
.warning-text {
  color: var(--im-color-warning-600, #d97706);
  font-size: 12px;
}
</style>
