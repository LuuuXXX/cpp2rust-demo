#!/usr/bin/env pwsh
# =============================================================================
# verify-rapidjson-ffi.ps1  (Windows PowerShell)
#
# 用途：本地验证 cpp2rust-demo 对 rapidjson C++ 库的完整 Rust safe FFI 生成能力
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位本地 shim 文件（references/rapidjson-refactoring/rapidjson_sys/shim）
#   § 3. 编译 shim 目标文件（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录
#   § 5a. 校验 / 兜底补全 build.rs
#   § 5b. cargo check
#   § 5c. cargo test
#   § 6. FFI 验证
#   § 7. 生成结果汇报
#
# 说明：rapidjson 为 shim 模式，依赖 references/rapidjson-refactoring/rapidjson_sys/shim
# 中的 extern "C" 包装层。Windows 当前总是走方案 B：hook_shim 不写 .opts，build.rs
# 由脚本直接兜底补全头路径、shim 源文件与 rust_file 注册。
# =============================================================================

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
$Feature     = if ($env:FEATURE) { $env:FEATURE } else { "rapidjson_shim" }
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
    Info "仓库根目录：$RepoDir"; Info "C++ 编译器：$Compiler"; Info "nm 工具：$NmTool"; if (Use-MsvcCompiler $Compiler -and $CxxStd -eq 'c++11') { Warn 'MSVC 不提供 /std:c++11，将使用 /std:c++14 兼容编译' }; Info 'Windows 当前总是走方案 B：hook_shim 不写 .opts，build.rs 将由脚本兜底补全'; Ok '环境检查完成'
    Step "§ 1. 安装 cpp2rust-demo"
    if ($SkipInstall -and (Get-Command cpp2rust-demo -EA SilentlyContinue)) { Ok '已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）' } else { $InstallLog = Join-Path $RuntimeDir 'cargo-install.log'; Info '从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…'; & cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo *> $InstallLog; if ($LASTEXITCODE -ne 0) { Write-Host '── cargo install 失败，完整日志：──'; Get-Content $InstallLog | Write-Host; Fail 'cpp2rust-demo 安装失败，请检查上方日志' }; Get-Content $InstallLog | Select-Object -Last 5 | Write-Host; Ok ('cpp2rust-demo 安装完成：{0}' -f (Get-Command cpp2rust-demo).Source) }
    try { & cpp2rust-demo --version 2>$null | Write-Host } catch { }
    Step "§ 2. 定位本地 shim 文件"
    $ShimDir = Join-Path $RepoDir 'references/rapidjson-refactoring/rapidjson_sys/shim'
    $RapidjsonInclude = Join-Path $RepoDir 'references/rapidjson-refactoring/rapidjson_legacy/include'
    if (-not (Test-Path $ShimDir)) { Fail 'rapidjson shim 目录不存在，请确认仓库中已初始化 references/rapidjson-refactoring/' }
    $ShimFiles = @(Get-ChildItem -Path $ShimDir -Filter '*.cpp' | Sort-Object Name)
    if ($ShimFiles.Count -eq 0) { Fail "shim 目录中未找到 .cpp 文件：$ShimDir" }
    Info "shim 目录：$ShimDir"
    Info "shim 源文件数量：$($ShimFiles.Count)"
    $ShimFiles | ForEach-Object { Write-Host "  - $($_.Name)" }
    Info "rapidjson include：$RapidjsonInclude"
    Ok 'shim 文件就绪'

    Step "§ 3. 编译 shim 目标文件（供 nm 符号验证）"
    $ObjDir = Join-Path $RuntimeDir 'obj'
    $null = New-Item -ItemType Directory -Force -Path $ObjDir
    $CompileErrors = 0
    foreach ($src in $ShimFiles) {
        $objFile = Join-Path $ObjDir "$($src.BaseName).obj"
        $logFile = Join-Path $ObjDir "$($src.BaseName).log"
        $exit = Invoke-CxxCompile -Compiler $Compiler -Std $CxxStd -IncludeDirs @($RapidjsonInclude, $ShimDir) -Sources @($src.FullName) -Output $objFile -LogFile $logFile
        if ($exit -eq 0) { Info "编译成功：$($src.BaseName).obj" }
        else { Warn "编译失败：$($src.Name)"; if (Test-Path $logFile) { Get-Content $logFile | Write-Host }; $CompileErrors++ }
    }
    if ($CompileErrors -gt 0) { Warn "$CompileErrors 个 shim 文件编译失败，nm 验证可能不完整" }
    else { Ok "全部 $($ShimFiles.Count) 个 shim 文件编译成功" }
    $ObjectFiles = @(Get-ChildItem -Path $ObjDir -Filter '*.obj' | Sort-Object Name | Select-Object -ExpandProperty FullName)

    Step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
    $BuildScript = Join-Path $RuntimeDir 'build-rapidjson.ps1'
    $ShimSources = @($ShimFiles | Select-Object -ExpandProperty FullName)
    Write-CompileBuildScript -Path $BuildScript -Compiler $Compiler -Std $CxxStd -IncludeDirs @($RapidjsonInclude, $ShimDir) -SourceFiles $ShimSources -OutputDir (Join-Path $RuntimeDir 'build') -IgnoreErrors
    Info "工作目录：$RepoDir"; Info "feature 名称：$Feature"; Info "构建命令：pwsh -NoProfile -File $BuildScript"
    Push-Location $RepoDir
    try { & cpp2rust-demo init --feature $Feature -- pwsh -NoProfile -File $BuildScript; if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo init 失败' } } finally { Pop-Location }
    Ok 'cpp2rust-demo init 完成'
    $Cpp2rustOutput = Join-Path (Join-Path $RepoDir '.cpp2rust') $Feature
    Info "输出目录：$Cpp2rustOutput"
    $Captured = @(Get-PreprocessedFiles $Cpp2rustOutput).Count
    Info "捕获预处理文件数：$Captured"

    Step "§ 5. cpp2rust-demo merge（整理输出结构）"
    Push-Location $RepoDir
    try { & cpp2rust-demo merge --feature $Feature; if ($LASTEXITCODE -ne 0) { Fail 'cpp2rust-demo merge 失败' } } finally { Pop-Location }
    Ok 'merge 完成'
    $RustProject = Join-Path $Cpp2rustOutput 'rust'; $RustSrc = Join-Path $RustProject 'src'; $RsFiles = @(Get-RustSourceFiles $RustSrc).Count; Info "生成 Rust 文件数：$RsFiles"
    if ($RsFiles -gt 0) { Write-Host '──── 生成的 .rs 文件（前 20 条）────'; Get-RustSourceFiles $RustSrc | Select-Object -First 20 | ForEach-Object { Write-Host "  $($_.FullName)" } }
    Write-Host ''; Info '降级标记统计（cpp2rust-todo）：'; $TodoHits = foreach ($file in Get-RustSourceFiles $RustSrc) { Select-String -Path $file.FullName -Pattern 'cpp2rust-todo' -EA SilentlyContinue }
    if ($TodoHits) { $TodoHits | Group-Object { $_.Line.Trim() } | Sort-Object Count -Descending | ForEach-Object { Write-Host ("  {0,4}  {1}" -f $_.Count, $_.Name) } } else { Write-Host '  （无降级标记）' }

    Step "§ 5a. 校验 build.rs（方案 B：脚本兜底注入头路径与 shim 实现）"
    $BuildRs = Join-Path $RustProject 'build.rs'
    $LibName = $Feature -replace '-', '_'
    if (Test-Path $BuildRs) {
        Info '工具生成的 build.rs：'
        Get-Content $BuildRs | ForEach-Object { Write-Host "    $_" }
        Warn 'Windows 当前总是走方案 B（hook_shim 不写 .opts），脚本将就地补全 build.rs'
        Write-FallbackBuildRs -BuildRs $BuildRs -RustProject $RustProject -LibName $LibName -Std $CxxStd -IncludeDirs @($RapidjsonInclude, $ShimDir) -SourceFiles $ShimSources -CommentLines @('与 shim 编译保持一致的 C++ 标准','① rapidjson 与 shim 头文件包含路径（hicc 生成的胶水 C++ #include 需要）','② 编译 shim 的 .cpp 文件，提供 rapidjson_* extern "C" 实现符号') -LinkStdCpp
        Info '兜底补全后的 build.rs：'
        Get-Content $BuildRs | ForEach-Object { Write-Host "    $_" }
        Ok 'build.rs 已就地补全（注入头路径 + shim 实现 + 逐文件注册）'
    } else { Warn "未找到 $BuildRs，跳过 build.rs 校验/补全" }

    Step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"
    $RustProject = Join-Path $Cpp2rustOutput 'rust'
    if (Test-Path (Join-Path $RustProject 'Cargo.toml')) {
        Push-Location $RustProject
        try {
            $checkOutput = cargo check 2>&1
            if ($LASTEXITCODE -ne 0) {
                $checkOutput | Write-Host
                Fail 'cargo check 失败'
            }
            Ok 'cargo check 通过 ✓'
        } finally { Pop-Location }
    }
    else { Warn "未找到 $RustProject/Cargo.toml，跳过 cargo check" }

    Step "§ 5c. cargo test（验证生成的冒烟测试可编译、可链接、可运行）"
    $RustProject = Join-Path $Cpp2rustOutput 'rust'
    $SmokeFile = Join-Path $RustProject 'tests/smoke.rs'
    if (Test-Path $SmokeFile) {
        Info "检测到冒烟测试文件：$SmokeFile"
        Push-Location $RustProject
        try {
            $testOutput = cargo test 2>&1
            if ($LASTEXITCODE -eq 0) { Ok 'cargo test 通过 ✓（生成的冒烟测试全部通过）' }
            else { $testOutput | Write-Host; Warn 'cargo test 失败，请检查上方日志'; $script:ScriptErrors++ }
        } finally { Pop-Location }
    }
    else { Info '未生成 tests/smoke.rs（可能 init 阶段无 pub class 类型），跳过 cargo test' }

    Step "§ 6. FFI 验证"
    Write-Host "`n6a. shim 目标文件 extern-C 符号（nm 验证）"
    $NmSymbols = Get-DefinedSymbols -NmTool $NmTool -ObjectFiles $ObjectFiles
    $TSymbols = @($NmSymbols | Where-Object { $_ -match '(^|\s)T(\s|$)' })
    Info "shim 目标文件已定义符号数：$($NmSymbols.Count)"
    Info "shim 目标文件中 T 段定义符号数：$($TSymbols.Count)"
    if ($TSymbols.Count -gt 0) { $TSymbols | Select-Object -First 20 | ForEach-Object { Write-Host "  $_" }; Ok 'shim 文件包含 extern-C 导出符号' } else { Warn '未找到 T 段符号（shim 文件是否编译成功？）'; $script:ScriptErrors++ }

    Write-Host "`n6b. 生成 Rust 代码中的 FFI 声明（import_lib! / import_class!）"
    $ImportLibFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_lib!').Count
    $ImportClassFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_class!').Count
    Info "包含 import_lib! 绑定的文件数：$ImportLibFiles"
    Info "包含 import_class! 绑定的文件数：$ImportClassFiles"
    Show-RustMatches -RustSrc $RustSrc -Pattern 'hicc::import_class!|hicc::import_lib!' -First 20
    if ($ImportLibFiles -gt 0) { Ok '生成代码包含 import_lib! 块（FFI 绑定存在）✓' } else { Warn '生成代码中未找到 import_lib! 块'; $script:ScriptErrors++ }

    Write-Host "`n6c. FFI 函数名交叉比对（生成代码 vs. shim 目标文件 nm 符号）"
    $GeneratedFuncs = @()
    foreach ($file in Get-RustSourceFiles $RustSrc) {
        $hits = Select-String -Path $file.FullName -Pattern '#\[cpp\(func = "([^"]+)"\)\]' -EA SilentlyContinue
        foreach ($hit in $hits) { if ($hit.Matches) { foreach ($m in $hit.Matches) { $GeneratedFuncs += $m.Groups[1].Value } } }
    }
    $GeneratedFuncs = @($GeneratedFuncs | Sort-Object -Unique)
    Info "生成的函数绑定数：$($GeneratedFuncs.Count)"
    foreach ($fname in ($GeneratedFuncs | Select-Object -First 15)) {
        $found = $NmSymbols | Where-Object { $_ -match [regex]::Escape($fname) }
        if ($found) { Write-Host "  $fname  ✓ 在目标文件中找到" -ForegroundColor Green }
        else { Write-Host "  $fname  ? 未在目标文件中直接找到（可能在 hicc cpp! 宏展开后才出现）" -ForegroundColor Yellow }
    }

    Write-Host "`n6d. 捕获的 .cpp2rust 预处理文件大小统计"
    $PreFiles = Get-PreprocessedFiles $Cpp2rustOutput
    if ($PreFiles.Count -gt 0) { Info "预处理文件总行数：$(Get-PreprocessedLineCount $Cpp2rustOutput)"; $PreFiles | ForEach-Object { [PSCustomObject]@{ Lines = @(Get-Content $_.FullName).Count; Path = $_.FullName } } | Sort-Object Lines -Descending | Select-Object -First 15 | ForEach-Object { Write-Host ("  {0,8}  {1}" -f $_.Lines, $_.Path) } } else { Warn '未找到预处理目录或 .cpp2rust 文件' }

    Write-Host "`n6e. struct/class 前缀 & restrict 清理验证"
    $StructHits = foreach ($file in Get-RustSourceFiles $RustSrc) { Select-String -Path $file.FullName -Pattern '\bstruct \b' -EA SilentlyContinue | Where-Object { $_.Line -notmatch '^\s*//' -and $_.Line -notmatch 'hicc::cpp!' } }
    $ClassHits = foreach ($file in Get-RustSourceFiles $RustSrc) { Select-String -Path $file.FullName -Pattern '\bclass \b' -EA SilentlyContinue | Where-Object { $_.Line -notmatch '^\s*//' -and $_.Line -notmatch 'hicc::cpp!' } }
    $RestrictHits = foreach ($file in Get-RustSourceFiles $RustSrc) { Select-String -Path $file.FullName -Pattern '__restrict|(?<!_)restrict(?!_)' -EA SilentlyContinue | Where-Object { $_.Line -notmatch '^\s*//' } }
    if (@($StructHits).Count -eq 0 -and @($ClassHits).Count -eq 0) { Ok 'Rust 绑定中无多余的 struct/class 前缀（特性④ 通过）' } else { Warn "Rust 绑定中仍有 struct/class 前缀：struct=$(@($StructHits).Count) class=$(@($ClassHits).Count)"; @($StructHits + $ClassHits) | Select-Object -First 10 | ForEach-Object { Write-Host ("  {0}:{1}: {2}" -f $_.Path, $_.LineNumber, $_.Line.Trim()) } }
    if (@($RestrictHits).Count -eq 0) { Ok 'Rust 绑定中无 restrict 限定符（特性⑤ 通过）' } else { Warn "Rust 绑定中仍有 restrict 限定符：$(@($RestrictHits).Count) 处"; @($RestrictHits) | Select-Object -First 10 | ForEach-Object { Write-Host ("  {0}:{1}: {2}" -f $_.Path, $_.LineNumber, $_.Line.Trim()) } }

    Step "§ 7. 生成结果汇报"
    Write-Host ''
    Write-Host '  项目：      rapidjson（通过 extern-C shim 层）'
    Write-Host "  feature：   $Feature"
    Write-Host "  shim 目录： $ShimDir"
    Write-Host "  输出目录：  $Cpp2rustOutput"
    Write-Host "  捕获预处理文件数：  $Captured"
    Write-Host "  生成 Rust 文件数：  $RsFiles"
    $ImportLibFiles = @(Get-RustMatchFiles $RustSrc 'hicc::import_lib!').Count
    $TotalFnBindings = 0
    foreach ($file in Get-RustSourceFiles $RustSrc) { $TotalFnBindings += @(Select-String -Path $file.FullName -Pattern '#\[cpp\(func = "([^"]+)"\)\]' -EA SilentlyContinue).Count }
    Write-Host "  import_lib! 文件数：      $ImportLibFiles"
    Write-Host "  FFI 函数绑定总数：       $TotalFnBindings"
    $TodoCount = 0
    foreach ($file in Get-RustSourceFiles $RustSrc) { $TodoCount += @(Select-String -Path $file.FullName -Pattern 'cpp2rust-todo' -EA SilentlyContinue).Count }
    if ($ImportLibFiles -gt 0) { Ok '已生成 rapidjson Rust safe FFI 脚手架' } else { Warn '未检测到 import_lib! 绑定'; $script:ScriptErrors++ }
    if ($TodoCount -gt 0) { Warn "检测到降级标记（需手动完善）：$TodoCount 处" } else { Ok '未检测到降级标记' }
    Ok '验证脚本执行完毕！'
    if ($script:ScriptErrors -gt 0) { Fail "验证脚本发现 $script:ScriptErrors 个错误，请检查上方输出" }
}
finally { if (Test-Path $RuntimeDir) { Remove-Item -Recurse -Force $RuntimeDir -EA SilentlyContinue } }
