# InfraMap Windows 发布脚本
# 完整发布流程：构建 + 打包 + GitHub Release 上传
#
# 用法:
#   .\release-win.ps1 -Tag "v0.1.0"
#   .\release-win.ps1 -Tag "v0.1.0" -SkipUpload        # 仅构建，不发布
#   .\release-win.ps1 -Tag "v0.1.0" -SkipBuild         # 跳过构建，直接发布已有产物
#   .\release-win.ps1 -Tag "v0.1.0" -Repo "user/repo"  # 指定目标仓库

param(
    [Parameter(Mandatory=$true)]
    [string]$Tag,

    [string]$Repo = "",

    [string]$WebView2Path = "E:\Microsoft.WebView2.FixedVersionRuntime",

    [switch]$SkipBuild,

    [switch]$SkipUpload
)

$ErrorActionPreference = "Stop"

# 规范化标签
$Tag = $Tag -replace '^v', ''
$versionTag = "v$Tag"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "InfraMap Windows 发布脚本" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "版本: $versionTag" -ForegroundColor Yellow

# 检查 gh CLI
if (-not $SkipUpload) {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        Write-Error @"
未找到 GitHub CLI (gh)。
请安装: https://cli.github.com/
然后登录: gh auth login
"@
        exit 1
    }

    # 检查 gh 登录状态
    $ghStatus = gh auth status 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "GitHub CLI 未登录。请运行: gh auth login"
        exit 1
    }

    Write-Host "GitHub CLI: 已登录" -ForegroundColor Gray
}

$projectRoot = Join-Path $PSScriptRoot ".."
$srcTauriDir = Join-Path $projectRoot "src-tauri"
$releaseDir = Join-Path $projectRoot "dist\releases\$versionTag"
$tempDir = Join-Path $env:TEMP "InfraMap-Release-$Tag-$(Get-Random)"

# 清理并创建发布目录
if (Test-Path $releaseDir) {
    Remove-Item $releaseDir -Recurse -Force
}
New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

Write-Host "`n发布目录: $releaseDir" -ForegroundColor Gray

# 检查 WebView2 Runtime
# 优先使用自定义路径，其次检查项目目录下的 WebView2 目录
$webview2SearchPaths = @($WebView2Path, (Join-Path $srcTauriDir "WebView2"))
$webview2Runtime = $null
$webview2BasePath = $null

foreach ($searchPath in $webview2SearchPaths) {
    if (Test-Path $searchPath) {
        $runtime = Get-ChildItem $searchPath -Directory -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -like "Microsoft.WebView2.FixedVersionRuntime.*" } |
            Select-Object -First 1

        if ($runtime) {
            $webview2Runtime = $runtime
            $webview2BasePath = $searchPath
            break
        }
    }
}

if ($webview2Runtime) {
    Write-Host "WebView2 Runtime: $($webview2Runtime.Name)" -ForegroundColor Green
    Write-Host "  路径: $webview2BasePath" -ForegroundColor Gray
    $hasFullVersion = $true
} else {
    Write-Warning "未找到 WebView2 Fixed Runtime。完整版构建将被跳过。"
    Write-Host "预期路径:" -ForegroundColor Gray
    Write-Host "  - $WebView2Path" -ForegroundColor Gray
    Write-Host "  - $(Join-Path $srcTauriDir "WebView2")" -ForegroundColor Gray
    Write-Host "下载地址: https://developer.microsoft.com/en-us/microsoft-edge/webview2/" -ForegroundColor Gray
    $hasFullVersion = $false
}

# ========== 阶段 1: 构建前端 ==========
if (-not $SkipBuild) {
    Write-Host "`n[1/6] 构建前端..." -ForegroundColor Green

    Set-Location $projectRoot
    pnpm build

    if ($LASTEXITCODE -ne 0) {
        Write-Error "前端构建失败"
        exit 1
    }
} else {
    Write-Host "`n[1/6] 跳过前端构建 (-SkipBuild)" -ForegroundColor Gray
}

# ========== 阶段 2: 构建轻量版 NSIS ==========
if (-not $SkipBuild) {
    Write-Host "`n[2/6] 构建轻量版 NSIS 安装包..." -ForegroundColor Green

    Set-Location $srcTauriDir

    # 使用 lite 配置构建
    $env:TAURI_BUNDLE_TYPE = "nsis"
    pnpm tauri build

    if ($LASTEXITCODE -ne 0) {
        Write-Error "轻量版构建失败"
        exit 1
    }

    # 复制产物
    $liteInstaller = Join-Path $srcTauriDir "target\release\bundle\nsis\InfraMap_${Tag}_x64-setup.exe"
    if (Test-Path $liteInstaller) {
        $destName = "InfraMap_${Tag}_x64_setup-lite.exe"
        Copy-Item $liteInstaller (Join-Path $releaseDir $destName)
        Write-Host "  ✓ $destName" -ForegroundColor Gray
    } else {
        # 尝试其他命名格式
        $alternativePaths = @(
            "target\release\bundle\nsis\InfraMap_${Tag}_x64-setup.exe",
            "target\release\bundle\nsis\InfraMap_${Tag}_x64_setup.exe",
            "target\release\bundle\nsis\InfraMap ${Tag}.exe",
            "target\release\bundle\nsis\InfraMap.exe"
        )

        $found = $false
        foreach ($path in $alternativePaths) {
            $fullPath = Join-Path $srcTauriDir $path
            if (Test-Path $fullPath) {
                $destName = "InfraMap_${Tag}_x64_setup-lite.exe"
                Copy-Item $fullPath (Join-Path $releaseDir $destName)
                Write-Host "  ✓ $destName (from $path)" -ForegroundColor Gray
                $found = $true
                break
            }
        }

        if (-not $found) {
            Write-Warning "未找到轻量版安装程序"
        }
    }
} else {
    Write-Host "`n[2/6] 跳过轻量版构建 (-SkipBuild)" -ForegroundColor Gray
}

# ========== 阶段 3: 构建完整版 NSIS ==========
if (-not $SkipBuild -and $hasFullVersion) {
    Write-Host "`n[3/6] 构建完整版 NSIS 安装包..." -ForegroundColor Green

    # 如果 WebView2 不在 src-tauri 目录下，需要临时复制过去
    $projectWebview2Path = Join-Path $srcTauriDir "WebView2"
    $runtimeCopied = $false

    if ($webview2BasePath -ne $projectWebview2Path) {
        Write-Host "  复制 WebView2 Runtime 到项目目录..." -ForegroundColor Gray
        if (-not (Test-Path $projectWebview2Path)) {
            New-Item -ItemType Directory -Path $projectWebview2Path -Force | Out-Null
        }
        $destRuntimePath = Join-Path $projectWebview2Path $webview2Runtime.Name
        if (-not (Test-Path $destRuntimePath)) {
            Copy-Item (Join-Path $webview2BasePath $webview2Runtime.Name) $destRuntimePath -Recurse
            $runtimeCopied = $true
        }
    }

    # 备份原始配置
    $tauriConfPath = Join-Path $srcTauriDir "tauri.conf.json"
    $originalConf = Get-Content $tauriConfPath -Raw
    $conf = $originalConf | ConvertFrom-Json

    # 修改配置使用固定运行时
    $offlineConf = $conf | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    if (-not $offlineConf.bundle.windows) {
        $offlineConf.bundle | Add-Member -NotePropertyName "windows" -NotePropertyValue @{} -Force
    }

    # 添加 WebView2 离线安装配置
    $webviewInstallMode = @{
        type = "fixedRuntime"
        path = "WebView2/$($webview2Runtime.Name)"
    }

    $offlineConf.bundle.windows | Add-Member -NotePropertyName "webviewInstallMode" -NotePropertyValue $webviewInstallMode -Force

    # 保存修改后的配置
    $offlineConf | ConvertTo-Json -Depth 10 | Set-Content $tauriConfPath

    try {
        Set-Location $srcTauriDir
        pnpm tauri build

        if ($LASTEXITCODE -ne 0) {
            Write-Error "完整版构建失败"
            exit 1
        }

        # 复制产物
        $fullInstaller = Join-Path $srcTauriDir "target\release\bundle\nsis\InfraMap_${Tag}_x64-setup.exe"
        if (Test-Path $fullInstaller) {
            $destName = "InfraMap_${Tag}_x64_setup-full.exe"
            Copy-Item $fullInstaller (Join-Path $releaseDir $destName)
            Write-Host "  ✓ $destName" -ForegroundColor Gray
        } else {
            # 尝试查找任何 nsis 安装程序
            $nsisDir = Join-Path $srcTauriDir "target\release\bundle\nsis"
            if (Test-Path $nsisDir) {
                $installer = Get-ChildItem $nsisDir -Filter "*.exe" | Select-Object -First 1
                if ($installer) {
                    $destName = "InfraMap_${Tag}_x64_setup-full.exe"
                    Copy-Item $installer.FullName (Join-Path $releaseDir $destName)
                    Write-Host "  ✓ $destName" -ForegroundColor Gray
                }
            }
        }
    } finally {
        # 恢复原始配置
        Set-Content $tauriConfPath $originalConf -NoNewline

        # 清理临时复制的 WebView2 Runtime
        if ($runtimeCopied -and (Test-Path $projectWebview2Path)) {
            Write-Host "  清理临时 WebView2 Runtime..." -ForegroundColor Gray
            Remove-Item $projectWebview2Path -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
} else {
    if ($SkipBuild) {
        Write-Host "`n[3/6] 跳过完整版构建 (-SkipBuild)" -ForegroundColor Gray
    } else {
        Write-Host "`n[3/6] 跳过完整版构建 (WebView2 Runtime 未找到)" -ForegroundColor Gray
    }
}

# ========== 阶段 4: 构建便携版 ==========
if (-not $SkipBuild) {
    Write-Host "`n[4/6] 构建便携版..." -ForegroundColor Green

    $targetDir = Join-Path $srcTauriDir "target\release"

    # 检查必要的文件
    $requiredFiles = @(
        "InfraMap.exe",
        "inframap_lib.dll"
    )

    $missingFiles = $requiredFiles | Where-Object { -not (Test-Path (Join-Path $targetDir $_)) }
    if ($missingFiles) {
        Write-Warning "缺少必要的构建产物: $($missingFiles -join ', ')"
    } else {
        # 轻量便携版
        $portableLiteDir = Join-Path $tempDir "InfraMap_${Tag}_x64_portable-lite"
        New-Item -ItemType Directory -Path $portableLiteDir -Force | Out-Null

        foreach ($file in $requiredFiles) {
            Copy-Item (Join-Path $targetDir $file) $portableLiteDir
        }

        # 创建启动脚本
        $startScript = @"
@echo off
start "" "%~dp0InfraMap.exe"
"@
        Set-Content (Join-Path $portableLiteDir "启动 InfraMap.bat") $startScript -Encoding UTF8

        $zipPath = Join-Path $releaseDir "InfraMap_${Tag}_x64_portable-lite.zip"
        Compress-Archive -Path "$portableLiteDir\*" -DestinationPath $zipPath -Force
        Write-Host "  ✓ InfraMap_${Tag}_x64_portable-lite.zip" -ForegroundColor Gray

        # 完整便携版（如果有 WebView2）
        if ($hasFullVersion) {
            $portableFullDir = Join-Path $tempDir "InfraMap_${Tag}_x64_portable-full"
            New-Item -ItemType Directory -Path $portableFullDir -Force | Out-Null

            foreach ($file in $requiredFiles) {
                Copy-Item (Join-Path $targetDir $file) $portableFullDir
            }

            # 复制 WebView2 Runtime
            $destWebview2Path = Join-Path $portableFullDir "WebView2"
            New-Item -ItemType Directory -Path $destWebview2Path -Force | Out-Null
            Copy-Item (Join-Path $webview2BasePath $webview2Runtime.Name) $destWebview2Path -Recurse

            # 创建启动脚本（包含 WebView2 路径）
            $startScriptFull = @"
@echo off
set WEBVIEW2_BROWSER_EXECUTABLE_FOLDER=%~dp0WebView2\$($webview2Runtime.Name)
start "" "%~dp0InfraMap.exe"
"@
            Set-Content (Join-Path $portableFullDir "启动 InfraMap.bat") $startScriptFull -Encoding UTF8

            $zipPathFull = Join-Path $releaseDir "InfraMap_${Tag}_x64_portable-full.zip"
            Compress-Archive -Path "$portableFullDir\*" -DestinationPath $zipPathFull -Force
            Write-Host "  ✓ InfraMap_${Tag}_x64_portable-full.zip" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "`n[4/6] 跳过便携版构建 (-SkipBuild)" -ForegroundColor Gray
}

# ========== 阶段 5: 生成 SHA256 校验和 ==========
Write-Host "`n[5/6] 生成 SHA256 校验和..." -ForegroundColor Green

$checksumsFile = Join-Path $releaseDir "SHA256SUMS.txt"
$checksums = @()

Get-ChildItem $releaseDir -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" } | ForEach-Object {
    $hash = Get-FileHash $_.FullName -Algorithm SHA256
    $line = "$($hash.Hash)  $($_.Name)"
    $checksums += $line
    Write-Host "  $line" -ForegroundColor Gray
}

$checksums | Set-Content $checksumsFile
Write-Host "  ✓ SHA256SUMS.txt" -ForegroundColor Gray

# ========== 阶段 6: 创建 GitHub Release ==========
if (-not $SkipUpload) {
    Write-Host "`n[6/6] 创建 GitHub Release..." -ForegroundColor Green

    # 确定仓库
    if (-not $Repo) {
        # 尝试从 git remote 获取
        Set-Location $projectRoot
        $remoteUrl = git remote get-url origin 2>$null
        if ($remoteUrl -match 'github\.com[:/]([^/]+/[^/]+?)(\.git)?$') {
            $Repo = $Matches[1]
        }
    }

    if (-not $Repo) {
        Write-Error "无法确定目标仓库。请使用 -Repo 参数指定，如: -Repo `"user/repo`""
        exit 1
    }

    Write-Host "目标仓库: $Repo" -ForegroundColor Gray

    # 检查标签是否存在，不存在则创建
    $tagExists = git tag -l $versionTag
    if (-not $tagExists) {
        Write-Host "创建标签: $versionTag" -ForegroundColor Yellow
        git tag $versionTag
        git push origin $versionTag
    }

    # 检查 Release 是否已存在
    $existingRelease = gh release view $versionTag -R $Repo 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Release $versionTag 已存在，将上传资源..." -ForegroundColor Yellow

        # 上传资源
        Get-ChildItem $releaseDir -File | ForEach-Object {
            Write-Host "上传: $($_.Name)" -ForegroundColor Gray
            gh release upload $versionTag $_.FullName -R $Repo --clobber
        }
    } else {
        # 创建新 Release
        Write-Host "创建新 Release: $versionTag" -ForegroundColor Yellow

        # 生成 release notes
        $releaseNotes = @"
## InfraMap $versionTag

### 安装包

- **轻量版安装包** (")InfraMap_${Tag}_x64_setup-lite.exe"): 需要系统预装 WebView2 Runtime
- **完整版安装包** (")InfraMap_${Tag}_x64_setup-full.exe"): 集成 WebView2 Runtime，无需额外安装

### 便携版

- **轻量便携版** (")InfraMap_${Tag}_x64_portable-lite.zip"): 解压即用，需系统预装 WebView2
$(if ($hasFullVersion) { "- **完整便携版** (`"InfraMap_${Tag}_x64_portable-full.zip`"): 包含 WebView2 Runtime，解压即用" } else { "" })

### 校验和

查看 [")SHA256SUMS.txt"](SHA256SUMS.txt) 获取所有文件的 SHA256 校验和。
"@

        # 创建 release 并上传
        gh release create $versionTag `
            --repo $Repo `
            --title "InfraMap $versionTag" `
            --notes $releaseNotes `
            (Get-ChildItem $releaseDir -File | ForEach-Object { $_.FullName })
    }

    Write-Host "`n✓ Release 创建成功!" -ForegroundColor Green
    Write-Host "  URL: https://github.com/$Repo/releases/tag/$versionTag" -ForegroundColor Cyan
} else {
    Write-Host "`n[6/6] 跳过上传 (-SkipUpload)" -ForegroundColor Gray
}

# 清理临时目录
Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue

# 输出总结
Write-Host "`n======================================" -ForegroundColor Green
Write-Host "发布完成！" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host "`n产物目录: $releaseDir" -ForegroundColor Cyan
Write-Host "`n文件列表:" -ForegroundColor Yellow
Get-ChildItem $releaseDir -File | ForEach-Object {
    $size = if ($_.Length -gt 1MB) {
        "{0:N1} MB" -f ($_.Length / 1MB)
    } else {
        "{0:N1} KB" -f ($_.Length / 1KB)
    }
    Write-Host "  - $($_.Name) ($size)"
}

if ($SkipUpload) {
    Write-Host "`n上传到 GitHub:" -ForegroundColor Yellow
    Write-Host "  gh release create $versionTag (Get-ChildItem '$releaseDir' -File)" -ForegroundColor Gray
}
