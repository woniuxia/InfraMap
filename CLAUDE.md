# InfraMap 仓库手册

## 1. 文档定位

- 本文件记录 InfraMap 当前仓库事实、目录结构、架构入口与模块分布，供开发者和代码代理快速建立上下文。
- 协作规则、验证基线、危险操作和提交流程见 `AGENTS.md`。
- `AGENTS.md` 与本文件都保留核心仓库事实；若脚本、测试入口、目录结构或构建方式变更，需同步更新两份文档。

## 2. 项目概述

- InfraMap 是一个完全离线的桌面应用，用于管理和可视化基础设施资源及其依赖关系。
- 面向对象主要是 DevOps 工程师、开发者和架构师，默认中文 UI，主要目标平台是 Windows。
- 前端通过 Vue 3 + Tauri 构建桌面界面，后端使用 Rust + SQLite 提供本地数据存储与查询能力。

## 3. 技术栈

### 3.1 前端

- Vue 3（Composition API + `<script setup lang="ts">`）
- TypeScript 5.9，`tsconfig.json` 开启 `strict`
- Vite 7
- Element Plus 2.13
- Pinia 3
- Vue Router 4
- Cytoscape.js 3，配套 `cytoscape-dagre`、`cytoscape-fcose`、`cytoscape-svg`、`dagre`
- `unplugin-auto-import` 与 `unplugin-vue-components`

### 3.2 桌面与后端

- Tauri 2.10
- Rust 2021
- rusqlite 0.32（启用 `bundled`、`backup`）
- r2d2 / r2d2_sqlite
- SQLite
- `tauri-plugin-dialog`
- `tauri-plugin-opener`

### 3.3 测试与工具链

- Vitest 4 + jsdom
- Playwright
- ESLint 8
- Prettier 3
- Husky 9 + lint-staged 15
- pnpm

## 4. 常用命令

- `pnpm dev`：启动前端开发服务器，默认 `localhost:15420`。
- `pnpm preview`：启动 Vite 预览服务器。
- `pnpm tauri dev`：启动 Tauri 开发模式。
- `pnpm build`：执行类型检查并构建前端。
- `pnpm build:installer`：执行 `tauri build`，构建安装包。
- `pnpm lint`：运行 ESLint。
- `pnpm lint:fix`：运行 ESLint 自动修复。
- `pnpm format`：执行 Prettier 写回格式。
- `pnpm format:check`：检查 Prettier 格式。
- `pnpm test`：运行前端单元测试。
- `pnpm test:watch`：Vitest 监听模式。
- `pnpm test:e2e`：运行 Playwright 端到端测试。
- `pnpm test:backend`：进入 `src-tauri` 执行 Rust 测试。
- `pnpm sync-version <version>`：同步版本号到前端与 Tauri 配置。
- `pnpm build:win`：调用 `scripts/build-tauri-win.ps1` 执行 Windows 构建。
- `pnpm release:win -Tag "vX.Y.Z"`：调用 `scripts/release-win.ps1` 执行构建、打包与发布。
- `pnpm prepare`：安装 Husky hooks。

## 5. 目录结构

### 5.1 根目录

- `src/`：前端业务代码。
- `src-tauri/`：Rust/Tauri 后端。
- `tests/e2e/`：Playwright 端到端测试。
- `scripts/`：版本同步与 Windows 构建发布脚本。
- `public/`：前端静态资源。
- `.husky/`：Git hooks。
- `.husky/_/`：Husky 辅助目录。
- `lint-staged.config.mjs`：暂存文件格式化与 lint 规则。
- `playwright.config.ts`、`vitest.config.ts`：前端测试入口配置。
- `infraMap.svg`、`app-icon.png`、`gen_icon.py`：图标源与生成脚本。
- `AGENTS.md`：协作规范。
- `CLAUDE.md`：仓库手册。

### 5.2 `src/`

- `api/`：Tauri 命令的类型化封装；`src/api/index.ts` 负责统一导出。
- `assets/`：静态样式资源与前端素材。
- `components/`：可复用业务组件，包括拓扑、编辑器、筛选器、手册分节等。
- `composables/`：组合式逻辑，例如错误展示、列表交互等。
- `constants/`：常量定义。
- `docs/`：前端内嵌说明内容。
- `icons/`：图标注册与解析。
- `layouts/`：页面布局与导航框架。
- `router/`：Vue Router 路由。
- `stores/`：Pinia store。
- `styles/`：主题变量、全局样式、反馈样式。
- `types/`：TypeScript 类型定义与声明扩展。
- `utils/`：工具函数与 Tauri 调用封装。
- `views/`：页面级视图，当前包含仪表盘、主机、IP、系统、服务、中间件、网关、联系人、批量录入、任务中心、数据健康、拓扑图、设置、使用手册等页面。
- `__mocks__/tauri.ts`：Vitest 的 Tauri mock。
- `auto-imports.d.ts`、`components.d.ts`：自动导入生成的声明文件。

### 5.3 `src-tauri/src/`

- `commands/`：Tauri 命令处理函数，按资源与能力分文件。
- `db/`：数据库层，包含连接池、CRUD 辅助、事务、迁移、审计与 schema 常量。
- `models/`：序列化模型与返回结构。
- `backup.rs`：备份、恢复、JSON 导入导出相关逻辑。
- `error.rs`：结构化错误定义与运行时日志输出。
- `storage.rs`：数据库和备份目录路径解析与迁移。
- `validation.rs`：输入校验逻辑。
- `lib.rs`：应用 setup、依赖注入、命令注册入口。
- `main.rs`：Tauri 二进制入口。
- `test_helpers.rs`：Rust 测试辅助方法。

## 6. 前端架构约定

### 6.1 应用入口与路由

- `src/main.ts` 创建 Vue 应用，注册 Pinia 与 Router，并在缺省情况下将 `document.documentElement.dataset.theme` 设为 `light`。
- 路由入口是 `src/router/index.ts`，当前主布局是 `DefaultLayout`。
- 当前一级业务页面包括：
- `DashboardView`
- `HostsView`
- `IpAddressesView`
- `SystemsView`
- `ServicesView`
- `MiddlewaresView`
- `NginxConfigsView`
- `ContactsView`
- `ImportWorkbenchView`
- `JobCenterView`
- `IntegrityCenterView`
- `TopologyView`
- `SettingsView`
- `ManualView`

### 6.2 API 与错误处理

- 所有业务侧 Tauri 调用统一通过 `src/utils/invoke.ts` 的 `tauriInvoke<T>()` 封装。
- `tauriInvoke()` 内部调用 `@tauri-apps/api/core` 的 `invoke`，并统一走错误归一化与展示流程。
- `src/api/*.ts` 负责资源级 API 封装，按资源或能力拆分文件，并通过 `src/api/index.ts` 聚合导出。
- 测试中的原生调用统一通过 `src/__mocks__/tauri.ts` 模拟。

### 6.3 主题与样式

- 主题变量集中在 `src/styles/variables.scss`，统一使用 `--im-*` token。
- 主题切换基于 `html[data-theme="light|dark"]`。
- `src/styles/global.scss` 提供全局背景、滚动条、固定列、焦点态等样式。
- `src/styles/element-plus-feedback.css` 补充 Element Plus 的反馈样式映射。

### 6.4 构建与自动导入

- `vite.config.ts` 配置了 `@` 指向 `src/` 的别名。
- `unplugin-auto-import` 自动导入 Vue、Vue Router、Pinia API。
- `unplugin-vue-components` 配合 `ElementPlusResolver` 自动注册组件。
- 自动导入声明文件输出到 `src/auto-imports.d.ts` 与 `src/components.d.ts`。
- Vite 构建对 Cytoscape 与拓扑相关模块做了手动分包，减少主包体积。

## 7. 后端架构约定

### 7.1 应用启动与命令注册

- `src-tauri/src/lib.rs` 在应用 setup 阶段完成：
- 解析存储路径。
- 初始化 SQLite 连接池。
- 创建迁移前备份。
- 执行数据库迁移。
- 注入 `DbPool` 与 `StoragePaths`。
- 启动自动备份线程。
- Tauri 命令通过 `tauri::generate_handler!` 注册，当前覆盖资源 CRUD、部署关系、调用关系、拓扑分析、设置、任务中心、数据健康、批量导入、快照、分类术语、备份恢复等能力。

### 7.2 错误模型

- 后端统一使用 `AppResult<T> = Result<T, AppError>`。
- `AppError` 包含 `code`、`message`、`details`、`command`、`retryable` 等字段，支持前端做统一展示和重试判断。
- 运行时错误与告警通过 `error.rs` 中的日志辅助函数写入标准错误输出。

### 7.3 数据库层

- 连接池定义在 `src-tauri/src/db/pool.rs`，当前启用：
- `PRAGMA journal_mode=WAL`
- `PRAGMA foreign_keys=ON`
- `PRAGMA busy_timeout=5000`
- 连接池默认 `max_size(8)`。
- `src-tauri/src/db` 目前包含：
- `audit.rs`
- `crud.rs`
- `macros.rs`
- `migration.rs`
- `migrations/`
- `pool.rs`
- `schema.rs`
- `transaction.rs`

### 7.4 迁移与存储

- 当前迁移目录为 `src-tauri/src/db/migrations`，已存在：
- `v001_init.rs`
- `v002_rename_to_systems.rs`
- `v003_topology_node_positions.rs`
- `v004_add_focus_target.rs`
- `hooks.rs`
- 存储路径逻辑位于 `storage.rs`：
- 默认数据库文件名为 `inframap.db`
- 备份目录名为 `backups`
- 引导配置文件名为 `storage-bootstrap.json`
- 支持自定义存储根目录与存储迁移

## 8. 数据与领域事实

### 8.1 当前核心实体

- 主机：`hosts`
- IP 地址与绑定：`ip_addresses`、`host_ip_bindings`
- 系统与服务：`systems`、`services`
- 中间件与网关：`middlewares`、`nginx_configs`
- 部署关系：`deployments`
- 调用关系：`call_relations`
- 联系人：`contacts`
- 分类术语：`taxonomy_terms`、`taxonomy_bindings`、`taxonomy_term_stats`
- 审计日志：`audit_logs`
- 系统设置与任务：`system_settings`、`system_jobs`
- 批量导入：`import_jobs`、`import_job_rows`、`import_job_issues`
- 拓扑节点位置：`topology_node_positions`

### 8.2 数据层通用约定

- 当前数据库以软删除为主，业务表普遍包含 `is_deleted`、`deleted_at` 字段。
- 查询逻辑通常默认过滤 `is_deleted = 0`。
- 删除流程普遍通过数据库辅助层与审计逻辑收敛处理。
- 拓扑、导入、完整性检查、备份恢复等能力依赖同一套本地 SQLite 数据。

### 8.3 常见唯一约束

- `systems(name, env)`：系统名称与环境组合唯一。
- `services(name, env, type)`：服务名称、环境、类型组合唯一。
- `middlewares(name, env)`：中间件名称与环境组合唯一。
- `nginx_configs(name, env)`：网关名称与环境组合唯一。
- `ip_addresses(ip_address, env)`：IP 与环境组合唯一。
- `host_ip_bindings(host_id, ip_id)`：主机与 IP 绑定唯一。
- `deployments(resource_id, resource_type, host_id)`：部署关系唯一。
- `call_relations(owner_id, owner_type, peer_id, peer_type, direction, relation_type)`：调用关系唯一。
- `taxonomy_terms(field_key, normalized_value)`：分类词条归一化值唯一。
- `taxonomy_bindings(term_id, resource_type, resource_id)`：词条绑定唯一。
- `import_job_rows(job_id, row_no)`：导入作业内的行号唯一。

## 9. 测试约定

- 前端单测使用 Vitest + jsdom。
- 测试文件通常与源码同目录，后缀为 `*.test.ts`。
- Tauri 调用在前端测试中通过 `src/__mocks__/tauri.ts` 进行 mock。
- Playwright 配置位于 `playwright.config.ts`，测试目录固定为 `tests/e2e`。
- Playwright 会自行拉起 `pnpm dev --host 127.0.0.1 --port 4173` 作为测试 Web Server。
- 当前 e2e 用例覆盖删除确认布局与调用关系目标标签等关键场景。
- Rust 后端测试通过 `pnpm test:backend` 执行，测试辅助逻辑集中在 `src-tauri/src/test_helpers.rs`。

## 10. 当前模块地图

- 资源管理：主机、IP、系统、服务、中间件、网关、联系人。
- 关系管理：部署关系、调用关系、系统与服务关联、主机与 IP 绑定。
- 分析能力：拓扑图、路径分析、影响面、故障排查报告、聚焦视图。
- 数据治理：完整性检查、批量导入、任务中心、分类术语。
- 运维能力：设置、存储路径迁移、备份恢复、JSON 导入导出、使用手册。

## 11. 构建与发布补充

- `src-tauri/tauri.conf.json` 当前配置：
- `beforeDevCommand` 使用 `pnpm dev`
- `beforeBuildCommand` 使用 `pnpm build`
- 安装包目标是 `nsis` 和 `msi`
- 图标由 `src-tauri/icons/*` 提供
- Windows NSIS 语言包含简体中文和英文
- `pnpm sync-version` 会同步更新 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`
- `scripts/README.md` 记录了 `sync-version.ps1`、`build-tauri-win.ps1`、`release-win.ps1` 的参数、前置要求与产物说明。

## 12. 协作提示

- 改动协作规则、验证策略、危险操作与提交流程时，优先更新 `AGENTS.md`。
- 改动命令脚本、目录结构、模块入口、架构事实与发布流程时，必须同步更新本文件和 `AGENTS.md`。
- 如果文档描述与仓库实现冲突，以仓库代码和配置为准，并在同轮工作中修正文档。
