# InfraMap Windows 构建脚本
# 在 Visual Studio 2022 开发者环境中执行 Tauri 构建

param(
    [switch]$Release = $true
)

$ErrorActionPreference = "Stop"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "InfraMap Windows 构建脚本" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan

# 检查 cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "未找到 cargo。请安装 Rust: https://rustup.rs/"
    exit 1
}

$cargoVersion = cargo --version
Write-Host "Rust/Cargo: $cargoVersion" -ForegroundColor Gray

# 检查 perl（OpenSSL 构建需要）
$perlPath = Get-Command perl -ErrorAction SilentlyContinue
if (-not $perlPath) {
    # 尝试查找 Strawberry Perl
    $strawberryPerl = "C:\Strawberry\perl\bin\perl.exe"
    if (Test-Path $strawberryPerl) {
        $env:PATH = "C:\Strawberry\perl\bin;$env:PATH"
        Write-Host "Perl: 使用 Strawberry Perl" -ForegroundColor Gray
    } else {
        Write-Warning "未找到 Perl。某些依赖可能无法构建。"
    }
} else {
    Write-Host "Perl: $($perlPath.Source)" -ForegroundColor Gray
}

# 查找 VsDevCmd.bat
function Find-VsDevCmd {
    $vsPaths = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional\Common7\Tools\VsDevCmd.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise\Common7\Tools\VsDevCmd.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"
    )

    foreach ($path in $vsPaths) {
        if (Test-Path $path) {
            return $path
        }
    }

    # 尝试使用 vswhere
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($installationPath) {
            $devCmdPath = Join-Path $installationPath "Common7\Tools\VsDevCmd.bat"
            if (Test-Path $devCmdPath) {
                return $devCmdPath
            }
        }
    }

    return $null
}

$vsDevCmd = Find-VsDevCmd
if (-not $vsDevCmd) {
    Write-Error @"
未找到 Visual Studio 2022 开发者命令提示工具 (VsDevCmd.bat)。
请确保已安装以下组件:
  - Visual Studio 2022 Community/Professional/Enterprise 或 Build Tools
  - "Desktop development with C++" 工作负载

下载地址: https://visualstudio.microsoft.com/downloads/
"@
    exit 1
}

Write-Host "VS DevCmd: $vsDevCmd" -ForegroundColor Gray

# 剥离 Git 的 usr/bin 目录以避免 link.exe 冲突
$originalPath = $env:PATH
$cleanPath = ($env:PATH -split ';' | Where-Object { $_ -notmatch 'Git\usr\bin' }) -join ';'
$env:PATH = $cleanPath

# 检查 Windows SDK
$kernel32Paths = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\Lib\*\um\x64\kernel32.lib",
    "${env:ProgramFiles}\Windows Kits\10\Lib\*\um\x64\kernel32.lib"
)
$hasKernel32 = $kernel32Paths | ForEach-Object { Test-Path $_ } | Where-Object { $_ } | Select-Object -First 1

if (-not $hasKernel32) {
    Write-Error @"
未找到 Windows SDK (kernel32.lib)。
请确保已安装 Windows 10/11 SDK。
"@
    exit 1
}

Write-Host "Windows SDK: 已安装" -ForegroundColor Gray

# 设置构建参数
$buildMode = if ($Release) { "release" } else { "debug" }
Write-Host "构建模式: $buildMode" -ForegroundColor Yellow

$projectRoot = Join-Path $PSScriptRoot ".."
$srcTauriDir = Join-Path $projectRoot "src-tauri"

Write-Host "`n开始构建..." -ForegroundColor Green

# 在 VS 开发者环境中执行构建
$buildScript = @"
cd /d "$srcTauriDir"
cargo build $(if ($Release) { "--release" })
"@

$tempBat = [System.IO.Path]::GetTempFileName() + ".bat"
$buildScript | Set-Content $tempBat -Encoding ASCII

try {
    $process = Start-Process -FilePath "cmd.exe" -ArgumentList "/c `"`"$vsDevCmd`" -arch=x64 && `"$tempBat`"`"" -Wait -PassThru -NoNewWindow

    if ($process.ExitCode -ne 0) {
        Write-Error "构建失败，退出代码: $($process.ExitCode)"
        exit $process.ExitCode
    }
} finally {
    Remove-Item $tempBat -ErrorAction SilentlyContinue
    $env:PATH = $originalPath
}

Write-Host "`n======================================" -ForegroundColor Green
Write-Host "构建成功！" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green

$targetDir = Join-Path $srcTauriDir "target\release"
Write-Host "`n输出文件:" -ForegroundColor Cyan
Get-ChildItem $targetDir -File | Where-Object {
    $_.Extension -in @('.exe', '.dll') -and $_.Name -notlike '*.*.*'
} | ForEach-Object {
    Write-Host "  - $($_.Name)" -ForegroundColor Gray
}
