<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type { NginxConfig, NginxEndpoint } from "@/types";
import { saveNginxConfig } from "@/api/nginx-configs";
import { replaceResourceCallRelations } from "@/api/call-relations";
import CallRelationsEditor from "@/components/CallRelationsEditor.vue";
import DeploymentPanel from "@/components/DeploymentPanel.vue";

type NginxEditorMode = "create" | "edit" | "copy";

const props = defineProps<{
  modelValue: boolean;
  mode: NginxEditorMode;
  initialDraft: Partial<NginxConfig>;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
  (e: "saved", payload: { id: string; mode: NginxEditorMode }): void;
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});
const isEditing = computed(() => props.mode === "edit");
const dialogTitle = computed(() => {
  if (props.mode === "edit") return "编辑负载均衡";
  if (props.mode === "copy") return "复制负载均衡";
  return "新增负载均衡";
});

const editingNc = ref<Partial<NginxConfig>>({});
const saveLoading = ref(false);
const callRelationsEditorRef = ref<InstanceType<typeof CallRelationsEditor> | null>(null);

function createEmptyEndpoint(): NginxEndpoint {
  return {
    host: "",
    port: 80,
  };
}

function normalizeEndpointDraft(endpoints?: NginxEndpoint[]): NginxEndpoint[] {
  if (!Array.isArray(endpoints)) {
    return [createEmptyEndpoint()];
  }
  const normalized = endpoints.map((item) => ({
    host: (item?.host ?? "").trim(),
    port: Number(item?.port ?? 0),
  }));
  return normalized.length > 0 ? normalized : [createEmptyEndpoint()];
}

function cloneDraft(draft: Partial<NginxConfig>): Partial<NginxConfig> {
  return {
    ...draft,
    endpoints: normalizeEndpointDraft(draft.endpoints),
  };
}

function ensureEndpointDraft() {
  editingNc.value.endpoints = normalizeEndpointDraft(editingNc.value.endpoints);
}

function addEndpoint() {
  ensureEndpointDraft();
  editingNc.value.endpoints!.push(createEmptyEndpoint());
}

function removeEndpoint(index: number) {
  ensureEndpointDraft();
  if (editingNc.value.endpoints!.length <= 1) {
    ElMessage.warning("至少保留一个连接端点");
    return;
  }
  editingNc.value.endpoints!.splice(index, 1);
}

function normalizeEndpointsForSave(endpoints?: NginxEndpoint[]): NginxEndpoint[] {
  if (!Array.isArray(endpoints)) {
    return [];
  }
  return endpoints.map((item) => ({
    host: (item.host || "").trim(),
    port: Number(item.port),
  }));
}

function isValidIpv4(host: string): boolean {
  const candidate = host.trim();
  const parts = candidate.split(".");
  if (parts.length !== 4) return false;
  return parts.every((part) => {
    if (!/^\d+$/.test(part)) return false;
    const value = Number(part);
    return value >= 0 && value <= 255;
  });
}

function pickDeployContextAddress(endpoints?: NginxEndpoint[]): string | undefined {
  const normalized = normalizeEndpointsForSave(endpoints);
  if (normalized.length === 0) return undefined;
  const selected = normalized.find((item) => isValidIpv4(item.host)) || normalized[0];
  if (!selected.host || !Number.isInteger(selected.port) || selected.port < 1 || selected.port > 65535) {
    return undefined;
  }
  return `${selected.host}:${selected.port}`;
}

function validateEndpointsBeforeSave(endpoints: NginxEndpoint[]): boolean {
  if (endpoints.length === 0) {
    ElMessage.warning("请至少填写一个连接端点");
    return false;
  }
  for (let index = 0; index < endpoints.length; index += 1) {
    const item = endpoints[index];
    if (!item.host) {
      ElMessage.warning(`第 ${index + 1} 条连接端点的地址或域名不能为空`);
      return false;
    }
    if (!Number.isInteger(item.port) || item.port < 1 || item.port > 65535) {
      ElMessage.warning(`第 ${index + 1} 条连接端点端口必须在 1-65535 之间`);
      return false;
    }
  }
  return true;
}

function hydrateFromDraft() {
  editingNc.value = cloneDraft(props.initialDraft || {});
}

async function handleSave() {
  if (saveLoading.value) return;

  const draftItems = callRelationsEditorRef.value?.getDraftItems();
  if (draftItems === null) {
    return;
  }

  const endpoints = normalizeEndpointsForSave(editingNc.value.endpoints);
  if (!validateEndpointsBeforeSave(endpoints)) {
    return;
  }

  const payload: Partial<NginxConfig> = {
    id: "",
    created_at: "",
    updated_at: "",
    ...editingNc.value,
    endpoints,
  };
  saveLoading.value = true;
  try {
    const nginxId = await saveNginxConfig(payload);
    try {
      await replaceResourceCallRelations({
        resource_id: nginxId,
        resource_type: "nginx",
        items: draftItems ?? [],
      });
    } catch {
      ElMessage.warning("负载均衡已保存，调用关系保存失败，请重新编辑后重试。");
    }
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    visible.value = false;
    emit("saved", { id: nginxId, mode: props.mode });
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

watch(
  () => props.modelValue,
  (value) => {
    if (value) {
      hydrateFromDraft();
    }
  },
  { immediate: true }
);

watch(
  () => props.initialDraft,
  () => {
    if (props.modelValue) {
      hydrateFromDraft();
    }
  },
  { deep: true }
);
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="dialogTitle"
    width="700px"
    align-center
    destroy-on-close
  >
    <el-form :model="editingNc" label-width="96px">
      <el-divider content-position="left">基础信息</el-divider>
      <el-form-item label="配置名称" required>
        <el-input v-model="editingNc.name" placeholder="请输入配置名称" />
      </el-form-item>
      <el-form-item label="连接端点" required>
        <div class="endpoint-editor">
          <div v-for="(item, index) in editingNc.endpoints || []" :key="index" class="endpoint-row">
            <el-input
              v-model="item.host"
              class="endpoint-host"
              placeholder="地址或域名，如 edge.example.com 或 10.0.0.8"
            />
            <el-input-number
              v-model="item.port"
              :min="1"
              :max="65535"
              class="endpoint-port"
              controls-position="right"
            />
            <el-button text type="danger" :disabled="(editingNc.endpoints || []).length <= 1" @click="removeEndpoint(index)">
              删除
            </el-button>
          </div>
          <el-button text type="primary" @click="addEndpoint">添加端点</el-button>
        </div>
      </el-form-item>
      <el-form-item label="负载策略">
        <el-select v-model="editingNc.strategy" class="w-full">
          <el-option label="轮询 (roundrobin)" value="roundrobin" />
          <el-option label="IP 哈希 (ip_hash)" value="ip_hash" />
        </el-select>
      </el-form-item>

      <el-divider content-position="left">运维信息</el-divider>
      <el-form-item label="环境" required>
        <el-select v-model="editingNc.env" class="w-full">
          <el-option label="生产" value="prod" />
          <el-option label="开发" value="dev" />
          <el-option label="测试" value="test" />
        </el-select>
      </el-form-item>
      <el-form-item label="状态" required>
        <el-select v-model="editingNc.status" class="w-full">
          <el-option label="运行中" value="running" />
          <el-option label="已停止" value="stopped" />
          <el-option label="维护中" value="maintenance" />
        </el-select>
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="editingNc.description" type="textarea" :rows="3" maxlength="300" show-word-limit />
      </el-form-item>
    </el-form>

    <CallRelationsEditor
      ref="callRelationsEditorRef"
      :resource-id="isEditing ? editingNc.id : undefined"
      resource-type="nginx"
    />

    <DeploymentPanel
      v-if="isEditing && editingNc.id"
      :resource-id="editingNc.id!"
      resource-type="nginx"
      :resource-address="pickDeployContextAddress(editingNc.endpoints)"
      :resource-env="editingNc.env"
    />

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.endpoint-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.endpoint-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 140px auto;
  gap: 8px;
  align-items: center;
}

.endpoint-host,
.endpoint-port {
  width: 100%;
}

@media (max-width: 768px) {
  .endpoint-row {
    grid-template-columns: 1fr;
  }
}
</style>
