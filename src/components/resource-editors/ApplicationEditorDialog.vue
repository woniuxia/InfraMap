<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type { Application, BusinessApplication, Contact, EditorMode } from "@/types";
import type { TaxonomyAppType } from "@/api/taxonomy";
import { saveApplication } from "@/api/applications";
import { listBusinessApplications } from "@/api/business-applications";
import { getContact, listContacts, saveContact } from "@/api/contacts";
import { listApplicationTechStackTerms } from "@/api/taxonomy";
import { replaceResourceCallRelations } from "@/api/call-relations";
import { saveDeployment } from "@/api/deployments";
import { DEPLOY_MODE_OPTIONS, ENV_OPTIONS, STATUS_OPTIONS } from "@/constants/options";
import { buildTechStackSuggestions, parseTechStack, techStackToText } from "@/utils/techStack";
import CallRelationsEditor from "@/components/CallRelationsEditor.vue";
import DeploymentPanel from "@/components/DeploymentPanel.vue";

const props = defineProps<{
  modelValue: boolean;
  mode: EditorMode;
  initialDraft: Partial<Application>;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
  (e: "saved", payload: { id: string; mode: EditorMode }): void;
}>();

interface DraftDeploymentItem {
  host_id: string;
  port?: number;
}

interface DeploymentPanelExposed {
  getDraftDeployments: () => DraftDeploymentItem[];
}

const BUSINESS_APPLICATION_OPTION_LIMIT = 500;

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});
const isEditing = computed(() => props.mode === "edit");
const dialogTitle = computed(() => {
  if (props.mode === "edit") return "编辑应用";
  if (props.mode === "copy") return "复制应用";
  return "新增应用";
});

const editingApp = ref<Partial<Application>>({});
const saveLoading = ref(false);
const businessApplicationOptionsLoading = ref(false);
const techStackList = ref<string[]>([]);
const topTechStackOptions = ref<string[]>([]);
const ownerContactIdList = ref<string[]>([]);
const ownerContactOptionsLoading = ref(false);
const ownerContactOptions = ref<Contact[]>([]);
const searchedOwnerKeyword = ref("");
const quickCreatingOwner = ref(false);
const businessApplicationOptions = ref<BusinessApplication[]>([]);
const callRelationsEditorRef = ref<InstanceType<typeof CallRelationsEditor> | null>(null);
const deploymentPanelRef = ref<DeploymentPanelExposed | null>(null);
const deployModeOptions = DEPLOY_MODE_OPTIONS;

const techStackSuggestions = computed(() =>
  buildTechStackSuggestions(
    topTechStackOptions.value.map((item) => ({ tech_stack: item })),
    techStackList.value,
  ),
);
const canQuickCreateOwner = computed(() => {
  const keyword = searchedOwnerKeyword.value.trim();
  return keyword.length > 0 && ownerContactOptions.value.length === 0;
});
const selectedBusinessApplicationId = computed(() =>
  normalizeBusinessApplicationId(editingApp.value.business_application_id),
);
const allBusinessApplicationOptions = computed<BusinessApplication[]>(() => {
  const activeOptions = businessApplicationOptions.value.filter((item) => item.status === "active");
  const selectedId = selectedBusinessApplicationId.value;
  const selectedName = editingApp.value.business_application_name?.trim();
  if (!selectedId || activeOptions.some((item) => item.id === selectedId) || !selectedName) {
    return activeOptions;
  }
  const fallbackInactiveOption: BusinessApplication = {
    id: selectedId,
    name: selectedName,
    status: "inactive",
    env: undefined,
    created_at: "",
    updated_at: "",
  };
  return [...activeOptions, fallbackInactiveOption];
});
const businessApplicationSelectOptions = computed(() => {
  const selectedId = selectedBusinessApplicationId.value;
  const appEnv = editingApp.value.env?.trim();
  if (!appEnv) {
    return allBusinessApplicationOptions.value;
  }
  return allBusinessApplicationOptions.value.filter(
    (item) => item.id === selectedId || item.env === appEnv,
  );
});
const selectedBusinessApplicationEnvMismatch = computed(() => {
  const selectedId = selectedBusinessApplicationId.value;
  if (!selectedId) {
    return false;
  }
  const selectedOption = allBusinessApplicationOptions.value.find((item) => item.id === selectedId);
  if (!selectedOption) {
    return true;
  }
  if (selectedOption.status !== "active") {
    return true;
  }
  const appEnv = editingApp.value.env?.trim();
  if (!appEnv) {
    return false;
  }
  return selectedOption.env !== appEnv;
});

function cloneDraft(draft: Partial<Application>): Partial<Application> {
  return {
    ...draft,
    owners: Array.isArray(draft.owners) ? [...draft.owners] : draft.owners,
    owner_contact_ids: Array.isArray(draft.owner_contact_ids)
      ? [...draft.owner_contact_ids]
      : draft.owner_contact_ids,
  };
}

function normalizeOwnerContactIds(ownerContactIds?: string[]) {
  const values = [...(ownerContactIds ?? [])]
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
  return Array.from(new Set(values));
}

function contactOptionLabel(contact: Contact) {
  const name = contact.name?.trim() || "-";
  const phone = contact.phone?.trim();
  const email = contact.email?.trim();
  const meta = [phone, email].filter(Boolean).join(" / ");
  return meta ? `${name}（${meta}）` : name;
}

function handleTechStackChange(values: string[]) {
  techStackList.value = buildTechStackSuggestions([], values);
}

async function fetchOwnerContactOptions(keyword: string) {
  ownerContactOptionsLoading.value = true;
  try {
    const result = await listContacts({
      page: 1,
      page_size: 50,
      search: keyword.trim(),
    });
    ownerContactOptions.value = result.data;
  } catch {
    // error shown by tauriInvoke
  } finally {
    ownerContactOptionsLoading.value = false;
  }
}

async function preloadOwnerContacts(contactIds: string[]) {
  const normalizedIds = normalizeOwnerContactIds(contactIds);
  if (normalizedIds.length === 0) return;

  try {
    const results = await Promise.all(
      normalizedIds.map(async (id) => {
        try {
          return await getContact(id);
        } catch {
          return null;
        }
      }),
    );

    const contacts = results.filter((item): item is Contact => Boolean(item));
    if (contacts.length > 0) {
      ownerContactOptions.value = contacts;
    }
  } catch {
    // error shown by tauriInvoke
  }
}

function handleOwnerSearch(keyword: string) {
  searchedOwnerKeyword.value = keyword.trim();
  void fetchOwnerContactOptions(searchedOwnerKeyword.value);
}

function handleOwnerDropdownVisible(visible: boolean) {
  if (!visible) return;
  void fetchOwnerContactOptions(searchedOwnerKeyword.value);
}

async function handleQuickCreateOwner() {
  const keyword = searchedOwnerKeyword.value.trim();
  if (!keyword || quickCreatingOwner.value) return;

  quickCreatingOwner.value = true;
  try {
    const contactId = await saveContact({
      id: "",
      name: keyword,
    });
    ownerContactIdList.value = normalizeOwnerContactIds([...ownerContactIdList.value, contactId]);
    ownerContactOptions.value = [
      {
        id: contactId,
        name: keyword,
        phone: undefined,
        email: undefined,
        remark: undefined,
        created_at: "",
        updated_at: "",
      },
    ];
    ElMessage.success("联系人已创建并回填");
  } catch {
    // error shown by tauriInvoke
  } finally {
    quickCreatingOwner.value = false;
  }
}

function resolveTechStackSide(type: Application["type"] | undefined): TaxonomyAppType {
  return type === "frontend" ? "frontend" : "backend";
}

function applyDefaultPortByType(type: Application["type"] | undefined) {
  if (type === "frontend") {
    editingApp.value.port = 80;
    return;
  }
  if (type === "backend") {
    editingApp.value.port = 8080;
  }
}

function handleTypeChange(type: Application["type"] | undefined) {
  applyDefaultPortByType(type);
  void fetchTopTechStackOptions(type);
}

async function fetchTopTechStackOptions(type: Application["type"] | undefined) {
  try {
    topTechStackOptions.value = await listApplicationTechStackTerms({
      app_type: resolveTechStackSide(type),
      limit: 10,
    });
  } catch {
    // error shown by tauriInvoke
  }
}

function normalizeBusinessApplicationId(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

function envLabel(env: string) {
  return ({ prod: "生产", dev: "开发", test: "测试" } as Record<string, string>)[env] || env;
}

function businessApplicationLabel(item: BusinessApplication) {
  const envText = item.env ? envLabel(item.env) : "未设置环境";
  return `${item.name?.trim() || "-"}（${envText}）`;
}

function handleBusinessApplicationChange(value: string | number | undefined | null) {
  const nextId = normalizeBusinessApplicationId(value);
  editingApp.value.business_application_id = nextId || undefined;
  if (!nextId) {
    editingApp.value.business_application_name = undefined;
    return;
  }
  const selected = allBusinessApplicationOptions.value.find((item) => item.id === nextId);
  editingApp.value.business_application_name =
    selected?.name?.trim() || editingApp.value.business_application_name;
}

async function fetchBusinessApplicationOptions() {
  businessApplicationOptionsLoading.value = true;
  try {
    const result = await listBusinessApplications({
      page: 1,
      page_size: BUSINESS_APPLICATION_OPTION_LIMIT,
      filters: {
        status: "active",
      },
    });
    businessApplicationOptions.value = result.data.filter((item) => item.status === "active");
  } catch {
    // error shown by tauriInvoke
  } finally {
    businessApplicationOptionsLoading.value = false;
  }
}

function hydrateFromDraft() {
  editingApp.value = cloneDraft(props.initialDraft || {});
  techStackList.value = parseTechStack(editingApp.value.tech_stack);
  ownerContactIdList.value = normalizeOwnerContactIds(editingApp.value.owner_contact_ids);
  searchedOwnerKeyword.value = "";
  ownerContactOptions.value = [];
  void preloadOwnerContacts(ownerContactIdList.value);
  void fetchTopTechStackOptions(editingApp.value.type);
  void fetchBusinessApplicationOptions();
}

async function handleSave() {
  if (saveLoading.value) return;

  const draftItems = callRelationsEditorRef.value?.getDraftItems();
  if (draftItems === null) {
    return;
  }
  if (selectedBusinessApplicationEnvMismatch.value) {
    ElMessage.warning("所属业务应用与当前环境不一致，请重新选择。");
    return;
  }

  const wasEditing = isEditing.value;
  const ownerContactIds = normalizeOwnerContactIds(ownerContactIdList.value);
  const businessApplicationId = normalizeBusinessApplicationId(
    editingApp.value.business_application_id,
  );
  const draftDeployments = !wasEditing
    ? (deploymentPanelRef.value?.getDraftDeployments?.() ?? [])
    : [];
  const {
    owners: _ignoredOwners,
    owner_contact_ids: _ignoredContactIds,
    ...appDraft
  } = editingApp.value;
  const payload: Partial<Application> = {
    id: "",
    created_at: "",
    updated_at: "",
    ...appDraft,
    business_application_id: businessApplicationId || undefined,
    business_application_name: undefined,
    owner_contact_ids: ownerContactIds,
    tech_stack: techStackToText(techStackList.value),
  };
  saveLoading.value = true;
  try {
    const appId = await saveApplication(payload);
    try {
      await replaceResourceCallRelations({
        resource_id: appId,
        resource_type: "application",
        items: draftItems ?? [],
      });
    } catch {
      ElMessage.warning("应用已保存，调用关系保存失败，请重新编辑后重试。");
    }
    if (!wasEditing && draftDeployments.length > 0) {
      try {
        await Promise.all(
          draftDeployments.map((item) =>
            saveDeployment({
              id: "",
              resource_id: appId,
              resource_type: "application",
              host_id: item.host_id,
              port: item.port,
            }),
          ),
        );
      } catch {
        ElMessage.warning("应用已保存，部署关系保存失败，请在部署关系中重试。");
      }
    }
    ElMessage.success(wasEditing ? "更新成功" : "创建成功");
    visible.value = false;
    emit("saved", { id: appId, mode: props.mode });
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
  { immediate: true },
);

watch(
  () => props.initialDraft,
  () => {
    if (props.modelValue) {
      hydrateFromDraft();
    }
  },
  { deep: true },
);
</script>

<template>
  <el-dialog v-model="visible" :title="dialogTitle" width="700px" align-center destroy-on-close>
    <el-form :model="editingApp" label-width="96px">
      <el-divider content-position="left">基础信息</el-divider>
      <el-form-item label="服务名称" required>
        <el-input v-model="editingApp.name" placeholder="请输入服务名称" />
      </el-form-item>
      <el-form-item label="类型" required>
        <el-select
          v-model="editingApp.type"
          class="w-full"
          @change="(v) => handleTypeChange(v as Application['type'])"
        >
          <el-option label="前端" value="frontend" />
          <el-option label="后端" value="backend" />
          <el-option label="网关" value="gateway" />
          <el-option label="批处理" value="batch_job" />
          <el-option label="微服务" value="microservice" />
          <el-option label="其他" value="other" />
        </el-select>
      </el-form-item>
      <el-form-item label="访问地址">
        <el-input
          v-model="editingApp.address"
          placeholder="如 api.example.com、https://app.example.com 或 192.168.1.100"
        />
      </el-form-item>
      <el-form-item label="端口">
        <el-input-number v-model="editingApp.port" :min="1" :max="65535" class="w-full" />
      </el-form-item>

      <el-divider content-position="left">部署信息</el-divider>
      <el-form-item label="技术栈">
        <el-select
          v-model="techStackList"
          class="w-full"
          multiple
          filterable
          allow-create
          default-first-option
          :reserve-keyword="false"
          placeholder="输入技术栈进行筛选，按回车可新增"
          @change="(values) => handleTechStackChange(values as string[])"
        >
          <el-option v-for="item in techStackSuggestions" :key="item" :label="item" :value="item" />
        </el-select>
      </el-form-item>
      <el-form-item label="部署方式">
        <el-select
          v-model="editingApp.deploy_mode"
          clearable
          placeholder="请选择部署方式"
          class="w-full"
        >
          <el-option
            v-for="item in deployModeOptions"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="环境" required>
        <el-select v-model="editingApp.env" class="w-full">
          <el-option
            v-for="item in ENV_OPTIONS"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="Git仓库">
        <el-input v-model="editingApp.git_repo" placeholder="Git 仓库地址" />
      </el-form-item>
      <el-form-item label="负责人">
        <el-select
          v-model="ownerContactIdList"
          multiple
          filterable
          remote
          clearable
          class="w-full"
          placeholder="搜索联系人（支持姓名/电话/邮箱，多选）"
          :loading="ownerContactOptionsLoading"
          :remote-method="handleOwnerSearch"
          @visible-change="handleOwnerDropdownVisible"
        >
          <el-option
            v-for="contact in ownerContactOptions"
            :key="contact.id"
            :label="contactOptionLabel(contact)"
            :value="contact.id"
          />
          <template #empty>
            <div class="owner-empty">
              <template v-if="!searchedOwnerKeyword">暂无联系人</template>
              <template v-else-if="canQuickCreateOwner">
                <span>未找到联系人：{{ searchedOwnerKeyword }}</span>
                <el-button
                  type="primary"
                  text
                  :loading="quickCreatingOwner"
                  @click="handleQuickCreateOwner"
                >
                  点击新增并回填
                </el-button>
              </template>
              <template v-else>未找到匹配联系人</template>
            </div>
          </template>
        </el-select>
        <div class="owner-hint">负责人基于联系人 ID 绑定，联系人姓名后续可修改。</div>
      </el-form-item>
      <el-form-item label="所属业务应用">
        <el-select
          v-model="editingApp.business_application_id"
          class="w-full"
          clearable
          filterable
          :loading="businessApplicationOptionsLoading"
          placeholder="请选择所属业务应用"
          @change="(value) => handleBusinessApplicationChange(value as string | number | undefined)"
          @clear="() => handleBusinessApplicationChange(undefined)"
        >
          <el-option
            v-for="item in businessApplicationSelectOptions"
            :key="item.id"
            :label="businessApplicationLabel(item)"
            :value="item.id"
          />
        </el-select>
        <div v-if="selectedBusinessApplicationEnvMismatch" class="business-app-warning">
          当前选择与环境不一致，保存前请重新选择或清空。
        </div>
      </el-form-item>

      <el-divider content-position="left">运维信息</el-divider>
      <el-form-item label="状态" required>
        <el-select v-model="editingApp.status" class="w-full">
          <el-option
            v-for="item in STATUS_OPTIONS"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="描述">
        <el-input
          v-model="editingApp.description"
          type="textarea"
          :rows="3"
          maxlength="300"
          show-word-limit
        />
      </el-form-item>
    </el-form>

    <CallRelationsEditor
      ref="callRelationsEditorRef"
      :resource-id="isEditing ? editingApp.id : undefined"
      resource-type="application"
    />

    <DeploymentPanel
      v-if="Boolean(editingApp.id)"
      ref="deploymentPanelRef"
      :resource-id="editingApp.id!"
      resource-type="application"
      :resource-persisted="isEditing"
      :default-port="editingApp.port"
    />

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.business-app-warning {
  margin-top: 6px;
  font-size: 12px;
  color: var(--im-warning);
}

.owner-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--im-text-tertiary);
}

.owner-empty {
  padding: 8px 12px;
  font-size: 12px;
  color: var(--im-text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
</style>
