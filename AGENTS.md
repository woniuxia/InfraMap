# InfraMap 协作规范（AGENTS）

## 1. 文档定位与优先级

- 本文件是 InfraMap 仓库级协作契约，适用于人类开发者与代码代理。
- 目标是让协作过程可执行、可检查、可追溯，优先保证质量、安全与一致性。
- 规则优先级：安全与数据约束 > 工程与测试基线 > 提交流程 > 风格与体验建议。
- 若外部会话规则与本文件冲突，以仓库事实和本文件约束为准。
- 本文件侧重“怎么协作、怎么验证、什么不能做”；更细的仓库结构、架构入口和模块说明见 `CLAUDE.md`。

## 2. 双文档分工与同步规则

- `AGENTS.md` 与 `CLAUDE.md` 都保留核心仓库事实，但职责不同：
- `AGENTS.md` 负责协作规则、验证基线、风险控制、提交流程。
- `CLAUDE.md` 负责仓库事实、目录结构、架构约定、模块地图。
- 以下内容属于“双写事实”，任何变更都必须在同一提交中同步更新两份文档：
- `package.json` 中的开发、测试、构建、发布脚本。
- `tests/e2e`、`playwright.config.ts`、`vitest.config.ts` 对应的测试入口与运行方式。
- `src/`、`src-tauri/src/`、`scripts/`、`.husky/` 等关键目录结构。
- Tauri/Vite/主题机制等高频协作基线。
- 若两份文档与仓库实现不一致，以代码和配置文件为准，并尽快修正文档。

## 3. 当前仓库基线（执行所需摘要）

### 3.1 产品与技术基线

- 产品形态：完全离线的桌面应用，用于管理和可视化基础设施组件及其依赖关系。
- 前端：Vue 3、TypeScript 5、Vite 7、Element Plus、Pinia、Vue Router。
- 图能力：Cytoscape.js，配套 `cytoscape-dagre`、`cytoscape-fcose`、`cytoscape-svg`、`dagre`。
- 桌面壳：Tauri 2。
- 后端：Rust 2021、rusqlite、r2d2、SQLite。
- 测试：Vitest、Playwright、Rust `cargo test`。

### 3.2 关键目录

- `src/`：前端业务代码。
- `src/api`：Tauri 命令类型化封装，统一从 `src/api/index.ts` 聚合导出。
- `src/components`、`src/composables`、`src/layouts`、`src/router`、`src/stores`、`src/views`：前端核心业务层。
- `src/constants`、`src/docs`、`src/icons`、`src/styles`、`src/types`、`src/utils`：通用常量、说明、图标、样式、类型与工具。
- `src/__mocks__/tauri.ts`：Vitest 使用的 Tauri mock。
- `src/auto-imports.d.ts`、`src/components.d.ts`：自动导入生成的声明文件。
- `public/`：前端静态资源与图标副本。
- `src-tauri/src/commands`、`src-tauri/src/db`、`src-tauri/src/models`：Rust 命令层、数据库层、模型层。
- `src-tauri/src/backup.rs`、`src-tauri/src/error.rs`、`src-tauri/src/storage.rs`、`src-tauri/src/validation.rs`、`src-tauri/src/test_helpers.rs`：后端通用能力。
- `tests/e2e`：Playwright 端到端测试。
- `scripts/`：版本同步、Windows 构建与发布脚本。
- `.husky/pre-commit`：提交前调用 `pnpm exec lint-staged --config lint-staged.config.mjs`。
- 更细的目录和模块分工见 `CLAUDE.md`。

### 3.3 开发与验证命令

- `pnpm dev`：启动前端开发服务器，默认 `localhost:15420`。
- `pnpm preview`：启动前端预览服务器。
- `pnpm tauri dev`：以 Tauri 开发模式运行桌面应用。
- `pnpm build`：执行 `vue-tsc --noEmit` 后构建前端。
- `pnpm build:installer`：执行 `tauri build`，产出安装包。
- `pnpm lint`：运行 ESLint。
- `pnpm lint:fix`：运行 ESLint 自动修复。
- `pnpm format`：运行 Prettier 写回格式化。
- `pnpm format:check`：检查 Prettier 格式。
- `pnpm test`：运行前端单元测试。
- `pnpm test:watch`：Vitest 监听模式。
- `pnpm test:e2e`：运行 Playwright 端到端测试。
- `pnpm test:backend`：进入 `src-tauri` 执行 Rust 测试。
- `pnpm sync-version <version>`：同步版本号到 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`。
- `pnpm build:win`：调用 `scripts/build-tauri-win.ps1` 执行 Windows 构建。
- `pnpm release:win -Tag "vX.Y.Z"`：调用 `scripts/release-win.ps1` 执行发布流程。
- `pnpm prepare`：安装 Husky hooks。
- 当前 Tauri 打包目标来自 `src-tauri/tauri.conf.json`，为 `nsis` 和 `msi`。

### 3.4 提交前自动化钩子

- `lint-staged.config.mjs` 当前规则：
- 暂存的 `*.ts`、`*.vue` 文件会执行 `pnpm exec eslint --fix` 和 `pnpm exec prettier --write`。
- 暂存的 `*.js`、`*.cjs`、`*.mjs`、`*.json`、`*.md`、`*.scss`、`*.css`、`*.html`、`*.yml`、`*.yaml` 文件会执行 `pnpm exec prettier --write`。
- 修改 lint、format、hook 或脚本行为时，必须同步更新本文件和 `CLAUDE.md`。
- `scripts/README.md` 记录了 `sync-version`、`build:win`、`release:win` 的参数和流程，涉及发布脚本时一并核对。

## 4. 工程规范（前后端）

### 4.1 前端约束

- Vue 页面和组件统一使用 Composition API 与 `<script setup lang="ts">`。
- `src/` 内导入统一使用 `@/` 别名，避免深层相对路径。
- 类型化 Tauri 调用统一经过 `src/utils/invoke.ts` 的 `tauriInvoke()` 封装；业务代码不直接散落原始 `invoke`。
- `src/api/*.ts` 负责资源级 API 封装，新增资源时同步更新 `src/api/index.ts`。
- 自动导入由 `vite.config.ts` 中的 `unplugin-auto-import` 与 `unplugin-vue-components` 提供，`src/auto-imports.d.ts` 和 `src/components.d.ts` 视为生成产物。
- 前端命名：组件/页面使用 `PascalCase.vue`，组合式函数使用 `useXxx.ts`，测试使用 `*.test.ts`。

### 4.2 后端约束

- Tauri 命令与 Rust 函数命名统一使用 `snake_case`。
- 后端统一返回结构化 `AppResult<T>`，错误通过 `src-tauri/src/error.rs` 管理。
- 数据库访问集中在 `src-tauri/src/db`，迁移定义位于 `src-tauri/src/db/migrations`。
- 改动 Rust 代码后执行 `cargo fmt`，并优先补充或更新 `#[cfg(test)]` 覆盖。
- 删除无用代码与失效分支，不保留长期悬挂的“临时兼容”逻辑。

### 4.3 UI 与主题规范

- 统一使用 `--im-*` 设计 token，覆盖颜色、文字、边框、圆角、阴影、动效、字体。
- 主题机制统一使用 `html[data-theme="light|dark"]`。
- Element Plus 主题变量必须由项目 token 映射，不在页面层长期硬编码颜色。
- 可交互元素必须具备 `hover`、`active`、`focus-visible` 三态。
- 响应式必须可用，窄屏双栏自动单列，关键交互高度不低于 `32px`。
- 禁止长期保留大量内联样式和无主题映射的默认 Element Plus 外观。

## 5. 测试策略与验收基线

- 前端单测框架是 Vitest，环境为 `jsdom`。
- 单测中的 Tauri 调用统一通过 `src/__mocks__/tauri.ts` 模拟，禁止真实调用原生 API。
- e2e 基于 Playwright，测试目录固定为 `tests/e2e`。
- Playwright 本地调试通过 `playwright.config.ts` 拉起 `127.0.0.1:4173` 的临时 `webServer`，不要与 Vite 默认开发端口 `15420` 混淆。
- Rust 后端测试通过 `pnpm test:backend` 进入 `src-tauri` 执行 `cargo test`。
- 后台执行长测建议设置超时，默认建议 60s 起，避免任务长期卡死。

### 5.1 提交前验证矩阵

- 仅文档改动：至少检查 `git diff`、Markdown 可读性、双文档事实是否一致。
- 前端页面、组件、API、composable 改动：建议至少执行 `pnpm test`。
- ESLint、Prettier、Vite、TypeScript 配置改动：建议执行 `pnpm lint`、`pnpm format:check`、`pnpm build`。
- 路由、关键交互流程、删除确认、手册页面改动：建议执行 `pnpm test:e2e`。
- Rust 后端、迁移、存储、命令改动：建议执行 `pnpm test:backend`。
- 跨前后端改动：建议至少执行一轮前端测试和后端测试。
- 打包、版本、发布脚本改动：建议至少重新核对 `scripts/README.md`、相关脚本参数和产物路径。

## 6. 提交流程与评审要求

### 6.1 提交前改动分析（必须）

- 提交前至少执行：
- `git status`
- `git diff --stat`
- 关键文件 `git diff` 或 `git diff --cached`
- 确认改动范围与任务目标一致后再提交。

### 6.2 commit message（必须）

- commit message 必须明确说明“改了什么 + 为什么改”。
- 建议格式：`type(scope): summary`
- 例如：`feat(topology): 新增聚焦视图入口以支持局部分析`
- 例如：`docs(agents): 同步脚本矩阵并重构协作文档分工`
- 禁止使用 `update`、`misc`、`tmp`、`test commit` 等无信息说明。
- 单次提交保持聚焦与原子化，避免混入无关改动。

### 6.3 PR 与评审

- PR 说明必须包含：用户可见影响、关联任务或 Issue、验证依据。
- 涉及 UI 的改动必须附截图或录屏，至少覆盖一个关键交互态。
- 涉及数据写入、删除、导入、备份恢复、存储迁移的改动，必须写清风险点与回退方案。
- 较大改动建议先做结构化代码审查，再进入合并流程。

## 7. 多 Agent / 多 CLI 并行协作规范

- 优先拆分为可独立、无写冲突的子任务并行执行。
- 同一文件多处修改时，先按不重叠区域拆分；无法拆分则串行。
- 遇到强依赖链（A -> B -> C）按顺序执行，不强行并行化。
- 信息收集和事实核对阶段优先并行，分析和集成阶段集中收口。
- 每个子任务必须明确输入边界、输出格式和文件归属。
- 发现他人改动默认视为正常协作状态，不得擅自回退、覆盖、丢弃。
- 每轮并行完成后先汇总结果、处理冲突，再进入下一轮拆解。

## 8. 危险操作确认机制

### 8.1 高风险操作清单

- 文件系统：删除文件或目录、批量覆盖、移动关键文件、重写大量文档。
- 系统配置：环境变量、系统权限、系统设置改动。
- 数据操作：数据库删除、结构变更、批量更新、存储迁移。
- 网络请求：向生产环境发送敏感数据或执行写操作。
- 包管理：全局安装或卸载、升级核心依赖、发布流程。

### 8.2 确认模板

在执行高风险操作前，必须先给出以下确认信息：

`⚠️ 危险操作检测！`

`操作类型：` [具体操作]

`影响范围：` [详细说明]

`风险评估：` [潜在后果]

`请确认是否继续？` [是/确认/继续]

## 9. 中文、编码与资源安全

- 所有源码与文档统一使用 UTF-8（无 BOM）+ LF 行尾。
- 禁止使用 GBK、ANSI 等本地编码写入仓库文件。
- 修改含中文文本文件后，必须人工确认中文可读，避免出现乱码。
- PowerShell 覆盖写文件必须显式指定编码，避免默认编码污染。
- 提交前建议执行乱码扫描：
- `rg -n "�|鏇|鍒|缂|璐|鍝|鎿|绫诲瀷|鍦板潃|鐜" src AGENTS.md CLAUDE.md`
- 若文件曾修复乱码，建议检查 BOM：
- `Format-Hex -Path <file> -Count 3`
- 若出现 `EF BB BF`，需移除 BOM 后再提交。

### 9.1 图标与静态资源变更

- 图标源文件与目标文件必须保持一致性，避免只改单端资源：
- 根目录 `infraMap.svg`
- `public/infraMap.svg`
- `src-tauri/icons/*`
- 若使用 `gen_icon.py` 生成图标源（`app-icon.png`），需确认输出路径与目标路径正确。
- 更新 `src-tauri/icons` 后，需核对 `src-tauri/tauri.conf.json` 的 `bundle.icon` 条目仍然可用。
- 图标批量更新应在提交说明中注明来源与生成方式，便于复现。

## 10. 文档维护策略

- 新增条款必须满足：可执行、可检查、可长期维护。
- 规则应以仓库现实为准，避免写入只对单次会话有效的临时上下文。
- 当命令、目录结构、测试入口、脚本参数、打包方式或主题机制发生变化时，必须同步更新 `AGENTS.md` 与 `CLAUDE.md`。
- `AGENTS.md` 适合沉淀协作硬规则，不适合塞入会频繁漂移的实现细节；细节说明应收敛到 `CLAUDE.md`。
- 规范以“质量优先、风险可控、协作高效”为核心，不做形式化膨胀。
