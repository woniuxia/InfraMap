# InfraMap 协作规范（AGENTS）

## 1. 文档目的与优先级
- 本文件是 InfraMap 仓库级协作契约，适用于人类开发者与代码代理。
- 目标是让协作过程可执行、可检查、可追溯，优先保证质量与安全。
- 规则优先级：安全与数据约束 > 工程与测试基线 > 提交流程 > 风格与体验建议。
- 若外部会话规则与本文件冲突，以本文件与仓库事实为准。

## 2. 仓库事实基线（当前代码状态）
### 2.1 技术栈
- 前端：Vue 3 + TypeScript + Vite + Element Plus + Pinia + Vue Router。
- 桌面壳：Tauri 2（配置见 `src-tauri/tauri.conf.json`）。
- 后端：Rust + rusqlite + r2d2（代码位于 `src-tauri/src`）。
- 测试：Vitest（jsdom）、Playwright（`tests/e2e`）、Rust `cargo test`。

### 2.2 目录结构
- `src/`：前端业务代码。
- `src/views`、`src/components`、`src/layouts`：页面与可复用 UI。
- `src/api`：Tauri 命令类型化封装（按资源分文件）。
- `src/composables`、`src/stores`、`src/router`、`src/utils`、`src/types`：通用逻辑与类型。
- `src/styles`：主题变量、全局样式与反馈样式。
- `src/__mocks__/tauri.ts`：Vitest 的 Tauri invoke mock。
- `src-tauri/src/commands`、`src-tauri/src/models`、`src-tauri/src/db`：Rust 业务层。
- `tests/e2e`：端到端测试用例。
- `public/`：静态资源。

## 3. 开发与验证命令（与脚本保持同步）
- `pnpm dev`：启动前端开发服务器（默认 `localhost:15420`）。
- `pnpm tauri dev`：以 Tauri 开发模式运行桌面应用。
- `pnpm build`：执行 `vue-tsc --noEmit` 后构建前端。
- `pnpm build:installer`：构建 Tauri 安装包。
- `pnpm test`：运行前端单元测试（Vitest run）。
- `pnpm test:watch`：Vitest 监听模式。
- `pnpm test:e2e`：运行 Playwright e2e（`tests/e2e`）。
- `pnpm test:backend`：进入 `src-tauri` 执行 Rust 测试（`cargo test`）。
- 后台执行长测建议设置超时（建议 60s 起），避免任务卡死。

## 4. 编码规范（前后端）
- Vue/TypeScript 统一使用 Composition API 与 `<script setup lang="ts">`。
- `src` 内导入统一使用 `@/` 别名，避免深层相对路径。
- 前端命名：组件/页面 `PascalCase.vue`，组合式函数 `useXxx.ts`，测试 `*.test.ts`。
- API 与类型命名需与后端命令/模型语义一致，避免前后端概念漂移。
- Rust 命令与函数使用 `snake_case`，改动后执行 `cargo fmt`。
- 删除无用代码与失效分支，不保留“临时兼容旧逻辑”作为长期状态。

## 5. UI 与主题规范（强约束）
### 必须
- 统一 token 体系（`--im-*`），覆盖颜色、文字、边框、圆角、阴影、动效、字体。
- Element Plus 主题变量必须由项目 token 映射，不在页面层长期硬编码颜色。
- 主题机制统一使用 `html[data-theme="light|dark"]`。
- 可交互元素必须具备 `hover`、`active`、`focus-visible` 三态。
- 响应式必须可用，窄屏双栏自动单列，关键交互高度不低于 `32px`。

### 建议
- 页面背景保留层次（底色 + 轻量渐变/光斑），避免整页纯色。
- 统一文本层级与溢出策略（ellipsis + tooltip/title）。
- 列表与卡片动效保持轻量（透明度与轻位移），不牺牲可读性。

### 禁止
- 模板中长期使用大量内联样式硬编码颜色、字号、间距。
- 新页面直接依赖 Element Plus 默认外观而不做主题映射。
- 只实现鼠标 hover，忽略键盘焦点态与可访问性反馈。

## 6. 测试策略与验收基线
- 前端单测框架为 Vitest + jsdom，配置见 `vitest.config.ts`。
- 单测中的 Tauri 调用统一通过 `src/__mocks__/tauri.ts`，禁止真实调用原生 API。
- 后端逻辑在可行情况下补充 Rust `#[cfg(test)]` 覆盖。
- e2e 基于 Playwright，测试目录固定为 `tests/e2e`。

### 6.1 提交前验证矩阵（建议项，不作为硬门禁）
- 仅文档改动：建议检查 `git diff` 与 Markdown 可读性。
- 前端页面/组件/API/composable 改动：建议至少执行 `pnpm test`。
- 前端构建链或类型相关改动：建议补充执行 `pnpm build`。
- Rust 后端改动：建议执行 `pnpm test:backend`。
- 路由、关键页面流程、删除确认等交互改动：建议执行 `pnpm test:e2e`。
- 跨前后端改动：建议至少执行一轮前端单测 + 后端测试。

## 7. 提交流程与 commit 规范
### 7.1 提交前改动分析（必须）
- 提交前必须先分析当前改动，至少执行：
- `git status`
- `git diff --stat`
- 关键文件 `git diff` 或 `git diff --cached`
- 确认改动范围与任务目标一致后再提交。

### 7.2 commit message（必须）
- commit message 必须有明确语义，至少说明“改了什么 + 为什么改”。
- 建议沿用 `type(scope): summary`，例如：
- `feat(topology): 新增统一控制栏以收敛筛选与导出操作`
- `fix(api): 修复部署上下文参数在编辑态下未传递的问题`
- 禁止使用无信息说明：`update`、`misc`、`tmp`、`test commit` 等。
- 单次提交保持聚焦与原子化，避免将无关改动混入同一提交。

## 8. PR 与评审要求
- PR 说明必须包含：用户可见影响、关联任务/Issue（如有）、验证依据。
- 涉及 UI 的变更必须附截图或录屏，至少覆盖一个关键交互态。
- 涉及数据写入、删除、导入流程改动时，必须在 PR 中写清风险点与回退方案。
- 较大改动建议先进行结构化代码审查，再进入合并流程。

## 9. 多 CLI / 多 Agent 并行协作规范
- 优先拆分为可独立、无写冲突的子任务并行执行。
- 同一文件多处修改时，先按不重叠区域拆分；无法拆分则串行。
- 遇到强依赖链（A -> B -> C）按顺序执行，不强行并行化。
- 发现他人改动默认视为正常协作状态，不得擅自回退、覆盖、丢弃。
- 每轮并行完成后先汇总结果，再进入下一轮拆解。

## 10. 危险操作确认机制
### 10.1 高风险操作清单
- 文件系统：删除文件/目录、批量覆盖、移动关键文件。
- 系统配置：环境变量、系统权限、系统设置改动。
- 数据操作：数据库删除、结构变更、批量更新。
- 网络请求：向生产环境发送敏感数据或执行写操作。
- 包管理：全局安装/卸载、升级核心依赖。

### 10.2 确认模板
在执行高风险操作前，必须先给出以下确认信息：

`⚠️ 危险操作检测！`

`操作类型：` [具体操作]

`影响范围：` [详细说明]

`风险评估：` [潜在后果]

`请确认是否继续？` [是/确认/继续]

## 11. 中文与编码安全
- 所有源码与文档统一使用 UTF-8（无 BOM）+ LF 行尾。
- 禁止使用 GBK/ANSI 等本地编码写入仓库文件。
- 修改含中文文本文件后，必须人工确认中文可读，避免出现乱码字形。
- PowerShell 覆盖写文件必须显式指定编码，避免默认编码污染。
- 提交前建议执行乱码扫描：
- `rg -n "�|鏇|鍒|缂|璐|鍝|鎿|绫诲瀷|鍦板潃|鐜" src AGENTS.md`
- 若文件曾修复乱码，建议检查 BOM：
- `Format-Hex -Path <file> -Count 3`
- 若出现 `EF BB BF`，需移除 BOM 后再提交。

## 12. 图标与静态资源变更规范
- 图标源文件与目标文件必须保持一致性，避免只改单端资源：
- 根目录 `infraMap.svg`
- `public/infraMap.svg`
- `src-tauri/icons/*`
- 若使用 `gen_icon.py` 生成图标源（`app-icon.png`），需确认输出文件与目标路径正确。
- 更新 `src-tauri/icons` 后，需核对 `src-tauri/tauri.conf.json` 的 `bundle.icon` 条目仍可用。
- 图标批量更新应在提交说明中注明来源与生成方式，便于复现。

## 13. 沟通与终端输出风格
- 默认使用简体中文，结论先行，随后给关键证据与操作细节。
- 输出优先短句和分组标题，避免超长段落堆叠。
- 调试、评审、定位问题时使用 `file:line` 提供可追溯定位。
- 复杂任务必须明确：当前阶段结果、待办项、下一步动作。
- 报告失败时必须说明失败原因、已尝试路径与阻塞点。

## 14. 文档维护策略
- 新增条款必须满足：可执行、可检查、可长期维护。
- 规则应以仓库现实为准，避免写入会话级临时上下文。
- 当脚本、目录结构、测试入口变更时，应同步更新本文件。
- 规范以“质量优先、风险可控、协作高效”为核心，不做形式化膨胀。
