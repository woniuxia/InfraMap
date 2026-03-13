# 生成数据库迁移

根据描述 `$ARGUMENTS` 生成标准化的数据库迁移 SQL。

## 执行步骤

### 1. 解析迁移描述

将 `$ARGUMENTS` 作为迁移描述，例如：

- `add certificates table` - 新增表
- `add status column to hosts` - 添加列
- `create index on applications name` - 创建索引
- `add unique constraint on middlewares name env` - 添加唯一约束

### 2. 获取当前版本号

读取 `src-tauri/src/db/schema.rs`，找到迁移数组中的最大版本号。新迁移版本号 = 最大版本号 + 1。

### 3. 生成迁移 SQL

在 `src-tauri/src/db/schema.rs` 的迁移数组末尾追加新条目。

#### 建表迁移模板

```sql
CREATE TABLE IF NOT EXISTS {table_name} (
    id TEXT PRIMARY KEY,
    -- 业务字段 --
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_{table_name}_{column} ON {table_name}({column}) WHERE is_deleted = 0;
```

#### 添加列迁移模板

```sql
ALTER TABLE {table_name} ADD COLUMN {column_name} {TYPE} {DEFAULT};
```

#### 项目约定

- 所有表必须包含：`id TEXT PRIMARY KEY`、`is_deleted INTEGER NOT NULL DEFAULT 0`、`deleted_at TEXT`、`created_at TEXT NOT NULL`、`updated_at TEXT NOT NULL`
- 唯一约束需加 `WHERE is_deleted = 0` 条件
- 索引需加 `WHERE is_deleted = 0` 条件
- 使用 `CREATE TABLE IF NOT EXISTS` 和 `CREATE INDEX IF NOT EXISTS`
- 迁移 SQL 包裹在 `r#"..."#` 原始字符串中
- 迁移注释使用 SQL 注释格式 `-- 描述`

### 4. 完成后提示

生成完毕后，提示用户：

1. 如果是新表，还需要创建对应的模型、命令、验证等代码（可使用 `/scaffold` 命令）
2. 运行 `cd src-tauri && cargo check 2>&1` 验证编译
3. 应用迁移将在下次启动时自动执行
