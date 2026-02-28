# InfraMap - 基础设施依赖管理平台

## 项目概述

完全离线的桌面应用，用于管理和可视化基础设施组件之间的依赖关系。面向 DevOps 工程师、开发者和架构师。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 (Composition API + `<script setup>`) |
| 类型系统 | TypeScript 5.9 (strict) |
| UI 组件库 | Element Plus 2.13 |
| 图可视化 | AntV G6 5.x |
| 状态管理 | Pinia 3 (Setup Function 模式) |
| 路由 | Vue Router 4 |
| 构建工具 | Vite 7 |
| 桌面运行时 | Tauri 2.10 |
| 后端语言 | Rust (2021 edition) |
| 数据库 | SQLite (rusqlite 0.32 + r2d2 连接池) |
| 包管理器 | pnpm |

## 常用命令

```bash
pnpm dev              # 启动前端开发服务器 (localhost:1420)
pnpm build            # TypeScript 检查 + Vite 构建
pnpm test             # 运行前端测试 (Vitest)
pnpm test:watch       # 前端测试监听模式
pnpm test:backend     # 运行 Rust 测试 (cd src-tauri && cargo test)
pnpm build:installer  # 构建桌面安装包 (NSIS/MSI)
```

## 目录结构

```
src/                          # 前端
  api/                        # Tauri 命令封装 (每个资源一个文件)
  components/                 # Vue 组件
  composables/                # 可复用组合式函数
  layouts/                    # 布局组件 (DefaultLayout)
  router/                     # 路由定义
  stores/                     # Pinia 状态仓库
  styles/                     # 全局样式
  types/index.ts              # TypeScript 接口定义 (集中管理)
  utils/invoke.ts             # Tauri invoke 统一封装
  views/                      # 页面组件
  __mocks__/tauri.ts          # 测试用 Tauri mock

src-tauri/                    # 后端
  src/
    commands/                 # Tauri 命令处理函数 (每个资源一个文件)
    db/                       # 数据库层 (连接池、CRUD 辅助、迁移、Schema)
    models/                   # 数据模型 (Serde 序列化)
    backup.rs                 # 备份/恢复逻辑
    validation.rs             # 输入校验
    lib.rs                    # 入口，注册所有命令
```

## 架构约定

### 前端

**API 层** (`src/api/`): 每个资源一个文件，函数封装 `tauriInvoke` 调用，提供完整 TypeScript 类型。所有 API 在 `src/api/index.ts` 统一导出。

**Tauri 调用**: 统一使用 `src/utils/invoke.ts` 的 `tauriInvoke<T>()` 封装，自动处理错误并弹出 `ElMessage.error()` 提示。禁止直接调用 `@tauri-apps/api/core` 的 `invoke`。

**组合式函数** (`src/composables/`): `useResourceList<T>` 封装了分页列表的完整逻辑 (加载、搜索、筛选、分页、删除确认)，所有资源列表页面复用此 composable。

**组件风格**: `<script setup lang="ts">` + `<template>` + `<style scoped lang="scss">`。使用 ref/reactive 管理状态，onMounted 初始化数据。

**自动导入**: Vue/VueRouter/Pinia API 和 Element Plus 组件均通过 unplugin 自动导入，无需手动 import。

**路径别名**: `@/` 映射到 `src/`。

### 后端

**命令模式**: `#[tauri::command]` 函数接收 `State<DbPool>` + 参数，返回 `Result<T, String>`。命令名使用 snake_case。

**CRUD 约定**:
- `list_{resource}(params: QueryParams) -> PagedResult<T>`
- `get_{resource}(id: String) -> T`
- `save_{resource}(data: T) -> ()` (根据 id 是否存在自动 insert/update)
- `soft_delete_{resource}(id: String) -> ()`

**软删除**: 所有实体包含 `is_deleted: i32` (0/1) 和 `deleted_at: Option<String>`。查询默认过滤 `WHERE is_deleted = 0`。

**校验**: `validation.rs` 提供 `validate_{resource}()` 函数，在 save 命令中调用。校验函数有完整单元测试。

**数据库迁移**: `db/schema.rs` 中以 `(version, sql)` 元组数组管理，启动时按序执行，事务保护。

**审计日志**: 所有 CUD 操作自动记录到 `audit_logs` 表。

### 通用

**类型定义**: 前端类型集中在 `src/types/index.ts`，后端模型在 `src-tauri/src/models/` 下按资源分文件。前后端字段命名保持一致 (后端用 `#[serde(rename)]` 适配)。

**错误处理**: 后端将所有错误转为 `String` 返回，前端通过 `tauriInvoke` 统一捕获并显示。

## 测试约定

**前端**: Vitest + jsdom 环境。测试文件与源码同目录，`.test.ts` 后缀。Tauri invoke 通过 `src/__mocks__/tauri.ts` 的 handler 机制 mock。Element Plus 的 `ElMessage` 需在测试中 `vi.mock('element-plus')`。

**后端**: Rust 标准 `#[cfg(test)] mod tests`，测试辅助函数在 `test_helpers.rs`。

## 数据库

SQLite 数据库存储在 OS 标准应用数据目录。r2d2 连接池默认 8 连接。

**核心表**: hosts, applications, middlewares, nginx_configs, deployments, dependencies, audit_logs, system_settings, schema_version。

**唯一约束**: applications/middlewares/nginx_configs 以 (name, env) 唯一；deployments 以 (resource_id, resource_type, host_id) 唯一；dependencies 以 (source_id, target_id, relation_type) 唯一 (均限 is_deleted=0)。

## 注意事项

- 中文 UI，面向国内用户
- 完全离线运行，无外部 API 依赖
- 新增资源类型需同时更新: models -> db/schema (迁移) -> commands -> validation -> 前端 types -> api -> view
- AntV G6 使用 5.x 长期维护版本，非最新大版本
- Windows 平台为主要目标 (NSIS + MSI 安装包)
