# InfraMap 版本同步脚本
# 用法: .\sync-version.ps1 <版本号>
# 示例: .\sync-version.ps1 0.2.0

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

# 移除版本号前缀的 'v'（如果有）
$Version = $Version -replace '^v', ''

# 验证版本号格式
if ($Version -notmatch '^\d+\.\d+\.\d+(-\w+)?$') {
    Write-Error "无效的版本号格式: $Version`n期望格式: x.y.z 或 x.y.z-label"
    exit 1
}

Write-Host "同步版本号到: $Version" -ForegroundColor Green

$projectRoot = Join-Path $PSScriptRoot ".."
$filesUpdated = @()

# 1. 更新 package.json
$packageJsonPath = Join-Path $projectRoot "package.json"
if (Test-Path $packageJsonPath) {
    $content = Get-Content $packageJsonPath -Raw
    $content = $content -replace '"version":\s*"[^"]+"', "`"version`": `"$Version`""
    Set-Content $packageJsonPath $content -NoNewline
    $filesUpdated += "package.json"
    Write-Host "  ✓ package.json" -ForegroundColor Gray
}

# 2. 更新 Cargo.toml
$cargoTomlPath = Join-Path $projectRoot "src-tauri\Cargo.toml"
if (Test-Path $cargoTomlPath) {
    $content = Get-Content $cargoTomlPath -Raw
    $content = $content -replace '^version = "[^"]+"', "version = `"$Version`""
    Set-Content $cargoTomlPath $content -NoNewline
    $filesUpdated += "src-tauri/Cargo.toml"
    Write-Host "  ✓ src-tauri/Cargo.toml" -ForegroundColor Gray
}

# 3. 更新 tauri.conf.json
$tauriConfPath = Join-Path $projectRoot "src-tauri\tauri.conf.json"
if (Test-Path $tauriConfPath) {
    $content = Get-Content $tauriConfPath -Raw
    $content = $content -replace '"version":\s*"[^"]+"', "`"version`": `"$Version`""
    Set-Content $tauriConfPath $content -NoNewline
    $filesUpdated += "src-tauri/tauri.conf.json"
    Write-Host "  ✓ src-tauri/tauri.conf.json" -ForegroundColor Gray
}

Write-Host "`n版本同步完成！以下文件已更新:" -ForegroundColor Green
$filesUpdated | ForEach-Object { Write-Host "  - $_" }

Write-Host "`n下一步:" -ForegroundColor Yellow
Write-Host "  1. 检查更改: git diff"
Write-Host "  2. 提交更改: git add . && git commit -m \"chore: bump version to $Version\""
Write-Host "  3. 创建标签: git tag v$Version"
