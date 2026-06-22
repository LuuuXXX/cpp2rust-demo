#!/usr/bin/env pwsh
# =============================================================================
# verify-fmtlib-ffi.ps1  (Windows PowerShell)
#
# 用途：本地验证 cpp2rust-demo 对 fmtlib C++ 库的完整 Rust safe FFI 生成能力
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 检查 fmtlib 子模块
#   § 3. 编译 fmtlib .cc 目标文件（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录
#   § 5a. 校验 / 兜底补全 build.rs
#   § 5b. cargo check
#   § 5c. cargo test
#   § 6. FFI 验证
#   § 7. 生成结果汇报
#
# 说明：fmtlib 为多翻译单元库，核心实现位于 src/format.cc 与 src/os.cc。
# 重点检查 build.rs 是否注入两份 .cc 实现，以及 import_lib! / import_class! 是否均已生成。
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
$Feature     = if ($env:FEATURE) { $env:FEATURE } else { "fmtlib_ffi" }
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
    Step "§ 2. 检查 fmtlib 子模块"
    $FmtDir = Join-Path $RepoDir 'references/fmtlib'
    $FmtInclude = Join-Path $FmtDir 'include'
    $FmtSrc = Join-Path $FmtDir 'src'
    $FmtSources = @(Join-Path $FmtSrc 'format.cc'; Join-Path $FmtSrc 'os.cc')
    if (-not (Test-Path (Join-Path $FmtSrc 'format.cc'))) { Fail 'fmtlib 子模块未初始化，请运行：git submodule update --init references/fmtlib' }
    Info "fmtlib 根目录：$FmtDir"
    Info "include 目录：$FmtInclude"
    Info "src 目录：$FmtSrc"
    $FmtSources | ForEach-Object { Write-Host "  - $_" }
    Ok 'fmtlib 子模块检查通过'
    Step "§ 3. 编译 fmtlib .cc 目标文件（供 nm 符号验证）"
    $ObjDir = Join-Path $RuntimeDir 'obj'
    $null = New-Item -ItemType Directory -Force -Path $ObjDir
    $ObjectFiles = @(); $CompileErrors = 0
    foreach ($src in $FmtSources) { $base = [System.IO.Path]::GetFileNameWithoutExtension($src); $obj = Join-Path $ObjDir ($base + '.obj'); $log = Join-Path $ObjDir ($base + '.log'); $exitCode = Invoke-CxxCompile -Compiler $Compiler -Std $CxxStd -IncludeDirs @($FmtInclude) -Sources @($src) -Output $obj -LogFile $log; $ObjectFiles += $obj; if ($exitCode -eq 0) { Info "编译成功：$($base).obj" } else { Warn "编译失败：$src"; if (Test-Path $log) { Get-Content $log | Write-Host }; $CompileErrors++ } }
    if ($CompileErrors -eq 0) { Ok '全部 fmtlib 源文件编译成功' } else { Warn "$CompileErrors 个 fmtlib 源文件编译失败，后续 nm 验证可能不完整" }

    Step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
    $BuildScript = Join-Path $RuntimeDir 'build-fmtlib.ps1'
    Write-CompileBuildScript -Path $BuildScript -Compiler $Compiler -Std $CxxStd -IncludeDirs @($FmtInclude) -SourceFiles $FmtSources -OutputDir (Join-Path $RuntimeDir 'build')
    Info "工作目录：$RepoDir"; Info "feature 名称：$Feature"; Info "构建命令：pwsh -NoProfile -File $BuildScript"
    Push-Location $RepoDir
    try { & cpp2rust-demo init --feature $Feature -- pwsh -NoProfile -File $BuildScript; if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo init 失败' } } finally { Pop-Location }
    Ok 'cpp2rust-demo init 完成'
    $Cpp2rustOutput = Join-Path (Join-Path $RepoDir '.cpp2rust') $Feature; Info "输出目录：$Cpp2rustOutput"; $Captured = @(Get-PreprocessedFiles $Cpp2rustOutput).Count; Info "捕获预处理文件数：$Captured"


    Step "§ 5. cpp2rust-demo merge（整理输出结构）"
    Push-Location $RepoDir
    try { & cpp2rust-demo merge --feature $Feature; if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo merge 失败' } } finally { Pop-Location }
    Ok 'merge 完成'
    $RustProject = Join-Path $Cpp2rustOutput 'rust'; $RustSrc = Join-Path $RustProject 'src'; $RsFiles = @(Get-RustSourceFiles $RustSrc).Count; Info "生成 Rust 文件数：$RsFiles"
    if ($RsFiles -gt 0) { Write-Host '──── 生成的 .rs 文件（前 20 条）────'; Get-RustSourceFiles $RustSrc | Select-Object -First 20 | ForEach-Object { Write-Host "  $($_.FullName)" } }
    Write-Host ''; Info '降级标记统计（cpp2rust-todo）：'; $TodoHits = foreach ($file in Get-RustSourceFiles $RustSrc) { Select-String -Path $file.FullName -Pattern 'cpp2rust-todo' -EA SilentlyContinue }
    if ($TodoHits) { $TodoHits | Group-Object { $_.Line.Trim() } | Sort-Object Count -Descending | ForEach-Object { Write-Host ("  {0,4}  {1}" -f $_.Count, $_.Name) } } else { Write-Host '  （无降级标记）' }

    Step "§ 5a. 校验 build.rs（方案 A：工具自动注入 include + .cc 实现）"
    $BuildRs = Join-Path $RustProject 'build.rs'
    $LibName = $Feature -replace '-', '_'
    if (Test-Path $BuildRs) { Info '工具生成的 build.rs：'; Get-Content $BuildRs | ForEach-Object { Write-Host "    $_" }; $BuildRsText = Get-Content $BuildRs -Raw; if ($BuildRsText -match 'cc_build\.include' -and $BuildRsText -match 'cc_build\.file') { Ok 'build.rs 已由工具自动注入头路径 + fmtlib .cc 实现（方案 A 生效）' } else { Warn '工具 build.rs 未同时注入 include/file，退回脚本就地补全（方案 B 兜底）'; Write-FallbackBuildRs -BuildRs $BuildRs -RustProject $RustProject -LibName $LibName -Std $CxxStd -IncludeDirs @($FmtInclude) -SourceFiles $FmtSources -CommentLines @('与 fmtlib 源码编译保持一致的 C++ 标准','① fmt include 路径（供 hicc 胶水 C++ #include 使用）','② 编译 format.cc 与 os.cc，提供底层实现符号') -LinkStdCpp; Ok 'build.rs 已就地补全（注入 fmt include + .cc 文件 + 逐文件注册）' } } else { Warn "未找到 $BuildRs，跳过 build.rs 校验/补全" }

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
            if ($LASTEXITCODE -eq 0) { Ok "cargo test 通过 ✓（生成的冒烟测试全部通过）" }
            else { $testOutput | Write-Host; Warn "cargo test 失败，请检查上方日志"; $script:ScriptErrors++ }
        } finally { Pop-Location }
    }
    else { Info "未生成 tests/smoke.rs（可能无 pub class 类型），跳过 cargo test" }

    Step "§ 6. FFI 验证"
    $NmOutput = Get-DefinedSymbols -NmTool $NmTool -ObjectFiles $ObjectFiles
    $FmtSymbols = @($NmOutput | Where-Object { $_ -match 'fmt::' })
    Info "nm 输出中匹配 fmt:: 的符号数：$($FmtSymbols.Count)"
    if ($FmtSymbols.Count -gt 0) { Ok 'fmtlib 目标文件中检测到 fmt:: 符号' } else { Warn '未在目标文件中检测到 fmt:: 符号'; $script:ScriptErrors++ }
    $ImportLibFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_lib!').Count
    $ImportClassFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_class!').Count
    Info "包含 import_lib! 绑定的文件数：$ImportLibFiles"
    Info "包含 import_class! 绑定的文件数：$ImportClassFiles"
    Show-RustMatches -RustSrc $RustSrc -Pattern 'hicc::import_lib!|hicc::import_class!' -First 20
    if ($ImportLibFiles -gt 0 -and $ImportClassFiles -gt 0) { Ok '生成代码同时包含 import_lib! 与 import_class! 块 ✓' } else { Warn 'fmtlib 预期同时生成 import_lib! 与 import_class!'; $script:ScriptErrors++ }
    Info 'fmtlib 为 hicc-direct 多源文件场景，本脚本不做 shim 符号交叉比对。'
    Step "§ 7. 生成结果汇报"
    Write-Host '  项目：      fmtlib（多源文件 + .cc 扩展名）'
    Write-Host "  feature：   $Feature"
    Write-Host "  fmt 目录：  $FmtDir"
    $ImportLibFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_lib!').Count
    $ImportClassFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_class!').Count
    Write-Host "  import_lib! 文件数：    $ImportLibFiles"
    Write-Host "  import_class! 文件数：  $ImportClassFiles"
    Ok '验证脚本执行完毕！'
    if ($script:ScriptErrors -gt 0) { Fail "验证脚本发现 $script:ScriptErrors 个错误，请检查上方输出" }
}
finally { if (Test-Path $RuntimeDir) { Remove-Item -Recurse -Force $RuntimeDir -EA SilentlyContinue } }
