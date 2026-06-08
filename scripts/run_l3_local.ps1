# scripts/run_l3_local.ps1
#
# L3 运行测试本地化脚本（Windows PowerShell）
#
# 功能：
#   1. 遍历 examples\ 下所有 NNN_*\cpp\ 目录
#   2. 使用 cl.exe (MSVC) 或 g++/clang++ 编译 DLL
#   3. 将 cpp 目录加入 PATH（Windows 动态库搜索路径）
#   4. 运行 L3 集成测试
#
# 用法：
#   .\scripts\run_l3_local.ps1
#   .\scripts\run_l3_local.ps1 -Filter "001"
#   .\scripts\run_l3_local.ps1 -CompileOnly
#   .\scripts\run_l3_local.ps1 -Clean
#
# 前提条件：
#   - MSVC：Visual Studio（含 C++ 桌面开发工作负载）已安装，或通过 vcvarsall.bat 初始化环境
#   - 或 MinGW-w64：g++ 已在 PATH 中（例如通过 MSYS2 安装）
#   - Rust 工具链（cargo）

param(
    [string]$Filter = "",
    [switch]$CompileOnly,
    [switch]$Clean,
    [int]$Threads = 4
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ─────────────────────────────────────────────────────────────────────────────
#  定位仓库根目录
# ─────────────────────────────────────────────────────────────────────────────
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir
$ExamplesDir = Join-Path $RepoRoot "examples"

if (-not (Test-Path $ExamplesDir)) {
    Write-Error "未找到 examples 目录：$ExamplesDir"
    exit 1
}

# ─────────────────────────────────────────────────────────────────────────────
#  检测编译器
# ─────────────────────────────────────────────────────────────────────────────
$CXX = $null
$UseMinGW = $false

# 优先使用 g++（MinGW / MSYS2）
if (Get-Command "g++" -ErrorAction SilentlyContinue) {
    $CXX = "g++"
    $UseMinGW = $true
}
# 其次使用 clang++
elseif (Get-Command "clang++" -ErrorAction SilentlyContinue) {
    $CXX = "clang++"
    $UseMinGW = $true
}
# 最后尝试 cl.exe (MSVC)
elseif (Get-Command "cl" -ErrorAction SilentlyContinue) {
    $CXX = "cl"
    $UseMinGW = $false
}
else {
    Write-Error @"
错误：未找到 C++ 编译器。
  选项 1（MinGW-w64 / MSYS2）：在 MSYS2 UCRT64 终端中运行：
      pacman -S mingw-w64-ucrt-x86_64-gcc
  选项 2（MSVC）：安装 Visual Studio 并从"开发人员命令提示符"运行此脚本：
      "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
"@
    exit 1
}

Write-Host "编译器：$CXX"

# ─────────────────────────────────────────────────────────────────────────────
#  --Clean 模式
# ─────────────────────────────────────────────────────────────────────────────
if ($Clean) {
    Write-Host "清理所有编译产物..."
    Get-ChildItem -Path $ExamplesDir -Recurse -Include "*.dll", "*.lib", "*.exp" |
        ForEach-Object {
            Write-Host "  删除：$($_.FullName)"
            Remove-Item $_.FullName -Force
        }
    Write-Host "清理完成。"
    exit 0
}

# ─────────────────────────────────────────────────────────────────────────────
#  编译阶段
# ─────────────────────────────────────────────────────────────────────────────
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host " L3 运行测试本地化（Windows）"
Write-Host " 编译器：$CXX"
if ($Filter) {
    Write-Host " 过滤器：$Filter"
}
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""

$LibDirs = [System.Collections.Generic.List[string]]::new()
$Failed  = [System.Collections.Generic.List[string]]::new()
$Compiled = 0

foreach ($ExampleDir in Get-ChildItem -Path $ExamplesDir -Directory) {
    $ExampleName = $ExampleDir.Name

    # 应用过滤器
    if ($Filter -and $ExampleName -notlike "*$Filter*") {
        continue
    }

    $CppDir = Join-Path $ExampleDir.FullName "cpp"
    if (-not (Test-Path $CppDir)) {
        continue
    }

    # 从目录名提取库名（去掉 NNN_ 数字前缀）
    $LibName = $ExampleName -replace "^\d+_", ""
    $DllFile = "${LibName}.dll"
    $DllOutput = Join-Path $CppDir $DllFile

    $CppFiles = @(Get-ChildItem -Path $CppDir -Filter "*.cpp" -File |
        Where-Object { $_.Name -notlike "*.test.cpp" } |
        Select-Object -ExpandProperty FullName)

    if ($CppFiles.Count -eq 0) {
        Write-Host "  [跳过] ${ExampleName}：cpp 目录中无 .cpp 文件"
        continue
    }

    Write-Host -NoNewline "  [编译] $ExampleName -> $DllFile ..."

    $Compiled_ok = $false
    try {
        if ($UseMinGW) {
            # MinGW / clang++：生成 DLL
            $Args = @("-std=c++17", "-shared", "-fPIC") + $CppFiles +
                    @("-o", $DllOutput, "-I$CppDir")
            $proc = Start-Process $CXX -ArgumentList $Args -NoNewWindow -Wait -PassThru -RedirectStandardError NUL
            $Compiled_ok = ($proc.ExitCode -eq 0)
        }
        else {
            # MSVC cl.exe：生成 DLL
            $Args = @("/std:c++17", "/LD") + $CppFiles +
                    @("/I$CppDir", "/Fe:$DllOutput")
            $proc = Start-Process $CXX -ArgumentList $Args -NoNewWindow -Wait -PassThru -RedirectStandardError NUL
            $Compiled_ok = ($proc.ExitCode -eq 0)
        }
    }
    catch {
        $Compiled_ok = $false
    }

    if ($Compiled_ok) {
        Write-Host " ✓"
        $LibDirs.Add($CppDir)
        $Compiled++
    }
    else {
        Write-Host " ✗"
        $Failed.Add($ExampleName)
    }
}

Write-Host ""
Write-Host "编译结果：成功 $Compiled 个，失败 $($Failed.Count) 个"

if ($Failed.Count -gt 0) {
    Write-Host "编译失败的示例（可能存在编译依赖问题，仍继续测试）："
    $Failed | ForEach-Object { Write-Host "  - $_" }
}

if ($CompileOnly) {
    Write-Host ""
    Write-Host "（-CompileOnly 模式，跳过测试）"
    exit 0
}

# ─────────────────────────────────────────────────────────────────────────────
#  设置 PATH 并运行 L3 测试
# ─────────────────────────────────────────────────────────────────────────────
# 在 Windows 上，DLL 通过 PATH 查找（不像 Linux 的 LD_LIBRARY_PATH）
$NewPaths = $LibDirs -join ";"
$env:PATH = "$NewPaths;$env:PATH"

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host " 运行 L3 测试"
Write-Host " PATH 中新增 $($LibDirs.Count) 个 cpp 目录"
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""

Set-Location $RepoRoot

$CargoArgs = @("test", "--test", "l3_run_tests", "--", "--include-ignored", "--test-threads=$Threads")
if ($Filter) {
    $CargoArgs += $Filter
}

& cargo @CargoArgs
