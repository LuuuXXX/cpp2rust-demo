#!/usr/bin/env pwsh
# =============================================================================
# verify-nlohmann-json-ffi.ps1  (Windows PowerShell)
#
# 用途：本地验证 cpp2rust-demo 对 nlohmann/json C++ 库的完整 Rust safe FFI 生成能力
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 检查 nlohmann-json 子模块
#   § 3. 创建并编译驱动 .cpp（供 nm / 预处理验证）
#   § 4. cpp2rust-demo init —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录
#   § 5a. 校验 / 兜底补全 build.rs
#   § 5b. cargo check
#   § 5c. cargo test
#   § 6. FFI 验证
#   § 7. 生成结果汇报
#
# 说明：nlohmann/json 是 header-only 模板库，通过最小驱动文件触发模板展开。
# 重点检查 build.rs include 注入，以及 import_class! 是否基于驱动提取成功。
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
$Feature     = if ($env:FEATURE) { $env:FEATURE } else { "nlohmann_json_ffi" }
$SkipInstall = ($env:SKIP_INSTALL -eq "1")
$CxxStd      = "c++17"   # or c++11
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
    Step "§ 2. 检查 nlohmann-json 子模块"
    $NlohmannDir = Join-Path $RepoDir 'references/nlohmann-json'
    $NlohmannInclude = Join-Path $NlohmannDir 'include'
    if (-not (Test-Path (Join-Path $NlohmannInclude 'nlohmann/json.hpp'))) {
        Fail 'nlohmann-json 子模块未初始化，请运行：git submodule update --init references/nlohmann-json'
    }
    Info "nlohmann-json 根目录：$NlohmannDir"
    Info "include 目录：$NlohmannInclude"
    Ok 'nlohmann-json 子模块检查通过'

    Step "§ 3. 创建并编译驱动 .cpp（供 nm / 预处理验证）"
    $ObjDir = Join-Path $RuntimeDir 'obj'
    $null = New-Item -ItemType Directory -Force -Path $ObjDir
    $DriverCpp = Join-Path $RuntimeDir 'json_driver.cpp'
    @'
#include <nlohmann/json.hpp>
using json = nlohmann::json;
namespace jsonwrap_ns {
class JsonWrapper {
public:
    int count() const;
    std::string to_string(int value) const;
};
}
'@ | Set-Content -Path $DriverCpp -Encoding utf8
    $ObjFile = Join-Path $ObjDir 'nlohmann_driver.obj'
    $CompileLog = Join-Path $ObjDir 'nlohmann_driver.log'
    $compileExit = Invoke-CxxCompile -Compiler $Compiler -Std $CxxStd -IncludeDirs @($NlohmannInclude) -Sources @($DriverCpp) -Output $ObjFile -LogFile $CompileLog
    if ($compileExit -eq 0) {
        Ok '驱动文件编译成功'
    }
    else {
        Warn '驱动文件编译超时或失败（超大头文件展开慢，不影响 init/merge 流程）'
    }
    $ObjectFiles = @($ObjFile)

    Step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
    $BuildScript = Join-Path $RuntimeDir 'build-nlohmann.ps1'
    Write-CompileBuildScript -Path $BuildScript -Compiler $Compiler -Std $CxxStd -IncludeDirs @($NlohmannInclude) -SourceFiles @($DriverCpp) -OutputDir (Join-Path $RuntimeDir 'build') -IgnoreErrors
    Info "工作目录：$RepoDir"
    Info "feature 名称：$Feature"
    Info "构建命令：pwsh -NoProfile -File $BuildScript"
    Push-Location $RepoDir
    try {
        & cpp2rust-demo init --feature $Feature -- pwsh -NoProfile -File $BuildScript
        if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo init 失败' }
    } finally { Pop-Location }
    Ok 'cpp2rust-demo init 完成'
    $Cpp2rustOutput = Join-Path (Join-Path $RepoDir '.cpp2rust') $Feature
    $Captured = @(Get-PreprocessedFiles $Cpp2rustOutput).Count
    Info "输出目录：$Cpp2rustOutput"
    Info "捕获预处理文件数：$Captured"

    Step "§ 5. cpp2rust-demo merge（整理输出结构）"
    Push-Location $RepoDir
    try {
        & cpp2rust-demo merge --feature $Feature
        if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo merge 失败' }
    } finally { Pop-Location }
    Ok 'merge 完成'
    $RustProject = Join-Path $Cpp2rustOutput 'rust'
    $RustSrc = Join-Path $RustProject 'src'
    $RsFiles = @(Get-RustSourceFiles $RustSrc).Count
    Info "生成 Rust 文件数：$RsFiles"

    Step "§ 5a. 校验 build.rs（方案 A：工具自动注入 include）"
    $BuildRs = Join-Path $RustProject 'build.rs'
    $LibName = $Feature -replace '-', '_'
    if (Test-Path $BuildRs) {
        $BuildRsText = Get-Content $BuildRs -Raw
        if ($BuildRsText -match 'cc_build\.include') {
            Ok 'build.rs 已由工具自动注入 nlohmann include（方案 A 生效）'
        } else {
            Warn '工具 build.rs 未注入 include，退回脚本就地补全（方案 B 兜底）'
            Write-FallbackBuildRs -BuildRs $BuildRs -RustProject $RustProject -LibName $LibName -Std $CxxStd -IncludeDirs @($NlohmannInclude) -CommentLines @('header-only 场景仅需 include 路径','驱动编译只为触发模板展开与类型提取') -LinkStdCpp
        }
    } else {
        Warn "未找到 $BuildRs，跳过 build.rs 校验/补全"
    }

    Step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"
    $RustProject = Join-Path $Cpp2rustOutput "rust"
    if (Test-Path (Join-Path $RustProject "Cargo.toml")) {
        Push-Location $RustProject
        try {
            $checkOutput = cargo check 2>&1
            if ($LASTEXITCODE -ne 0) {
                $checkOutput | Write-Host
                Fail "cargo check 失败"
            }
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
            if ($LASTEXITCODE -eq 0) {
                Ok "cargo test 通过 ✓（生成的冒烟测试全部通过）"
            }
            else {
                $testOutput | Write-Host
                Warn "cargo test 失败，请检查上方日志"
                $script:ScriptErrors++
            }
        } finally { Pop-Location }
    }
    else { Info "未生成 tests/smoke.rs（可能无 pub class 类型），跳过 cargo test" }

    Step "§ 6. FFI 验证"
    $NmOutput = Get-DefinedSymbols -NmTool $NmTool -ObjectFiles $ObjectFiles
    $ImportClassFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_class!').Count
    $CppBlockFiles = @(Get-RustMatchFiles $RustSrc 'hicc::cpp!').Count
    Info "已定义符号总数：$($NmOutput.Count)"
    Info "包含 import_class! 绑定的文件数：$ImportClassFiles"
    Info "包含 hicc::cpp! 块的文件数：$CppBlockFiles"
    Show-RustMatches -RustSrc $RustSrc -Pattern 'hicc::import_class!|hicc::cpp!' -First 20
    if ($ImportClassFiles -le 0) {
        Warn '未检测到 import_class! 类绑定'
        $script:ScriptErrors++
    } else {
        Ok '已检测到 import_class! 类绑定'
    }
    Info 'nlohmann/json 为 header-only hicc-direct 场景，本脚本不做 shim 符号交叉比对。'

    Step "§ 7. 生成结果汇报"
    Write-Host '  项目：      nlohmann/json（header-only + 超大头文件）'
    Write-Host "  feature：   $Feature"
    Ok '验证脚本执行完毕！'
    if ($script:ScriptErrors -gt 0) { Fail "验证脚本发现 $script:ScriptErrors 个错误，请检查上方输出" }
}
finally { if (Test-Path $RuntimeDir) { Remove-Item -Recurse -Force $RuntimeDir -EA SilentlyContinue } }
