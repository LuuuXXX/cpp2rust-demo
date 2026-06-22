#!/usr/bin/env pwsh
# =============================================================================
# verify-sqlite3-ffi.ps1  (Windows PowerShell)
#
# 用途：本地验证 cpp2rust-demo 对 sqlite3 C 接口库的完整 Rust safe FFI 生成能力
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 检查 sqlite3.h 头文件
#   § 3. 创建并编译 wrapper .cpp（供 nm / 预处理验证）
#   § 4. cpp2rust-demo init —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录
#   § 5a. 校验 / 兜底补全 build.rs
#   § 5b. cargo check
#   § 5c. cargo test
#   § 6. FFI 验证
#   § 7. 生成结果汇报
#
# 说明：sqlite3 使用系统提供的 extern-C 接口，脚本通过最小 wrapper 触发编译拦截。
# 重点检查 build.rs 是否声明 rustc-link-lib=sqlite3，以及 import_lib! / sqlite3_* 是否生成。
#
# 系统要求（Windows）：
#   winget install Git.Git LLVM.LLVM Rustlang.Rustup
#   # 或通过 scoop / choco / MSYS2 安装
#   # MSVC：Visual Studio 2019+ 含 C++ 工具集
# =============================================================================

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
$Feature     = if ($env:FEATURE) { $env:FEATURE } else { "sqlite3_ffi" }
$SkipInstall = ($env:SKIP_INSTALL -eq "1")
$CxxStd      = "c++11"   # or c++11
$script:ScriptErrors = 0

# ─── 辅助函数 ─────────────────────────────────────────────────────────────────
function Step($msg) {
    Write-Host ""
    Write-Host ("═" * 46) -ForegroundColor Cyan
    Write-Host "  $msg" -ForegroundColor Cyan
    Write-Host ("═" * 46) -ForegroundColor Cyan
}
function Info($msg)  { Write-Host "[INFO]  $msg" -ForegroundColor Cyan }
function Ok($msg)    { Write-Host "[ OK ]  $msg" -ForegroundColor Green }
function Warn($msg)  { Write-Host "[WARN]  $msg" -ForegroundColor Yellow }
function Fail($msg)  { Write-Host "[FAIL]  $msg" -ForegroundColor Red; exit 1 }
function NeedCmd($cmd) {
    if (-not (Get-Command $cmd -EA SilentlyContinue)) {
        Fail "未找到命令：$cmd  请先安装后重试"
    }
}
function Get-CxxCompiler {
    foreach ($c in @("cl", "g++", "clang++")) {
        if (Get-Command $c -EA SilentlyContinue) { return $c }
    }
    return $null
}
function ConvertTo-BuildPath($path) {
    if (Get-Command cygpath -EA SilentlyContinue) { return (cygpath -m $path) }
    return $path
}
function Get-NmTool {
    foreach ($t in @("llvm-nm", "nm")) {
        if (Get-Command $t -EA SilentlyContinue) { return $t }
    }
    return $null
}

# Locate repo root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoDir = (git -C $ScriptDir rev-parse --show-toplevel 2>$null) ?? (Split-Path -Parent $ScriptDir)
$RepoDir = (Resolve-Path $RepoDir).Path

function Use-MsvcCompiler([string]$cmd) {
    $leaf = [System.IO.Path]::GetFileName($cmd)
    return ($leaf -ieq "cl") -or ($leaf -ieq "cl.exe")
}
function Get-CompilerStdArg([string]$compiler, [string]$std) {
    if (Use-MsvcCompiler $compiler) {
        switch ($std) {
            "c++17" { return "/std:c++17" }
            "c++11" { return "/std:c++14" }
            default  { return $null }
        }
    }
    return "-std=$std"
}
function Get-DefineArg([string]$compiler, [string]$define) {
    if (Use-MsvcCompiler $compiler) { return "/D$define" }
    return "-D$define"
}
function ConvertTo-PsArrayLiteral([string[]]$Items) {
    if (-not $Items -or $Items.Count -eq 0) { return "@()" }
    $quoted = $Items | ForEach-Object { "'" + ($_ -replace "'", "''") + "'" }
    return "@(" + ($quoted -join ", ") + ")"
}
function Invoke-CxxCompile {
    param(
        [string]$Compiler,
        [string]$Std,
        [string[]]$IncludeDirs = @(),
        [string[]]$Sources = @(),
        [string]$Output,
        [string]$LogFile,
        [string[]]$ExtraArgs = @()
    )

    $args = @()
    if (Use-MsvcCompiler $Compiler) {
        $args += @("/nologo", "/EHsc", "/c")
        $stdArg = Get-CompilerStdArg $Compiler $Std
        if ($stdArg) { $args += $stdArg }
        foreach ($dir in $IncludeDirs) { $args += "/I$(ConvertTo-BuildPath $dir)" }
        foreach ($arg in $ExtraArgs) { $args += $arg }
        foreach ($src in $Sources) { $args += (ConvertTo-BuildPath $src) }
        if ($Output) { $args += "/Fo$(ConvertTo-BuildPath $Output)" }
    }
    else {
        $args += "-c"
        $stdArg = Get-CompilerStdArg $Compiler $Std
        if ($stdArg) { $args += $stdArg }
        foreach ($dir in $IncludeDirs) { $args += "-I"; $args += (ConvertTo-BuildPath $dir) }
        foreach ($arg in $ExtraArgs) { $args += $arg }
        foreach ($src in $Sources) { $args += (ConvertTo-BuildPath $src) }
        if ($Output) { $args += "-o"; $args += (ConvertTo-BuildPath $Output) }
    }
    if ($LogFile) { & $Compiler @args *> $LogFile } else { & $Compiler @args }
    return $LASTEXITCODE
}
function Write-CompileBuildScript {
    param(
        [string]$Path,
        [string]$Compiler,
        [string]$Std,
        [string[]]$IncludeDirs = @(),
        [string[]]$SourceFiles = @(),
        [string]$OutputDir,
        [string[]]$ExtraArgs = @(),
        [switch]$IgnoreErrors
    )
    $content = @"
[CmdletBinding()]
param()
`$ErrorActionPreference = "Stop"
function Use-MsvcCompiler([string]`$cmd) {
    `$leaf = [System.IO.Path]::GetFileName(`$cmd)
    return (`$leaf -ieq "cl") -or (`$leaf -ieq "cl.exe")
}
function Get-CompilerStdArg([string]`$compiler, [string]`$std) {
    if (Use-MsvcCompiler `$compiler) {
        switch (`$std) {
            "c++17" { return "/std:c++17" }
            "c++11" { return "/std:c++14" }
            default  { return `$null }
        }
    }
    return "-std=`$std"
}
`$Compiler = '__COMPILER__'
`$Std = '__STD__'
`$IncludeDirs = __INCLUDES__
`$SourceFiles = __SOURCES__
`$OutputDir = '__OUTPUT__'
`$ExtraArgs = __EXTRA_ARGS__
New-Item -ItemType Directory -Force -Path `$OutputDir | Out-Null
foreach (`$src in `$SourceFiles) {
    `$base = [System.IO.Path]::GetFileNameWithoutExtension(`$src)
    `$obj = Join-Path `$OutputDir ("`$base.obj")
    `$args = @()
    if (Use-MsvcCompiler `$Compiler) {
        `$args += @("/nologo", "/EHsc", "/c")
        `$stdArg = Get-CompilerStdArg `$Compiler `$Std
        if (`$stdArg) { `$args += `$stdArg }
        foreach (`$dir in `$IncludeDirs) { `$args += "/I`$dir" }
        foreach (`$arg in `$ExtraArgs) { `$args += `$arg }
        `$args += `$src
        `$args += "/Fo`$obj"
    }
    else {
        `$args += "-c"
        `$stdArg = Get-CompilerStdArg `$Compiler `$Std
        if (`$stdArg) { `$args += `$stdArg }
        foreach (`$dir in `$IncludeDirs) { `$args += "-I"; `$args += `$dir }
        foreach (`$arg in `$ExtraArgs) { `$args += `$arg }
        `$args += `$src
        `$args += "-o"
        `$args += `$obj
    }
    & `$Compiler @args *> `$null
    if (`$LASTEXITCODE -ne 0) {
        __ON_ERROR__
    }
}
exit 0
"@
    $content = $content.Replace('__COMPILER__', $Compiler.Replace("'", "''"))
    $content = $content.Replace('__STD__', $Std.Replace("'", "''"))
    $content = $content.Replace('__INCLUDES__', (ConvertTo-PsArrayLiteral @($IncludeDirs | ForEach-Object { ConvertTo-BuildPath $_ })))
    $content = $content.Replace('__SOURCES__', (ConvertTo-PsArrayLiteral @($SourceFiles | ForEach-Object { ConvertTo-BuildPath $_ })))
    $content = $content.Replace('__OUTPUT__', (ConvertTo-BuildPath $OutputDir).Replace("'", "''"))
    $content = $content.Replace('__EXTRA_ARGS__', (ConvertTo-PsArrayLiteral $ExtraArgs))
    $content = $content.Replace('__ON_ERROR__', $(if ($IgnoreErrors) { 'exit 0' } else { 'exit `$LASTEXITCODE' }))
    Set-Content -Path $Path -Value $content -Encoding utf8
}
function Get-RustSourceFiles([string]$RustSrc) {
    if (-not (Test-Path $RustSrc)) { return @() }
    return @(Get-ChildItem -Path $RustSrc -Recurse -File -Filter *.rs | Sort-Object FullName)
}
function Get-RustMatchFiles([string]$RustSrc, [string]$Pattern) {
    $matches = @(); foreach ($file in Get-RustSourceFiles $RustSrc) { if (Select-String -Path $file.FullName -Pattern $Pattern -Quiet -EA SilentlyContinue) { $matches += $file.FullName } }
    return @($matches | Sort-Object -Unique)
}
function Show-RustMatches {
    param([string]$RustSrc,[string]$Pattern,[int]$First = 20)
    $hits = foreach ($file in Get-RustSourceFiles $RustSrc) { Select-String -Path $file.FullName -Pattern $Pattern -EA SilentlyContinue }
    $hits | Select-Object -First $First | ForEach-Object { Write-Host ("{0}:{1}: {2}" -f $_.Path, $_.LineNumber, $_.Line.Trim()) }
}
function Get-PreprocessedFiles([string]$Cpp2rustOutput) {
    $cDir = Join-Path $Cpp2rustOutput 'c'; if (-not (Test-Path $cDir)) { return @() }
    return @(Get-ChildItem -Path $cDir -Recurse -File -Filter *.cpp2rust | Sort-Object FullName)
}
function Get-PreprocessedLineCount([string]$Cpp2rustOutput) { $total = 0; foreach ($file in Get-PreprocessedFiles $Cpp2rustOutput) { $total += @(Get-Content $file.FullName -EA SilentlyContinue).Count }; return $total }
function Write-FallbackBuildRs {
    param([string]$BuildRs,[string]$RustProject,[string]$LibName,[string]$Std,[string[]]$IncludeDirs = @(),[string[]]$SourceFiles = @(),[string[]]$ExtraLines = @(),[string[]]$CommentLines = @(),[switch]$LinkStdCpp)
    $lines = @('fn main() {','    let mut build = hicc_build::Build::new();','    use std::ops::DerefMut;','    let cc_build: &mut cc::Build = build.deref_mut();',('    cc_build.std("{0}");' -f $Std))
    foreach ($comment in $CommentLines) { $lines += ('    // {0}' -f $comment) }
    foreach ($dir in $IncludeDirs) { $lines += ('    cc_build.include("{0}");' -f (ConvertTo-BuildPath $dir).Replace('\\', '/')) }
    foreach ($src in $SourceFiles) { $lines += ('    cc_build.file("{0}");' -f (ConvertTo-BuildPath $src).Replace('\\', '/')) }
    foreach ($extra in $ExtraLines) { $lines += ('    {0}' -f $extra) }
    $lines += '    build'
    foreach ($file in Get-RustSourceFiles (Join-Path $RustProject 'src')) { if ($file.Name -in @('lib.rs','mod.rs')) { continue }; $relative = $file.FullName.Substring($RustProject.Length + 1).Replace('\\', '/'); $lines += ('        .rust_file("{0}")' -f $relative) }
    $lines += ('        .compile("{0}");' -f $LibName)
    if ($LinkStdCpp) { $lines += ''; $lines += '    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]'; $lines += '    println!("cargo::rustc-link-lib=stdc++");' }
    $lines += '}'
    Set-Content -Path $BuildRs -Value ($lines -join "`n") -Encoding utf8
}
function Get-DefinedSymbols { param([string]$NmTool,[string[]]$ObjectFiles) $symbols = @(); foreach ($obj in $ObjectFiles) { if (-not (Test-Path $obj)) { continue }; $output = & $NmTool -C --defined-only -f posix $obj 2>$null; if ($LASTEXITCODE -ne 0) { $output = & $NmTool -C --defined-only $obj 2>$null }; if ($output) { $symbols += $output } }; return @($symbols) }
$RuntimeRoot = Join-Path $RepoDir '.cpp2rust-script-work'; $RunId = "$Feature-$([DateTimeOffset]::UtcNow.ToUnixTimeSeconds())-$PID"; $RuntimeDir = Join-Path $RuntimeRoot $RunId; $null = New-Item -ItemType Directory -Force -Path $RuntimeDir
try {
    Step "§ 0. 环境检查"; NeedCmd git; NeedCmd cargo; $Compiler = Get-CxxCompiler; if (-not $Compiler) { Fail "未找到 C++ 编译器：请先安装 cl、g++ 或 clang++" }; $NmTool = Get-NmTool; if (-not $NmTool) { Fail "未找到 nm 工具：请先安装 llvm-nm 或 nm" }
    Info "仓库根目录：$RepoDir"; Info "C++ 编译器：$Compiler"; Info "nm 工具：$NmTool"; if (Use-MsvcCompiler $Compiler -and $CxxStd -eq 'c++11') { Warn 'MSVC 不提供 /std:c++11，将使用 /std:c++14 兼容编译' }; Ok '环境检查完成'
    Step "§ 1. 安装 cpp2rust-demo"
    if ($SkipInstall -and (Get-Command cpp2rust-demo -EA SilentlyContinue)) { Ok '已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）' } else { $InstallLog = Join-Path $RuntimeDir 'cargo-install.log'; Info '从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…'; & cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo *> $InstallLog; if ($LASTEXITCODE -ne 0) { Write-Host '── cargo install 失败，完整日志：──'; Get-Content $InstallLog | Write-Host; Fail 'cpp2rust-demo 安装失败，请检查上方日志' }; Get-Content $InstallLog | Select-Object -Last 5 | Write-Host; Ok ('cpp2rust-demo 安装完成：{0}' -f (Get-Command cpp2rust-demo).Source) }
    try { & cpp2rust-demo --version 2>$null | Write-Host } catch { }
    Step "§ 2. 定位 sqlite3.h"
    $Sqlite3Header = if ($env:SQLITE3_HEADER) { $env:SQLITE3_HEADER } elseif (Test-Path 'C:\msys64\usr\include\sqlite3.h') { 'C:\msys64\usr\include\sqlite3.h' } elseif (Test-Path 'C:\msys64\mingw64\include\sqlite3.h') { 'C:\msys64\mingw64\include\sqlite3.h' } else { $null }
    if (-not $Sqlite3Header) {
        Warn '未找到 sqlite3.h，可通过以下方式安装：'
        Warn '  scoop install sqlite'
        Warn '  或 vcpkg install sqlite3'
        Warn '  或 MSYS2：pacman -S mingw-w64-x86_64-sqlite3'
        Warn '  安装后设置：$env:SQLITE3_HEADER = ''path\to\sqlite3.h'''
        Fail '缺少 sqlite3.h，无法继续'
    }
    $Sqlite3IncludeDir = Split-Path -Parent $Sqlite3Header
    Ok 'sqlite3 头文件检查通过'

    Step "§ 3. 创建并编译 wrapper .cpp（供 nm / 预处理验证）"
    $ObjDir = Join-Path $RuntimeDir 'obj'
    $null = New-Item -ItemType Directory -Force -Path $ObjDir
    $WrapperCpp = Join-Path $RuntimeDir 'sqlite3_wrapper.cpp'
    @'
extern "C" {
#include <sqlite3.h>
}
'@ | Set-Content -Path $WrapperCpp -Encoding utf8
    $ObjFile = Join-Path $ObjDir 'sqlite3_wrapper.obj'
    $CompileLog = Join-Path $ObjDir 'sqlite3_wrapper.log'
    $compileExit = Invoke-CxxCompile -Compiler $Compiler -Std $CxxStd -IncludeDirs @($Sqlite3IncludeDir) -Sources @($WrapperCpp) -Output $ObjFile -LogFile $CompileLog
    if ($compileExit -ne 0) { Warn 'wrapper 文件编译失败'; $script:ScriptErrors++ } else { Ok 'wrapper 文件编译成功' }
    $ObjectFiles = @($ObjFile)

    Step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
    $BuildScript = Join-Path $RuntimeDir 'build-sqlite3.ps1'
    Write-CompileBuildScript -Path $BuildScript -Compiler $Compiler -Std $CxxStd -IncludeDirs @($Sqlite3IncludeDir) -SourceFiles @($WrapperCpp) -OutputDir (Join-Path $RuntimeDir 'build')
    Push-Location $RepoDir
    try {
        & cpp2rust-demo init --feature $Feature -- pwsh -NoProfile -File $BuildScript
        if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo init 失败' }
    } finally { Pop-Location }
    Ok 'cpp2rust-demo init 完成'
    $Cpp2rustOutput = Join-Path (Join-Path $RepoDir '.cpp2rust') $Feature

    Step "§ 5. cpp2rust-demo merge（整理输出结构）"
    Push-Location $RepoDir
    try { & cpp2rust-demo merge --feature $Feature; if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo merge 失败' } } finally { Pop-Location }
    Ok 'merge 完成'
    $RustProject = Join-Path $Cpp2rustOutput 'rust'
    $RustSrc = Join-Path $RustProject 'src'

    Step "§ 5a. 校验 build.rs（方案 A：工具自动注入 include / link-lib）"
    $BuildRs = Join-Path $RustProject 'build.rs'
    $LibName = $Feature -replace '-', '_'
    if (Test-Path $BuildRs) {
        $BuildRsText = Get-Content $BuildRs -Raw
        if ($BuildRsText -notmatch 'rustc-link-lib=sqlite3') {
            Write-FallbackBuildRs -BuildRs $BuildRs -RustProject $RustProject -LibName $LibName -Std $CxxStd -IncludeDirs @($Sqlite3IncludeDir) -ExtraLines @('println!("cargo::rustc-link-lib=sqlite3");') -CommentLines @('system lib 场景：不额外编译 sqlite3 源文件','fallback build.rs 仅补 include 路径与 sqlite3 链接声明') -LinkStdCpp
        }
    }

    Step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"
    $RustProject = Join-Path $Cpp2rustOutput "rust"
    if (Test-Path (Join-Path $RustProject "Cargo.toml")) {
        Push-Location $RustProject
        try {
            $checkOutput = cargo check 2>&1
            if ($LASTEXITCODE -ne 0) { $checkOutput | Write-Host; Fail "cargo check 失败" }
            Ok "cargo check 通过 ✓"
        } finally { Pop-Location }
    }
    else { Warn "未找到 $RustProject/Cargo.toml，跳过 cargo check" }

    Step "§ 5c. cargo test（验证生成的冒烟测试可编译、可链接、可运行）"
    $RustProject = Join-Path $Cpp2rustOutput "rust"
    $SmokeFile = Join-Path $RustProject "tests/smoke.rs"
    if (Test-Path $SmokeFile) {
        Info "检测到冒烟测试文件：$SmokeFile"
        Push-Location $RustProject
        try {
            $testOutput = cargo test 2>&1
            if ($LASTEXITCODE -eq 0) { Ok "cargo test 通过 ✓（生成的冒烟测试全部通过）" }
            else { $testOutput | Write-Host; Warn "cargo test 失败，请检查上方日志"; $script:ScriptErrors++ }
        } finally { Pop-Location }
    }
    else { Info "未生成 tests/smoke.rs（可能无 pub class 类型），跳过 cargo test" }

    Step "§ 6. FFI 验证"
    $ImportLibFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_lib!').Count
    $SqliteFnHits = 0
    foreach ($file in Get-RustSourceFiles $RustSrc) {
        $SqliteFnHits += @(Select-String -Path $file.FullName -Pattern 'sqlite3_' -EA SilentlyContinue).Count
    }
    if ($ImportLibFiles -le 0 -or $SqliteFnHits -le 0) { Warn '未完整检测到 sqlite3 import_lib! / sqlite3_* 输出'; $script:ScriptErrors++ } else { Ok '已检测到 sqlite3 extern-C import_lib! 绑定' }

    Step "§ 7. 生成结果汇报"
    Ok '验证脚本执行完毕！'
    if ($script:ScriptErrors -gt 0) { Fail "验证脚本发现 $script:ScriptErrors 个错误，请检查上方输出" }
}
finally { if (Test-Path $RuntimeDir) { Remove-Item -Recurse -Force $RuntimeDir -EA SilentlyContinue } }
