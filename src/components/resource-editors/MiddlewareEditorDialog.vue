<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type { Middleware } from "@/types";
import { saveMiddleware } from "@/api/middlewares";
import { replaceResourceCallRelations } from "@/api/call-relations";
import { saveDeployment } from "@/api/deployments";
import {
  MIDDLEWARE_CATEGORY_OPTIONS,
  getMiddlewareDefaultPort,
  getMiddlewareIconByType,
  getMiddlewareTypeOptionsWithIcon,
} from "@/utils/middlewareCatalog";
import CallRelationsEditor from "@/components/CallRelationsEditor.vue";
import DeploymentPanel from "@/components/DeploymentPanel.vue";

type MiddlewareEditorMode = "create" | "edit" | "copy";

const props = defineProps<{
  modelValue: boolean;
  mode: MiddlewareEditorMode;
  initialDraft: Partial<Middleware>;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
  (e: "saved", payload: { id: string; mode: MiddlewareEditorMode }): void;
}>();

interface DraftDeploymentItem {
  host_id: string;
  port?: number;
}

interface DeploymentPanelExposed {
  getDraftDeployments: () => DraftDeploymentItem[];
}

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});
const isEditing = computed(() => props.mode === "edit");
const dialogTitle = computed(() => {
  if (props.mode === "edit") return "编辑中间件";
  if (props.mode === "copy") return "复制中间件";
  return "新增中间件";
});

const editingMw = ref<Partial<Middleware>>({});
const saveLoading = ref(false);
const autoFilledPort = ref<number | undefined>(undefined);
const callRelationsEditorRef = ref<InstanceType<typeof CallRelationsEditor> | null>(null);
const deploymentPanelRef = ref<DeploymentPanelExposed | null>(null);

const categoryOptions = MIDDLEWARE_CATEGORY_OPTIONS;
const middlewareTypeOptionsWithIcon = computed(() =>
  getMiddlewareTypeOptionsWithIcon(editingMw.value.category, editingMw.value.type)
);

function cloneDraft(draft: Partial<Middleware>): Partial<Middleware> {
  return {
    ...draft,
  };
}

function middlewareTypeLabel(type?: string): string {
  const normalized = (type ?? "").trim();
  return normalized || "-";
}

function middlewareTypeSlotValue(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);
  return "";
}

function middlewareTypeLabelBySlot(label: unknown, value: unknown): string {
  if (typeof label === "string" && label.trim().length > 0) return label;
  return middlewareTypeLabel(middlewareTypeSlotValue(value));
}

function middlewareTypeIconSrcBySlot(value: unknown): string {
  return getMiddlewareIconByType(middlewareTypeSlotValue(value), editingMw.value.category).src;
}

function middlewareTypeIconAltBySlot(value: unknown): string {
  return getMiddlewareIconByType(middlewareTypeSlotValue(value), editingMw.value.category).alt;
}

function hydrateFromDraft() {
  autoFilledPort.value = undefined;
  editingMw.value = cloneDraft(props.initialDraft || {});
}

async function handleSave() {
  if (saveLoading.value) return;

  const draftItems = callRelationsEditorRef.value?.getDraftItems();
  if (draftItems === null) {
    return;
  }

  const wasEditing = isEditing.value;
  const draftDeployments = !wasEditing ? deploymentPanelRef.value?.getDraftDeployments?.() ?? [] : [];
  const payload: Partial<Middleware> = {
    id: "",
    created_at: "",
    updated_at: "",
    ...editingMw.value,
  };
  saveLoading.value = true;
  try {
    const middlewareId = await saveMiddleware(payload);
    try {
      await replaceResourceCallRelations({
        resource_id: middlewareId,
        resource_type: "middleware",
        items: draftItems ?? [],
      });
    } catch {
      ElMessage.warning("中间件已保存，调用关系保存失败，请重新编辑后重试。");
    }
    if (!wasEditing && draftDeployments.length > 0) {
      try {
        await Promise.all(
          draftDeployments.map((item) =>
            saveDeployment({
              id: "",
              resource_id: middlewareId,
              resource_type: "middleware",
              host_id: item.host_id,
              port: item.port,
            })
          )
        );
      } catch {
        ElMessage.warning("中间件已保存，部署关系保存失败，请在部署关系中重试。");
      }
    }
    ElMessage.success(wasEditing ? "更新成功" : "创建成功");
    visible.value = false;
    emit("saved", { id: middlewareId, mode: props.mode });
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

watch(
  () => editingMw.value.type,
  (newType) => {
    const defaultPort = getMiddlewareDefaultPort(newType);
    if (!defaultPort) return;
    const currentPort = editingMw.value.port;
    if (currentPort == null || currentPort === autoFilledPort.value) {
      editingMw.value.port = defaultPort;
      autoFilledPort.value = defaultPort;
    }
  }
);

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
    <el-form :model="editingMw" label-width="96px">
      <el-divider content-position="left">基础信息</el-divider>
      <el-form-item label="实例名称" required>
        <el-input v-model="editingMw.name" placeholder="请输入实例名称" />
      </el-form-item>
      <el-form-item label="分类" required>
        <el-select v-model="editingMw.category" class="w-full">
          <el-option
            v-for="option in categoryOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="类型" required>
        <el-select
          v-model="editingMw.type"
          class="w-full"
          filterable
          allow-create
          default-first-option
          clearable
          placeholder="可选择常用类型或手动输入"
        >
          <template #label="{ label, value }">
            <div class="middleware-type-option middleware-type-selected">
              <img
                :src="middlewareTypeIconSrcBySlot(value)"
                :alt="middlewareTypeIconAltBySlot(value)"
                class="middleware-type-option-icon"
              />
              <span>{{ middlewareTypeLabelBySlot(label, value) }}</span>
            </div>
          </template>
          <el-option
            v-for="option in middlewareTypeOptionsWithIcon"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          >
            <div class="middleware-type-option">
              <img :src="option.icon.src" :alt="option.icon.alt" class="middleware-type-option-icon" />
              <span>{{ option.label }}</span>
            </div>
          </el-option>
        </el-select>
      </el-form-item>
      <el-form-item label="连接地址" required>
        <el-input v-model="editingMw.address" placeholder="如 192.168.1.100" />
      </el-form-item>
      <el-form-item label="端口">
        <el-input-number v-model="editingMw.port" :min="1" :max="65535" class="w-full" />
      </el-form-item>

      <el-divider content-position="left">实例信息</el-divider>
      <el-form-item label="版本">
        <el-input v-model="editingMw.version" placeholder="如 8.0.33" />
      </el-form-item>
      <el-form-item label="环境" required>
        <el-select v-model="editingMw.env" class="w-full">
          <el-option label="生产" value="prod" />
          <el-option label="开发" value="dev" />
          <el-option label="测试" value="test" />
        </el-select>
      </el-form-item>

      <el-divider content-position="left">运维信息</el-divider>
      <el-form-item label="描述">
        <el-input v-model="editingMw.description" type="textarea" :rows="3" maxlength="300" show-word-limit />
      </el-form-item>
    </el-form>

    <CallRelationsEditor
      ref="callRelationsEditorRef"
      :resource-id="isEditing ? editingMw.id : undefined"
      resource-type="middleware"
    />

    <DeploymentPanel
      v-if="Boolean(editingMw.id)"
      ref="deploymentPanelRef"
      :resource-id="editingMw.id!"
      resource-type="middleware"
      :resource-persisted="isEditing"
      :resource-address="editingMw.address"
      :resource-env="editingMw.env"
      :default-port="editingMw.port"
    />

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.middleware-type-option {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.middleware-type-option-icon {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  object-fit: cover;
}

.middleware-type-selected {
  max-width: 100%;
}
</style>
