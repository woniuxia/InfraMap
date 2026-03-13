# 全栈质量检查

依次执行所有质量检查，遇到错误立即报告并尝试修复。

## 检查步骤

按以下顺序执行检查，每一步都要报告结果：

### 1. Rust 编译检查

```bash
cd src-tauri && cargo check 2>&1
```

如果失败，分析错误原因并修复 Rust 代码。

### 2. TypeScript 类型检查

```bash
npx vue-tsc --noEmit 2>&1
```

如果失败，分析类型错误并修复 TypeScript/Vue 代码。

### 3. ESLint 检查

```bash
pnpm lint 2>&1
```

如果有可自动修复的问题，运行 `pnpm lint --fix`。其余错误手动修复。

### 4. 前端单元测试

```bash
pnpm test 2>&1
```

如果测试失败，分析失败原因，修复测试或相关代码。

### 5. 后端单元测试

```bash
cd src-tauri && cargo test 2>&1
```

如果测试失败，分析失败原因，修复测试或相关代码。

## 结果汇总

所有检查完成后，输出汇总表格：

| 检查项          | 结果      | 说明               |
| --------------- | --------- | ------------------ |
| Rust 编译       | PASS/FAIL | 错误数量或 OK      |
| TypeScript 类型 | PASS/FAIL | 错误数量或 OK      |
| ESLint          | PASS/FAIL | 警告/错误数量或 OK |
| 前端测试        | PASS/FAIL | 通过/失败数量      |
| 后端测试        | PASS/FAIL | 通过/失败数量      |

如果有任何失败项且无法自动修复，给出具体的修复建议。
