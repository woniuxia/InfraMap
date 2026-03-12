<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { FormInstance, FormRules } from "element-plus";
import { ElMessage } from "element-plus";
import { Phone, Message, Edit, Delete, Grid, List as ListIcon } from "@element-plus/icons-vue";
import type { Contact } from "@/types";
import type { SearchFieldConfig, SearchToolbarQueryPayload } from "@/types/searchToolbar";
import { listContacts, saveContact, deleteContact } from "@/api/contacts";
import { useResourceList } from "@/composables/useResourceList";
import { useViewToggle } from "@/composables/useViewToggle";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import ContactCard from "@/components/contact/ContactCard.vue";
import ContactAvatar from "@/components/contact/ContactAvatar.vue";

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

const { currentView, isCardView } = useViewToggle({
  storageKey: "contacts-view-mode",
  defaultView: "card",
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
  company: [{ min: 0, max: 100, message: "单位长度需在 0-100 字符内", trigger: "blur" }],
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
    company: undefined,
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

function handleEditFromCard(contact: Contact) {
  openEdit(contact);
}

function handleDeleteFromCard(contact: Contact) {
  handleDelete(contact.id, contact.name);
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
    company: normalizeOptionalText(editingContact.value.company),
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
      search-placeholder="搜索姓名/单位/电话/邮箱/备注..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <div class="toolbar-actions">
          <el-radio-group v-model="currentView" size="small" class="view-toggle">
            <el-radio-button label="card">
              <el-icon><Grid /></el-icon>
            </el-radio-button>
            <el-radio-button label="list">
              <el-icon><ListIcon /></el-icon>
            </el-radio-button>
          </el-radio-group>
          <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
          <el-button type="primary" @click="openAdd">新增联系人</el-button>
        </div>
      </template>
    </SearchToolbar>

    <!-- Card View -->
    <div v-if="isCardView" v-loading="loading" class="contact-grid">
      <ContactCard
        v-for="contact in data"
        :key="contact.id"
        :contact="contact"
        @edit="handleEditFromCard"
        @delete="handleDeleteFromCard"
      />
      <el-empty v-if="!loading && data.length === 0" description="暂无联系人" class="contact-empty">
        <el-button type="primary" @click="openAdd">添加第一个联系人</el-button>
      </el-empty>
    </div>

    <!-- List View -->
    <el-table
      v-else
      :data="data"
      v-loading="loading"
      border
      stripe
      class="w-full im-table-fixed-ops"
    >
      <el-table-column label="姓名" min-width="200">
        <template #default="{ row }">
          <div class="contact-name-cell">
            <ContactAvatar :name="row.name" :size="36" />
            <div class="contact-name-info">
              <span class="name">{{ row.name }}</span>
              <span v-if="row.company" class="company">{{ row.company }}</span>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="联系方式" min-width="180">
        <template #default="{ row }">
          <div class="contact-info-cell">
            <div v-if="row.phone" class="info-item">
              <el-icon><Phone /></el-icon>
              <span>{{ row.phone }}</span>
            </div>
            <div v-if="row.email" class="info-item">
              <el-icon><Message /></el-icon>
              <span>{{ row.email }}</span>
            </div>
            <span v-if="!row.phone && !row.email">-</span>
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="remark" label="备注" min-width="200" show-overflow-tooltip>
        <template #default="{ row }">{{ row.remark || "-" }}</template>
      </el-table-column>
      <el-table-column prop="updated_at" label="更新时间" width="150" align="center">
        <template #default="{ row }">{{ row.updated_at || "-" }}</template>
      </el-table-column>
      <el-table-column label="操作" width="100" fixed="right" align="center">
        <template #default="{ row }">
          <el-button link type="primary" size="small" @click="openEdit(row)">
            <el-icon><Edit /></el-icon>
          </el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row.id, row.name)">
            <el-icon><Delete /></el-icon>
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrapper">
      <el-pagination
        v-model:current-page="queryParams.page"
        v-model:page-size="queryParams.page_size"
        :total="total"
        :page-sizes="[12, 24, 48, 96]"
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
        <el-form-item label="单位" prop="company">
          <el-input
            v-model="editingContact.company"
            placeholder="公司/组织（可选）"
            maxlength="100"
          />
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

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.view-toggle {
  .el-radio-button__inner {
    padding: 6px 12px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
}

.contact-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
  min-height: 200px;
}

.contact-empty {
  grid-column: 1 / -1;
  padding: 60px 0;
}

.contact-name-cell {
  display: flex;
  align-items: center;
  gap: 12px;

  .contact-name-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-weight: 500;
    color: var(--el-text-color-primary);
  }

  .company {
    font-size: 12px;
    color: var(--el-text-color-secondary);
  }
}

.contact-info-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .info-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--el-text-color-regular);

    .el-icon {
      font-size: 14px;
      color: var(--el-text-color-secondary);
    }
  }
}

@media (max-width: 1400px) {
  .contact-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (max-width: 1100px) {
  .contact-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .contact-grid {
    grid-template-columns: 1fr;
  }

  .toolbar-actions {
    flex-wrap: wrap;
    gap: 8px;
  }
}
</style>
