<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import type { Dependency, Application, Middleware, NginxConfig } from "@/types";
import { listDependencies, saveDependency, softDeleteDependency } from "@/api/dependencies";
import { listApplications } from "@/api/applications";
import { listMiddlewares } from "@/api/middlewares";
import { listNginxConfigs } from "@/api/nginx-configs";

const props = defineProps<{
  resourceId: string;
  resourceType: string;
}>();

const dependencies = ref<Dependency[]>([]);
const loading = ref(false);
const addVisible = ref(false);
const saveLoading = ref(false);

interface ResourceOption {
  id: string;
  name: string;
  type: string;
}

const targetOptions = ref<ResourceOption[]>([]);
const newDep = ref<{
  target_id: string;
  target_type: string;
  relation_type: NonNullable<Dependency["relation_type"]>;
  description: string;
}>({
  target_id: "",
  target_type: "",
  relation_type: "http_call",
  description: "",
});

async function fetchDependencies() {
  if (!props.resourceId) return;
  loading.value = true;
  try {
    const result = await listDependencies({
      page: 1,
      page_size: 100,
      filters: { source_id: props.resourceId },
    });
    dependencies.value = result.data;
  } catch {
    // error shown
  } finally {
    loading.value = false;
  }
}

async function fetchTargetOptions() {
  const results: ResourceOption[] = [];
  try {
    const [apps, mws, ngs] = await Promise.all([
      listApplications({ page: 1, page_size: 999 }),
      listMiddlewares({ page: 1, page_size: 999 }),
      listNginxConfigs({ page: 1, page_size: 999 }),
    ]);
    apps.data.forEach((a: Application) => results.push({ id: a.id, name: a.name, type: "application" }));
    mws.data.forEach((m: Middleware) => results.push({ id: m.id, name: m.name, type: "middleware" }));
    ngs.data.forEach((n: NginxConfig) => results.push({ id: n.id, name: n.name, type: "nginx" }));
  } catch {
    // ignore
  }
  targetOptions.value = results.filter((r) => r.id !== props.resourceId);
}

function targetName(targetId: string) {
  const t = targetOptions.value.find((o) => o.id === targetId);
  return t ? t.name : targetId;
}

function relationLabel(type: string) {
  return (
    ({
      http_call: "HTTP调用",
      tcp: "TCP连接",
      mq_produce: "MQ生产",
      mq_consume: "MQ消费",
      grpc_call: "gRPC调用",
      db_query: "数据库访问",
      cache_access: "缓存访问",
    } as Record<string, string>)[type] || type
  );
}

function openAdd() {
  newDep.value = { target_id: "", target_type: "", relation_type: "http_call", description: "" };
  addVisible.value = true;
}

function onTargetChange(targetId: string) {
  const t = targetOptions.value.find((o) => o.id === targetId);
  if (t) {
    newDep.value.target_type = t.type;
  }
}

async function handleAdd() {
  if (!newDep.value.target_id) {
    ElMessage.warning("请选择目标资源");
    return;
  }
  saveLoading.value = true;
  try {
    await saveDependency({
      id: "",
      source_id: props.resourceId,
      source_type: props.resourceType,
      target_id: newDep.value.target_id,
      target_type: newDep.value.target_type,
      relation_type: newDep.value.relation_type,
      description: newDep.value.description || undefined,
    });
    ElMessage.success("依赖关系添加成功");
    addVisible.value = false;
    fetchDependencies();
  } catch {
    // error shown
  } finally {
    saveLoading.value = false;
  }
}

async function handleRemove(dep: Dependency) {
  try {
    await ElMessageBox.confirm("确认删除此依赖关系?", "确认", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      type: "warning",
    });
    await softDeleteDependency(dep.id);
    ElMessage.success("已删除");
    fetchDependencies();
  } catch {
    // cancelled
  }
}

watch(() => props.resourceId, fetchDependencies);
onMounted(() => {
  fetchDependencies();
  fetchTargetOptions();
});
</script>

<template>
  <div class="dependency-panel">
    <div class="panel-header">
      <span class="panel-title">调用关系 (下游依赖)</span>
      <el-button text type="primary" size="small" @click="openAdd">添加</el-button>
    </div>

    <el-table :data="dependencies" v-loading="loading" size="small" stripe max-height="250">
      <el-table-column label="目标资源" min-width="150">
        <template #default="{ row }">{{ targetName(row.target_id) }}</template>
      </el-table-column>
      <el-table-column prop="target_type" label="类型" width="90" align="center" />
      <el-table-column prop="relation_type" label="关系" width="100" align="center">
        <template #default="{ row }">
          <el-tag size="small">{{ relationLabel(row.relation_type) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="70" align="center">
        <template #default="{ row }">
          <el-button text type="danger" size="small" @click="handleRemove(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty
      v-if="!dependencies.length && !loading"
      description="暂无依赖关系"
      :image-size="40"
    />

    <el-dialog v-model="addVisible" title="添加依赖关系" width="450px" append-to-body>
      <el-form :model="newDep" label-width="90px">
        <el-form-item label="目标资源" required>
          <el-select
            v-model="newDep.target_id"
            filterable
            placeholder="选择目标资源"
            class="w-full"
            @change="onTargetChange"
          >
            <el-option-group
              v-for="group in [
                { label: '应用', type: 'application' },
                { label: '中间件', type: 'middleware' },
                { label: '负载均衡', type: 'nginx' },
              ]"
              :key="group.type"
              :label="group.label"
            >
              <el-option
                v-for="opt in targetOptions.filter((o) => o.type === group.type)"
                :key="opt.id"
                :label="opt.name"
                :value="opt.id"
              />
            </el-option-group>
          </el-select>
        </el-form-item>
        <el-form-item label="关系类型" required>
          <el-select v-model="newDep.relation_type" class="w-full">
            <el-option label="HTTP调用" value="http_call" />
            <el-option label="TCP连接" value="tcp" />
            <el-option label="MQ生产" value="mq_produce" />
            <el-option label="MQ消费" value="mq_consume" />
            <el-option label="gRPC调用" value="grpc_call" />
            <el-option label="数据库访问" value="db_query" />
            <el-option label="缓存访问" value="cache_access" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="newDep.description" type="textarea" :rows="2" />
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
.dependency-panel {
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
</style>
