# 仓库贡献指南

## 项目结构与模块组织
- `src/`：Vue 3 + TypeScript 前端代码。
- `src/views`、`src/components`、`src/layouts`：页面级与可复用 UI 组件。
- `src/api`：Tauri 命令的类型化封装（每类资源一个文件）。
- `src/composables`、`src/stores`、`src/router`、`src/utils`：复用逻辑、Pinia 状态、路由与调用工具。
- `src/__mocks__/tauri.ts`：Vitest 使用的 Tauri invoke Mock。
- `src-tauri/src`：Tauri Rust 后端（`commands/`、`models/`、`db/`、`validation.rs`、`backup.rs`）。
- `public/`：Vite 提供的静态资源。

## 构建、测试与开发命令
- `pnpm dev`：启动 Vite 开发服务器（默认 `localhost:1420`）。
- `pnpm tauri dev`：以 Tauri 开发模式运行桌面应用。
- `pnpm build`：先用 `vue-tsc` 做类型检查，再执行 Vite 构建。
- `pnpm build:installer`：构建可分发的 Tauri 安装包。
- `pnpm test`：一次性运行前端单元测试（Vitest）。
- `pnpm test:watch`：以监听模式运行 Vitest。
- `pnpm test:backend`：在 `src-tauri` 中运行 Rust 测试（`cargo test`）。

## 代码风格与命名规范
- TypeScript/Vue：使用 Composition API 与 `<script setup lang="ts">`。
- API 与类型命名应与后端命令/模型保持一致。
- 前端文件命名：组件/页面使用 `PascalCase.vue`，组合式函数使用 `useXxx.ts`，测试文件使用同目录 `*.test.ts`。
- Rust：命令与函数使用 `snake_case`，提交前执行 `cargo fmt`。
- 从 `src` 导入时统一使用 `@/` 别名。

## UI 设计规范（参考 LazyCat）
本规范用于统一 InfraMap 的视觉与交互基线。定位为中等约束：新增页面或重构页面应默认遵循，存量页面允许渐进式对齐。

### 设计原则
- 一致性优先：统一 token、间距、圆角、动效与状态反馈，避免“每页一套样式”。
- 信息效率优先：样式服务于可读性和操作效率，不做高噪音装饰。
- 渐进演进：不强制一次性重构全站，但新增/重构必须按规范落地。

### 必须项（Required）
- 统一 Token 体系：全局样式变量使用统一前缀（建议 `--im-*`），至少覆盖颜色、文字、边框、圆角、阴影、动效、字体。
- Element Plus 主题变量必须由项目 token 映射，不允许在页面中大面积硬编码颜色值。
- 主题机制统一使用 `html[data-theme="dark|light"]`；明暗主题都需可用，默认可按产品选择其一。
- 页面背景需有层次（底色 + 轻量渐变/径向光斑），禁止整页单一平色导致界面发闷。
- 交互状态必须完整：可点击元素至少实现 `hover`、`active`、`focus-visible` 三态，且焦点态清晰可见。
- 布局与间距遵循统一节奏：优先使用 `8/12/16/24` 间距体系，避免同屏出现多种随机间距。
- 导航、卡片、输入框、按钮、表格的样式反馈需统一：边框增强、底色变化和阴影变化应遵循同一语言。
- 动效时长仅使用有限档位（建议 `120ms/180ms/300ms`），缓动函数统一（建议 `cubic-bezier(0.4, 0, 0.2, 1)`）。
- 字体分层明确：标题、正文、等宽字体分别配置变量，代码/IP/端口等信息使用等宽字体。
- 响应式必须保证可用：窄屏场景下双栏自动单列，卡片网格可降级，关键交互区域高度不低于 `32px`。
- 新增页面优先复用 `src/styles` 的全局规则与 token 文件，不允许重复造一套平行主题。

### 建议项（Recommended）
- 使用轻量的“品牌强调元素”：如顶部 2px 渐变强调条、active 左侧高亮条、卡片 hover 轻光晕。
- 首页/列表卡片可使用小幅 reveal 动画（透明度 + 轻位移），提升层次感但不影响读写速度。
- 统一文本层级，建议形成稳定字号梯度（如 `26/18/15/13`）并在多个页面复用。
- 长文本与工具名统一省略策略（ellipsis + tooltip/title），避免挤压布局。
- 将常见状态色（success/warning/danger/info）纳入 token，避免同语义多色值并存。

### 禁止项（Disallowed）
- 禁止在模板中长期使用大量内联样式（`style="..."`）直接写颜色、字号、间距。
- 禁止新增页面直接依赖默认 Element Plus 外观而不做主题映射。
- 禁止无规则引入过多动画、强发光、复杂渐变，影响数据阅读和表单操作。
- 禁止在同一页面混用多套圆角、阴影和边框强度，导致视觉噪音。
- 禁止只实现鼠标 hover 而忽略键盘焦点态与可访问性反馈。

### 交付与评审要求
- 涉及 UI 的 PR 必须附截图（至少包含主界面和一个关键交互态）。
- 若因业务原因偏离规范，需在 PR 描述中注明偏离点、原因和影响范围。
- 评审时按本章节逐项检查：token 使用、状态完整性、响应式可用性、视觉一致性。

## 编码与文件保存规范
- 所有源码与文档统一使用 `UTF-8`（建议无 BOM）；禁止使用 GBK/ANSI 等本地编码。
- 修改含中文文案的文件后，必须检查字符串引号是否完整，避免出现 `Unexpected token` 一类编译错误。
- 统一使用 LF 行尾；避免在模板中写入字面量转义文本（如 `` `r`n ``）。
- PowerShell 写文件时显式指定编码：`Set-Content -Encoding utf8`。
- 提交前建议执行：`pnpm build`，并用 `git diff` 快速检查是否出现乱码字符（如 `�`）。

### 乱码防护约束（必做）
- 修改包含中文的 `*.vue`、`*.ts`、`*.md` 文件后，必须执行 `git diff -- <file>` 人工确认中文文案可读，不允许出现“鏇/鍒/缂/璐”等异常字形。
- 禁止使用可能引入系统默认编码的写文件方式（如未指定编码的 `Out-File`、`>` 重定向批量覆盖）；PowerShell 写文件统一使用显式 `-Encoding utf8`。
- 对修复过乱码的文件，保存后必须复查 BOM：`Format-Hex -Path <file> -Count 3`；若出现 `EF BB BF`，需转为 UTF-8 无 BOM 后再提交。
- 涉及中文文案批量调整时，提交前必须执行乱码扫描：`rg -n "�|鏇|鍒|缂|璐|鍝|鎿|绫诲瀷|鍦板潃|鐜" src`，若命中需人工逐条确认并修复。
## 测试规范
- 前端测试框架为 Vitest + jsdom（见 `vitest.config.ts`）。
- 通过 `src/__mocks__/tauri.ts` Mock Tauri invoke，单测中避免真实调用原生 API。
- 修改 `src/api`、`src/composables`、工具模块时，应同步补充或扩展测试。
- 后端逻辑在可行情况下应添加 Rust `#[cfg(test)]` 覆盖。

## 提交与 Pull Request 规范
- 当前历史采用 `type: summary` 风格（示例：`init: ...`），建议继续使用（如 `feat:`、`fix:`、`refactor:`、`test:`）。
- 提交应保持聚焦且原子化，避免将前后端重构与功能变更混在同一提交。
- PR 应包含：
  - 对用户可见影响的简要说明；
  - 关联 issue/任务 ID（如有）；
  - 测试依据（`pnpm test`、`pnpm test:backend`）；
  - 涉及 UI 变更时附截图或录屏。

## 安全与配置说明
- 不要提交本地数据库文件、密钥或环境特定配置。
- 所有写库前的输入应在命令边界（`validation.rs`）完成校验。
- 优先遵循后端既有的软删除与审计日志流程。
- 阶段说明：当前处于快速开发迭代测试阶段，功能开发与验证优先，暂不要求对历史数据/旧版本数据库做兼容处理。

