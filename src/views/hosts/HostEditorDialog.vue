<script setup lang="ts">
import { computed } from "vue";
import type { Ref } from "vue";
import type { ComponentPublicInstance } from "vue";
import type { FormInstance, FormRules } from "element-plus";
import type { Host, IpAddress } from "@/types";
import {
  COMMON_CPU_CORES_OPTIONS,
  COMMON_CPU_FREQ_OPTIONS,
  COMMON_CPU_THREADS_OPTIONS,
  COMMON_DISK_OPTIONS_GB,
  COMMON_RAM_OPTIONS_GB,
} from "@/views/hostsHardwareOptions";

interface Props {
  modelValue: boolean;
  isEditing: boolean;
  editingHost: Partial<Host>;
  formRef: Ref<FormInstance | undefined>;
  formRules: FormRules;
  envOptions: Array<{ label: string; value: string }>;
  statusOptions: Array<{ label: string; value: string }>;
  osOptions: string[];
  tagList: string[];
  formTagSuggestionOptions: Array<{ label: string; value: string }>;
  selectedIpIds: string[];
  bindingLoading: boolean;
  allowCrossEnv: boolean;
  filteredIpOptions: IpAddress[];
  searchedIpKeyword: string;
  canQuickCreateIp: boolean;
  saveLoading: boolean;
  formatIpOptionLabel: (ip: IpAddress) => string;
  quickIpDialogVisible: boolean;
  quickIpSaving: boolean;
  quickIpFormRef: Ref<FormInstance | undefined>;
  quickIpForm: Partial<IpAddress>;
  quickIpFormRules: FormRules;
  quickRealIpList: string[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  "update:selectedIpIds": [value: string[]];
  "update:allowCrossEnv": [value: boolean];
  "update:tagList": [value: string[]];
  "update:quickIpDialogVisible": [value: boolean];
  save: [];
  "binding-search": [keyword: string];
  "binding-dropdown-visible": [visible: boolean];
  "refresh-binding-context": [hostId?: string];
  "open-quick-create-ip": [];
  "save-quick-ip": [];
  "add-quick-real-ip": [];
  "remove-quick-real-ip": [index: number];
}>();

const dialogVisibleModel = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});

const selectedIpIdsModel = computed({
  get: () => props.selectedIpIds,
  set: (value: string[]) => emit("update:selectedIpIds", value),
});

const allowCrossEnvModel = computed({
  get: () => props.allowCrossEnv,
  set: (value: boolean) => emit("update:allowCrossEnv", value),
});

const tagListModel = computed({
  get: () => props.tagList,
  set: (value: string[]) => emit("update:tagList", value),
});

const quickIpDialogVisibleModel = computed({
  get: () => props.quickIpDialogVisible,
  set: (value: boolean) => emit("update:quickIpDialogVisible", value),
});

const ipv4Pattern = /^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$/;

function bindMainFormRef(instance: Element | ComponentPublicInstance | null) {
  props.formRef.value = (instance as FormInstance | null) ?? undefined;
}

function bindQuickFormRef(instance: Element | ComponentPublicInstance | null) {
  props.quickIpFormRef.value = (instance as FormInstance | null) ?? undefined;
}

function isHostFormBasicValid(): boolean {
  const hostname = props.editingHost.hostname?.trim();
  return Boolean(hostname && props.editingHost.env && props.editingHost.status);
}

function isQuickIpFormBasicValid(): boolean {
  const ip = props.quickIpForm.ip_address?.trim() || "";
  return Boolean(ipv4Pattern.test(ip) && props.quickIpForm.env);
}

async function submitHostForm() {
  const hasValidator = typeof props.formRef.value?.validate === "function";
  if (hasValidator) {
    const valid = await props.formRef.value?.validate().catch(() => false);
    if (!valid) return;
    emit("save");
    return;
  }

  if (!isHostFormBasicValid()) {
    return;
  }
  emit("save");
}

async function submitQuickIpForm() {
  const hasValidator = typeof props.quickIpFormRef.value?.validate === "function";
  if (hasValidator) {
    const valid = await props.quickIpFormRef.value?.validate().catch(() => false);
    if (!valid) return;
    emit("save-quick-ip");
    return;
  }

  if (!isQuickIpFormBasicValid()) {
    return;
  }
  emit("save-quick-ip");
}
</script>

<template>
  <el-dialog
    v-model="dialogVisibleModel"
    :title="props.isEditing ? '编辑服务器' : '新增服务器'"
    width="760px"
    align-center
    destroy-on-close
  >
    <el-form :ref="bindMainFormRef" :model="props.editingHost" :rules="props.formRules" label-width="96px">
      <el-divider content-position="left">基础信息</el-divider>
      <el-row :gutter="12">
        <el-col :span="12">
          <el-form-item label="主机名" prop="hostname" required>
            <el-input
              v-model="props.editingHost.hostname"
              placeholder="请输入主机名，例如 web-prod-01"
            />
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item label="环境" prop="env" required>
            <el-select v-model="props.editingHost.env" class="w-full">
              <el-option
                v-for="option in props.envOptions"
                :key="option.value"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
          </el-form-item>
        </el-col>
      </el-row>

      <el-form-item label="绑定IP">
        <div class="binding-editor">
          <div class="binding-toolbar">
            <el-switch
              v-model="allowCrossEnvModel"
              inline-prompt
              :active-text="'跨环境'"
              :inactive-text="'同环境'"
            />
            <el-button
              text
              size="small"
              :loading="props.bindingLoading"
              @click="emit('refresh-binding-context', props.editingHost.id)"
            >
              刷新IP列表
            </el-button>
          </div>
          <el-select
            v-model="selectedIpIdsModel"
            multiple
            filterable
            remote
            clearable
            class="w-full"
            placeholder="选择绑定IP（支持多选）"
            :loading="props.bindingLoading"
            :remote-method="(keyword: string) => emit('binding-search', keyword)"
            @visible-change="(visible: boolean) => emit('binding-dropdown-visible', visible)"
          >
            <el-option
              v-for="ip in props.filteredIpOptions"
              :key="ip.id"
              :label="props.formatIpOptionLabel(ip)"
              :value="ip.id"
            />
            <template #empty>
              <div class="binding-empty">
                <template v-if="!props.searchedIpKeyword">暂无可选 IP</template>
                <template v-else-if="props.canQuickCreateIp">
                  <span>当前环境尚未录入 IP：{{ props.searchedIpKeyword }}</span>
                  <el-button type="primary" text @click="emit('open-quick-create-ip')">
                    点击新增并回填
                  </el-button>
                </template>
                <template v-else>未找到匹配 IP</template>
              </div>
            </template>
          </el-select>
          <div class="binding-hint">
            默认仅显示与服务器同环境 IP，开启“跨环境”可查看全部。支持在下拉框中输入 IP，未录入时可直接新增。
          </div>
        </div>
      </el-form-item>

      <el-form-item label="操作系统">
        <el-select
          v-model="props.editingHost.os_type"
          filterable
          allow-create
          clearable
          placeholder="选择或输入操作系统版本，如 Ubuntu 22.04"
          class="w-full"
        >
          <el-option v-for="os in props.osOptions" :key="os" :label="os" :value="os" />
        </el-select>
      </el-form-item>

      <el-divider content-position="left">硬件规格</el-divider>
      <el-form-item label="CPU 型号">
        <el-input v-model="props.editingHost.cpu_model" placeholder="如 Intel Xeon E5-2680 v4" />
      </el-form-item>
      <el-form-item label="CPU 参数">
        <el-row :gutter="12" class="w-full">
          <el-col :span="8">
            <div class="inline-field">
              <span class="inline-label">核心数</span>
              <el-select
                v-model="props.editingHost.cpu_cores"
                filterable
                allow-create
                default-first-option
                clearable
                placeholder="8"
                class="inline-input"
              >
                <el-option
                  v-for="value in COMMON_CPU_CORES_OPTIONS"
                  :key="`cpu-core-${value}`"
                  :label="String(value)"
                  :value="value"
                />
              </el-select>
            </div>
          </el-col>
          <el-col :span="8">
            <div class="inline-field">
              <span class="inline-label">线程数</span>
              <el-select
                v-model="props.editingHost.cpu_threads"
                filterable
                allow-create
                default-first-option
                clearable
                placeholder="16"
                class="inline-input"
              >
                <el-option
                  v-for="value in COMMON_CPU_THREADS_OPTIONS"
                  :key="`cpu-thread-${value}`"
                  :label="String(value)"
                  :value="value"
                />
              </el-select>
            </div>
          </el-col>
          <el-col :span="8">
            <div class="inline-field">
              <span class="inline-label">频率</span>
              <el-select
                v-model="props.editingHost.cpu_freq"
                filterable
                allow-create
                default-first-option
                clearable
                placeholder="2.4"
                class="inline-input"
              >
                <el-option
                  v-for="value in COMMON_CPU_FREQ_OPTIONS"
                  :key="`cpu-freq-${value}`"
                  :label="value"
                  :value="value"
                />
              </el-select>
              <span class="inline-unit">GHz</span>
            </div>
          </el-col>
        </el-row>
      </el-form-item>
      <el-form-item label="硬件配置">
        <el-row :gutter="12" class="w-full">
          <el-col :span="12">
            <div class="inline-field">
              <span class="inline-label">内存</span>
              <el-select
                v-model="props.editingHost.ram_gb"
                filterable
                allow-create
                default-first-option
                clearable
                placeholder="16"
                class="inline-input"
              >
                <el-option
                  v-for="value in COMMON_RAM_OPTIONS_GB"
                  :key="`ram-${value}`"
                  :label="String(value)"
                  :value="value"
                />
              </el-select>
              <span class="inline-unit">GB</span>
            </div>
          </el-col>
          <el-col :span="12">
            <div class="inline-field">
              <span class="inline-label">磁盘</span>
              <el-select
                v-model="props.editingHost.disk_gb"
                filterable
                allow-create
                default-first-option
                clearable
                placeholder="512"
                class="inline-input"
              >
                <el-option
                  v-for="value in COMMON_DISK_OPTIONS_GB"
                  :key="`disk-${value}`"
                  :label="String(value)"
                  :value="value"
                />
              </el-select>
              <span class="inline-unit">GB</span>
            </div>
          </el-col>
        </el-row>
      </el-form-item>

      <el-divider content-position="left">运维信息</el-divider>
      <el-row :gutter="12">
        <el-col :span="12">
          <el-form-item label="状态" prop="status" required>
            <el-select v-model="props.editingHost.status" class="w-full">
              <el-option
                v-for="option in props.statusOptions"
                :key="option.value"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item label="标签">
            <el-select
              v-model="tagListModel"
              multiple
              filterable
              allow-create
              default-first-option
              :reserve-keyword="false"
              class="w-full"
              placeholder="输入标签后回车"
            >
              <el-option
                v-for="option in props.formTagSuggestionOptions"
                :key="`host-tag-${option.value}`"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item label="描述">
        <el-input
          v-model="props.editingHost.description"
          type="textarea"
          :rows="3"
          placeholder="可补充用途、机房位置、负责人等信息"
          show-word-limit
          maxlength="300"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="dialogVisibleModel = false">取消</el-button>
      <el-button data-testid="host-save-btn" type="primary" :loading="props.saveLoading" @click="submitHostForm">
        保存
      </el-button>
    </template>
  </el-dialog>

  <el-dialog
    v-model="quickIpDialogVisibleModel"
    title="新增IP资源"
    width="620px"
    align-center
    destroy-on-close
  >
    <el-form :ref="bindQuickFormRef" :model="props.quickIpForm" :rules="props.quickIpFormRules" label-width="96px">
      <el-form-item label="IP地址" prop="ip_address" required>
        <el-input v-model="props.quickIpForm.ip_address" placeholder="如 10.0.0.21" />
      </el-form-item>
      <el-form-item label="环境" prop="env" required>
        <el-select v-model="props.quickIpForm.env" class="w-full">
          <el-option
            v-for="option in props.envOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="是否VIP">
        <el-radio-group v-model="props.quickIpForm.is_vip">
          <el-radio :value="false">否</el-radio>
          <el-radio :value="true">是</el-radio>
        </el-radio-group>
      </el-form-item>
      <el-form-item v-if="props.quickIpForm.is_vip" label="真实IP列表" required>
        <div class="quick-real-ip-editor">
          <div
            v-for="(_ip, index) in props.quickRealIpList"
            :key="`quick-real-ip-${index}`"
            class="quick-real-ip-row"
          >
            <el-input v-model="props.quickRealIpList[index]" placeholder="如 10.0.0.31" />
            <el-button text type="danger" @click="emit('remove-quick-real-ip', index)">删除</el-button>
          </div>
          <el-button size="small" @click="emit('add-quick-real-ip')">+ 添加真实IP</el-button>
        </div>
      </el-form-item>
      <el-form-item label="描述">
        <el-input
          v-model="props.quickIpForm.description"
          type="textarea"
          :rows="3"
          maxlength="300"
          show-word-limit
          placeholder="可填写用途、备注等信息"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="quickIpDialogVisibleModel = false">取消</el-button>
      <el-button data-testid="quick-ip-save-btn" type="primary" :loading="props.quickIpSaving" @click="submitQuickIpForm">
        保存并回填
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.inline-field {
  display: flex;
  align-items: flex-start;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.inline-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.2;
  min-width: 40px;
  white-space: nowrap;
}

.inline-unit {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.2;
}

.inline-input {
  width: 100%;
}

.binding-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.binding-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.binding-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.binding-empty {
  padding: 8px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  text-align: center;
}

.quick-real-ip-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.quick-real-ip-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
}

@media (max-width: 768px) {
  .binding-toolbar {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
}
</style>
