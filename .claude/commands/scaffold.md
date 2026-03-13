# Scaffold 全栈 CRUD 资源

根据输入的资源名称 `$ARGUMENTS`，按照 InfraMap 项目约定生成完整的全栈 CRUD 代码。

## 输入格式

资源名称使用 snake_case 单数形式，例如：`certificate`、`domain`、`config_item`

## 执行步骤

### 准备工作

1. 将 `$ARGUMENTS` 解析为资源名称（单数形式）
2. 推导命名变体：
   - 单数 snake_case: `$ARGUMENTS`（如 `certificate`）
   - 复数 snake_case: 自动加 `s`（如 `certificates`）
   - PascalCase: 首字母大写（如 `Certificate`）
   - 中文标签: 需要你根据语义推断（如 `certificate` → `证书`）
3. 参考 `contacts` 模块作为模板（最新、最规范的模块）

### 前置检查

读取以下文件了解现有代码模式：

- `src-tauri/src/models/contact.rs` - 模型结构参考
- `src-tauri/src/commands/contacts.rs` - 命令处理参考
- `src-tauri/src/validation.rs` - 校验规则参考
- `src-tauri/src/db/schema.rs` - 获取当前最大迁移版本号
- `src-tauri/src/lib.rs` - 了解命令注册位置
- `src-tauri/src/commands/mod.rs` - 模块注册格式
- `src-tauri/src/models/mod.rs` - 模型注册格式
- `src/types/index.ts` - TypeScript 类型定义格式
- `src/api/contacts.ts` - API 封装格式
- `src/api/index.ts` - API 导出格式

### 按顺序生成/修改文件

**在开始之前，先询问用户该资源有哪些字段及其类型，以及中文标签名称。**

#### 1. 后端模型 - `src-tauri/src/models/{单数}.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {PascalCase} {
    #[serde(default)]
    pub id: String,
    // 业务字段...
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
```

约定：

- `id`、`created_at`、`updated_at` 加 `#[serde(default)]`
- 可选字段用 `Option<String>`
- 不包含 `is_deleted`、`deleted_at`（由 DB 层管理）

#### 2. 注册模型模块 - `src-tauri/src/models/mod.rs`

追加 `pub mod {单数};`

#### 3. 数据库迁移 - `src-tauri/src/db/schema.rs`

在迁移数组末尾追加新版本：

- 版本号 = 当前最大版本号 + 1
- 建表包含：id (TEXT PRIMARY KEY), 业务字段, is_deleted (INTEGER NOT NULL DEFAULT 0), deleted_at (TEXT), created_at (TEXT NOT NULL), updated_at (TEXT NOT NULL)
- 创建必要索引（WHERE is_deleted = 0）
- 如需唯一约束，加 WHERE is_deleted = 0

#### 4. 数据验证 - `src-tauri/src/validation.rs`

追加 `validate_{单数}()` 函数：

- 必填字段用 `validate_required()`
- 字符串长度用 `validate_string_length()`
- 可选字段先 `as_deref().map(str::trim).filter(|v| !v.is_empty())` 再校验

#### 5. 命令处理 - `src-tauri/src/commands/{复数}.rs`

实现四个命令函数：

- `list_{复数}(pool, params: QueryParams) -> AppResult<PagedResult<{PascalCase}>>` - 分页列表
- `get_{单数}(pool, id: String) -> AppResult<{PascalCase}>` - 获取单项
- `save_{单数}(pool, data: {PascalCase}) -> AppResult<String>` - 使用 `impl_save_command!` 宏
- `delete_{单数}(pool, id: String) -> AppResult<()>` - 使用 `impl_delete_command!` 宏或自定义实现

参考 `contacts.rs` 中的具体实现模式（SELECT_COLUMNS、build_where_clause、row_to_model 等）。

#### 6. 注册命令模块 - `src-tauri/src/commands/mod.rs`

追加 `pub mod {复数};`

#### 7. 注册到 invoke_handler - `src-tauri/src/lib.rs`

在 `tauri::generate_handler![]` 中追加：

```rust
commands::{复数}::list_{复数},
commands::{复数}::get_{单数},
commands::{复数}::save_{单数},
commands::{复数}::delete_{单数},
```

#### 8. TypeScript 类型 - `src/types/index.ts`

追加接口定义（不包含 is_deleted、deleted_at）：

```typescript
export interface {PascalCase} {
  id: string;
  // 业务字段...
  created_at: string;
  updated_at: string;
}
```

#### 9. API 封装 - `src/api/{复数}.ts`

```typescript
import { tauriInvoke } from "@/utils/invoke";
import type { {PascalCase}, PagedResult, QueryParams } from "@/types";

export function list{PascalCase复数}(params: QueryParams): Promise<PagedResult<{PascalCase}>> {
  return tauriInvoke<PagedResult<{PascalCase}>>("list_{复数}", { params });
}

export function get{PascalCase}(id: string): Promise<{PascalCase}> {
  return tauriInvoke<{PascalCase}>("get_{单数}", { id });
}

export function save{PascalCase}(data: Partial<{PascalCase}>): Promise<string> {
  return tauriInvoke<string>("save_{单数}", { data });
}

export function delete{PascalCase}(id: string): Promise<void> {
  return tauriInvoke<void>("delete_{单数}", { id });
}
```

#### 10. API 导出 - `src/api/index.ts`

追加 `export * from "./{复数}";`

### 完成后检查

生成完毕后，运行以下检查确认代码正确：

1. `cd src-tauri && cargo check 2>&1` - Rust 编译检查
2. `npx vue-tsc --noEmit 2>&1` - TypeScript 类型检查

如果有编译错误，立即修复。

### 后续步骤提示

代码骨架生成完毕后，提醒用户还需要手动完成：

1. 创建视图组件 `src/views/{PascalCase复数}View.vue`
2. 配置路由 `src/router/index.ts`
3. 添加到侧边栏导航菜单
4. （可选）创建卡片组件、编辑对话框等 UI 组件
