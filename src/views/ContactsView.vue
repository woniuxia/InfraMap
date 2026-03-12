<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { FormInstance, FormRules } from "element-plus";
import { ElMessage } from "element-plus";
import type { Contact } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listContacts, saveContact, deleteContact } from "@/api/contacts";
import { useResourceList } from "@/composables/useResourceList";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";

const {
  loading,
  data,
  total,
  queryParams,
  fetchData,
  handleQuery,
  handlePageChange,
  handlePageSizeChange,
  handleDelete,
} = useResourceList<Contact>({
  listFn: listContacts,
  deleteFn: deleteContact,
  entityLabel: "联系人",
});

const searchText = ref("");
const listFilters = ref<Record<string, string | string[]>>({});
const toolbarFields = computed<SearchFieldConfig[]>(() => []);

const dialogVisible = ref(false);
const isEditing = ref(false);
const saveLoading = ref(false);
const formRef = ref<FormInstance>();
const editingContact = ref<Partial<Contact>>({});

const dialogTitle = computed(() => (isEditing.value ? "编辑联系人" : "新增联系人"));

const formRules: FormRules = {
  name: [
    { required: true, message: "请输入联系人姓名", trigger: "blur" },
    { min: 1, max: 100, message: "姓名长度需在 1-100 字符内", trigger: "blur" },
  ],
  phone: [{ min: 0, max: 50, message: "电话长度需在 0-50 字符内", trigger: "blur" }],
  email: [{ min: 0, max: 100, message: "邮箱长度需在 0-100 字符内", trigger: "blur" }],
  remark: [{ min: 0, max: 300, message: "备注长度需在 0-300 字符内", trigger: "blur" }],
};

function normalizeOptionalText(value: unknown) {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function handleToolbarQuery(payload: SearchToolbarQueryPayload) {
  handleQuery(payload);
}

function openAdd() {
  isEditing.value = false;
  editingContact.value = {
    id: "",
    name: "",
    phone: undefined,
    email: undefined,
    remark: undefined,
  };
  dialogVisible.value = true;
}

function openEdit(row: Contact) {
  isEditing.value = true;
  editingContact.value = { ...row };
  dialogVisible.value = true;
}

async function handleSave() {
  if (saveLoading.value) return;

  const hasValidator = typeof formRef.value?.validate === "function";
  if (hasValidator) {
    const valid = await formRef.value?.validate().catch(() => false);
    if (!valid) return;
  }

  const payload: Partial<Contact> = {
    id: typeof editingContact.value.id === "string" ? editingContact.value.id.trim() : "",
    name: String(editingContact.value.name ?? "").trim(),
    phone: normalizeOptionalText(editingContact.value.phone),
    email: normalizeOptionalText(editingContact.value.email),
    remark: normalizeOptionalText(editingContact.value.remark),
  };

  if (!payload.name) {
    ElMessage.warning("请输入联系人姓名");
    return;
  }

  saveLoading.value = true;
  try {
    const id = await saveContact(payload);
    ElMessage.success(isEditing.value ? "更新成功" : "创建成功");
    editingContact.value.id = id;
    dialogVisible.value = false;
    await fetchData();
  } catch {
    // error shown by tauriInvoke
  } finally {
    saveLoading.value = false;
  }
}

onMounted(() => {
  fetchData();
});
</script>

<template>
  <div class="resource-view">
    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索姓名/电话/邮箱/备注..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增联系人</el-button>
      </template>
    </SearchToolbar>

    <el-table :data="data" v-loading="loading" border stripe class="w-full im-table-fixed-ops">
      <el-table-column prop="name" label="姓名" min-width="140" show-overflow-tooltip />
      <el-table-column prop="phone" label="电话" min-width="140" show-overflow-tooltip>
        <template #default="{ row }">{{ row.phone || "-" }}</template>
      </el-table-column>
      <el-table-column prop="email" label="邮箱" min-width="180" show-overflow-tooltip>
        <template #default="{ row }">{{ row.email || "-" }}</template>
      </el-table-column>
      <el-table-column prop="remark" label="备注" min-width="220" show-overflow-tooltip>
        <template #default="{ row }">{{ row.remark || "-" }}</template>
      </el-table-column>
      <el-table-column prop="updated_at" label="更新时间" width="170" align="center">
        <template #default="{ row }">{{ row.updated_at || "-" }}</template>
      </el-table-column>
      <el-table-column label="操作" width="150" fixed="right" align="center">
        <template #default="{ row }">
          <el-button text type="primary" size="small" @click="openEdit(row)">编辑</el-button>
          <el-button text type="danger" size="small" @click="handleDelete(row.id, row.name)">
            删除
          </el-button>
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

    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="520px" destroy-on-close>
      <el-form ref="formRef" :model="editingContact" :rules="formRules" label-width="92px">
        <el-form-item label="姓名" prop="name" required>
          <el-input v-model="editingContact.name" placeholder="请输入联系人姓名" maxlength="100" />
        </el-form-item>
        <el-form-item label="电话" prop="phone">
          <el-input v-model="editingContact.phone" placeholder="可选" maxlength="50" />
        </el-form-item>
        <el-form-item label="邮箱" prop="email">
          <el-input v-model="editingContact.email" placeholder="可选" maxlength="100" />
        </el-form-item>
        <el-form-item label="备注" prop="remark">
          <el-input
            v-model="editingContact.remark"
            type="textarea"
            :rows="3"
            maxlength="300"
            show-word-limit
            placeholder="可选"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saveLoading" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.resource-view {
  padding: 0;
}
</style>
