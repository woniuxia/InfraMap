# InfraMap 构建发布脚本

本目录包含 InfraMap 桌面应用的自动化构建和发布脚本。

## 前置要求

### 必需

- [Rust](https://rustup.rs/) - 后端构建
- [Node.js + pnpm](https://pnpm.io/) - 前端构建
- [Visual Studio 2022](https://visualstudio.microsoft.com/downloads/) + "Desktop development with C++" 工作负载
- [GitHub CLI](https://cli.github.com/) - 发布到 GitHub Releases（仅发布时需要）

### 可选

- [Strawberry Perl](https://strawberryperl.com/) - 某些依赖构建需要
- WebView2 Fixed Version Runtime - 完整版打包需要

## 脚本说明

### sync-version.ps1

同步版本号到所有配置文件。

```powershell
# 同步版本号到 0.2.0
.\sync-version.ps1 0.2.0

# 或使用 pnpm
pnpm sync-version 0.2.0
```

**影响的文件：**

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

---

### build-tauri-win.ps1

在 Visual Studio 2022 开发者环境中执行 Tauri 构建。

```powershell
# 标准构建（Release 模式）
.\build-tauri-win.ps1

# 或使用 pnpm
pnpm build:win
```

**功能特点：**

- 自动查找 Visual Studio 2022（支持 Community/Professional/Enterprise/BuildTools）
- 自动剥离 Git 的 usr/bin 目录避免 link.exe 冲突
- 优先使用 Strawberry Perl
- 自动检测 Windows SDK

---

### release-win.ps1

完整发布流程：构建 + 打包 + GitHub Release 上传。

```powershell
# 完整发布流程
.\release-win.ps1 -Tag "v0.1.0"

# 或使用 pnpm
pnpm release:win -Tag "v0.1.0"

# 仅构建，不上传到 GitHub
.\release-win.ps1 -Tag "v0.1.0" -SkipUpload

# 跳过构建，直接发布已有产物
.\release-win.ps1 -Tag "v0.1.0" -SkipBuild

# 指定目标仓库（fork 场景）
.\release-win.ps1 -Tag "v0.1.0" -Repo "username/InfraMap"

# 指定自定义 WebView2 路径（默认使用 E:\Microsoft.WebView2.FixedVersionRuntime）
.\release-win.ps1 -Tag "v0.1.0" -WebView2Path "D:\WebView2Runtime"
```

**产物清单：**

| 文件                                       | 说明                                  |
| ------------------------------------------ | ------------------------------------- |
| `InfraMap_{version}_x64_setup-lite.exe`    | 轻量版安装包（需系统预装 WebView2）   |
| `InfraMap_{version}_x64_setup-full.exe`    | 完整版安装包（集成 WebView2 Runtime） |
| `InfraMap_{version}_x64_portable-lite.zip` | 轻量便携版                            |
| `InfraMap_{version}_x64_portable-full.zip` | 完整便携版（如有 WebView2）           |
| `SHA256SUMS.txt`                           | 校验和文件                            |

**产物目录：** `dist/releases/{Tag}/`

---

## WebView2 Runtime 配置（完整版需要）

完整版安装包需要嵌入 WebView2 Fixed Version Runtime。

### 默认路径

脚本默认查找以下位置的 WebView2 Runtime：

1. `E:\Microsoft.WebView2.FixedVersionRuntime\` （优先级最高）
2. `src-tauri\WebView2\` （项目目录下）

### 下载

1. 访问 [Microsoft Edge WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
2. 下载 "Fixed Version Runtime"
3. 选择合适的架构（x64）

### 安装

**方式 1：使用默认路径（推荐）**

将下载的压缩包解压到：

```
E:\Microsoft.WebView2.FixedVersionRuntime\Microsoft.WebView2.FixedVersionRuntime.{版本号}\
```

**方式 2：放在项目目录下**

将下载的压缩包解压到：

```
src-tauri/WebView2/Microsoft.WebView2.FixedVersionRuntime.{版本号}/
```

**方式 3：使用自定义路径**

运行发布脚本时指定路径：

```powershell
.\release-win.ps1 -Tag "v0.1.0" -WebView2Path "D:\Custom\WebView2\Path"
```

目录结构示例：

```
E:\Microsoft.WebView2.FixedVersionRuntime\
  Microsoft.WebView2.FixedVersionRuntime.130.0.2849.46\
    msedgewebview2.exe
    ...
```

---

## 完整发布流程示例

```powershell
# 1. 更新版本号
pnpm sync-version 0.2.0

# 2. 检查更改
git diff

# 3. 提交并创建标签
git add .
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0

# 4. 推送到远程（如有需要）
git push origin main
git push origin v0.2.0

# 5. 构建并发布（确保已登录 gh CLI）
gh auth login
pnpm release:win -Tag "v0.2.0"

# 或者在 GitHub Actions 中自动发布
```

---

## 故障排除

### "未找到 VsDevCmd.bat"

安装 Visual Studio 2022 并确保包含 "Desktop development with C++" 工作负载。

### "link.exe" 冲突

脚本会自动处理 Git 的 usr/bin/link.exe 冲突。如果仍有问题，临时从 PATH 中移除 Git\usr\bin。

### WebView2 完整版构建失败

- 检查是否正确放置 WebView2 Runtime
- 确认目录名称以 `Microsoft.WebView2.FixedVersionRuntime.` 开头

### GitHub 发布失败

- 检查 `gh auth status` 确保已登录
- 确认有仓库的写入权限
- 检查标签是否存在：`git tag -l v0.x.x`

---

## package.json 脚本

```json
{
  "scripts": {
    "sync-version": "powershell -ExecutionPolicy Bypass -File ./scripts/sync-version.ps1",
    "build:win": "powershell -ExecutionPolicy Bypass -File ./scripts/build-tauri-win.ps1",
    "release:win": "powershell -ExecutionPolicy Bypass -File ./scripts/release-win.ps1"
  }
}
```
