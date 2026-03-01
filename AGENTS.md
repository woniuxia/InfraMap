# InfraMap 协作规范（AGENTS）

## 1. 文档目的与适用范围
- 本文件是 InfraMap 仓库级协作契约，适用于人类开发者与代码代理。
- 目标是保证交付质量、协作效率和规则一致性，新增与重构默认遵循，存量改动可渐进对齐。
- 规则优先级：安全与数据约束 > 编码与测试基线 > 风格与体验建议。

## 2. 项目结构速览
- `src/`：Vue 3 + TypeScript 前端代码。
- `src/views`、`src/components`、`src/layouts`：页面与可复用 UI 组件。
- `src/api`：Tauri 命令类型化封装（按资源分文件）。
- `src/composables`、`src/stores`、`src/router`、`src/utils`：复用逻辑、状态、路由与工具。
- `src/__mocks__/tauri.ts`：Vitest 的 Tauri invoke mock。
- `src-tauri/src`：Rust 后端（`commands/`、`models/`、`db/`、`validation.rs`、`backup.rs` 等）。
- `public/`：Vite 静态资源。

## 3. 开发命令（最小必备）
- `pnpm dev`：启动前端开发服务器（默认 `localhost:1420`）。
- `pnpm tauri dev`：以 Tauri 开发模式运行桌面应用。
- `pnpm build`：执行 `vue-tsc` 类型检查并构建前端。
- `pnpm build:installer`：构建 Tauri 安装包。
- `pnpm test`：运行前端单元测试（Vitest）。
- `pnpm test:watch`：以监听模式运行 Vitest。
- `pnpm test:backend`：在 `src-tauri` 运行 Rust 测试（`cargo test`）。

## 4. 代码与命名规范（前后端）
- Vue/TypeScript 统一使用 Composition API 与 `<script setup lang="ts">`。
- API 与类型命名需与后端命令/模型语义保持一致。
- 前端命名：组件/页面 `PascalCase.vue`，组合式函数 `useXxx.ts`，测试文件 `*.test.ts`。
- Rust 命令与函数使用 `snake_case`，提交前执行 `cargo fmt`。
- `src` 内导入统一使用 `@/` 别名。

## 5. UI 规范（精简强制项）
### 必须
- 统一 token 体系（建议前缀 `--im-*`），覆盖颜色、文字、边框、圆角、阴影、动效、字体。
- Element Plus 主题变量由项目 token 映射，不在页面大面积硬编码颜色值。
- 主题机制统一使用 `html[data-theme="dark|light"]`，明暗主题均需可用。
- 可点击元素必须具备 `hover`、`active`、`focus-visible` 三态，焦点态清晰可见。
- 响应式必须可用：窄屏双栏自动单列，关键交互高度不低于 `32px`。

### 建议
- 页面背景提供层次（底色 + 轻量渐变或径向光斑），避免整页单色。
- 使用轻量品牌强调元素（如顶部 2px 渐变条、active 左侧高亮条）。
- 列表/卡片使用小幅 reveal 动画（透明度 + 轻位移），不影响可读性。
- 统一文本层级与长文本省略策略（ellipsis + tooltip/title）。

### 禁止
- 在模板中长期使用大量内联样式硬编码颜色、字号与间距。
- 新页面直接依赖默认 Element Plus 外观而不做主题映射。
- 只实现鼠标 hover，忽略键盘焦点态与可访问性反馈。
- 无规则堆叠强发光、复杂渐变和过量动画。

## 6. 编码与中文文本安全
- 所有源码与文档统一使用 UTF-8（无 BOM）和 LF 行尾。
- 禁止使用 GBK/ANSI 等本地编码；PowerShell 写文件请显式使用 `-Encoding utf8`。
- 修改含中文文本文件后，必须人工确认中文可读，避免出现“鏇/鍒/缂/璐”等异常字形。
- 避免使用可能引入系统默认编码的覆盖写入方式（如未指定编码的 `Out-File` 或批量重定向覆盖）。
- 提交前建议执行乱码扫描：`rg -n "�|鏇|鍒|缂|璐|鍝|鎿|绫诲瀷|鍦板潃|鐜" src`。
- 若文件曾修复乱码，保存后检查 BOM：`Format-Hex -Path <file> -Count 3`，若出现 `EF BB BF` 必须移除。

## 7. 测试与验收基线
- 前端测试框架为 Vitest + jsdom（见 `vitest.config.ts`）。
- 单测通过 `src/__mocks__/tauri.ts` mock invoke，避免真实调用原生 API。
- 修改 `src/api`、`src/composables`、`src/utils` 时，应同步补充或扩展测试。
- 后端逻辑在可行情况下补充 Rust `#[cfg(test)]` 覆盖。

## 8. 提交与 PR 要求
- 提交信息建议沿用 `type: summary` 风格（如 `feat:`、`fix:`、`refactor:`、`test:`）。
- 单次提交保持聚焦与原子化，避免将无关改动混入同一提交。
- PR 需包含：用户可见影响说明、关联任务/Issue（如有）、测试依据（`pnpm test`/`pnpm test:backend`）。
- 涉及 UI 变更时，必须附截图或录屏（至少主界面和一个关键交互态）。

## 9. 安全与数据约束
- 禁止提交本地数据库文件、密钥或环境特定配置。
- 所有写库前输入需在命令边界（`validation.rs`）完成校验。
- 优先遵循既有软删除与审计日志流程。
- 当前阶段可优先功能开发与架构合理性，不要求兼容历史数据库版本。

## 10. 多 CLI 并行协作
- 允许多个 CLI 并行修改，发现非本人改动视为正常协作状态。
- 不得擅自回退、覆盖或丢弃他人改动。
- 在不破坏现有改动前提下，直接基于当前工作区继续推进自身任务。

## 11. 技能使用规则（触发原则 + 常用技能）
### 触发原则
- 开始任务先判断是否存在匹配技能；命中则优先按技能流程执行。
- 多技能同时适用时，先过程技能（如需求澄清、调试、验证），再实现技能。
- 技能规则与仓库安全或测试基线冲突时，以安全与测试基线为最高优先级。
- 仅固化长期有效流程，不固化会话级环境细节或临时工具清单。

### 常用技能与场景
- `brainstorming`：需求不清、方案分歧或需要先明确目标边界时使用。
- `writing-plans`：多步骤任务开始前，生成可执行实施计划。
- `test-driven-development`：功能开发与缺陷修复时优先测试先行。
- `systematic-debugging`：出现故障、测试失败或行为异常时先定位根因。
- `verification-before-completion`：声明完成前必须执行并核对验证命令。
- `requesting-code-review`：较大改动或合并前进行结构化审查。

## 12. 变更策略与优先级
- 当前采用轻量协作策略：规则精简、可执行、可验证，避免文档膨胀。
- 新增条款必须满足三点：可执行、可检查、可长期维护。
- 本文件优先描述仓库稳定约束，不写会话级临时上下文与工具运行时信息。

