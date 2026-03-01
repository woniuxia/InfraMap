<script setup lang="ts">
import { onMounted } from "vue";
import SearchToolbar from "@/components/filters/SearchToolbar.vue";
import HostEditorDialog from "@/views/hosts/HostEditorDialog.vue";
import HostsListTable from "@/views/hosts/HostsListTable.vue";
import { useHostViewModel } from "@/views/hosts/useHostViewModel";

const {
  loading,
  data,
  total,
  queryParams,
  searchText,
  listFilters,
  toolbarFields,
  dialogVisible,
  editingHost,
  isEditing,
  saveLoading,
  formRef,
  formRules,
  selectedIpIds,
  bindingLoading,
  allowCrossEnv,
  quickIpDialogVisible,
  quickIpSaving,
  quickIpFormRef,
  quickRealIpList,
  quickIpForm,
  quickIpFormRules,
  envOptions,
  statusOptions,
  osOptions,
  tagList,
  formTagSuggestionOptions,
  filteredIpOptions,
  searchedIpKeyword,
  canQuickCreateIp,
  handleToolbarQuery,
  handlePageChange,
  handlePageSizeChange,
  handleDelete,
  openAdd,
  openEdit,
  openCopy,
  handleSave,
  formatIpOptionLabel,
  refreshBindingContext,
  handleBindingSearch,
  handleBindingDropdownVisible,
  openQuickCreateIpDialog,
  handleQuickCreateIpSave,
  addQuickRealIp,
  removeQuickRealIp,
  init,
} = useHostViewModel();

const editorRefs = {
  formRef,
  quickIpFormRef,
};

function handleDeleteRequest(payload: { id: string; hostname: string }) {
  handleDelete(payload.id, payload.hostname);
}

onMounted(() => {
  init();
});
</script>

<template>
  <div class="resource-view">
    <SearchToolbar
      v-model:search-text="searchText"
      v-model:filters="listFilters"
      search-placeholder="搜索主机名/IP..."
      :fields="toolbarFields"
      @query="handleToolbarQuery"
    >
      <template #actions="{ hasActiveFilters, reset }">
        <el-button :disabled="!hasActiveFilters" @click="reset">重置筛选</el-button>
        <el-button type="primary" @click="openAdd">新增服务器</el-button>
      </template>
    </SearchToolbar>

    <HostsListTable
      :data="data"
      :loading="loading"
      :total="total"
      :page="queryParams.page || 1"
      :page-size="queryParams.page_size || 20"
      @edit="openEdit"
      @copy="openCopy"
      @delete="handleDeleteRequest"
      @page-change="handlePageChange"
      @page-size-change="handlePageSizeChange"
    />

    <HostEditorDialog
      v-model="dialogVisible"
      :is-editing="isEditing"
      :editing-host="editingHost"
      :form-ref="editorRefs.formRef"
      :form-rules="formRules"
      :env-options="envOptions"
      :status-options="statusOptions"
      :os-options="osOptions"
      :tag-list="tagList"
      :form-tag-suggestion-options="formTagSuggestionOptions"
      :selected-ip-ids="selectedIpIds"
      :binding-loading="bindingLoading"
      :allow-cross-env="allowCrossEnv"
      :filtered-ip-options="filteredIpOptions"
      :searched-ip-keyword="searchedIpKeyword"
      :can-quick-create-ip="canQuickCreateIp"
      :save-loading="saveLoading"
      :format-ip-option-label="formatIpOptionLabel"
      :quick-ip-dialog-visible="quickIpDialogVisible"
      :quick-ip-saving="quickIpSaving"
      :quick-ip-form-ref="editorRefs.quickIpFormRef"
      :quick-ip-form="quickIpForm"
      :quick-ip-form-rules="quickIpFormRules"
      :quick-real-ip-list="quickRealIpList"
      @update:selected-ip-ids="(value) => (selectedIpIds = value)"
      @update:allow-cross-env="(value) => (allowCrossEnv = value)"
      @update:tag-list="(value) => (tagList = value)"
      @update:quick-ip-dialog-visible="(value) => (quickIpDialogVisible = value)"
      @save="handleSave"
      @binding-search="handleBindingSearch"
      @binding-dropdown-visible="handleBindingDropdownVisible"
      @refresh-binding-context="refreshBindingContext"
      @open-quick-create-ip="openQuickCreateIpDialog"
      @save-quick-ip="handleQuickCreateIpSave"
      @add-quick-real-ip="addQuickRealIp"
      @remove-quick-real-ip="removeQuickRealIp"
    />
  </div>
</template>

<style scoped lang="scss">
.resource-view {
  padding: 0;
}
</style>
