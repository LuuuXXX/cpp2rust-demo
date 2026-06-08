# build_cpp_libs.ps1 — 批量编译所有 L3 运行测试所需的 C++ 动态库（Windows PowerShell）
#
# 用法：
#   .\scripts\build_cpp_libs.ps1          # 编译全部
#   .\scripts\build_cpp_libs.ps1 001_hello_world  # 只编译指定示例
#
# 前提条件（二选一）：
#   - MinGW/MSYS2 g++ 在 PATH 中（推荐：scoop install mingw）
#   - LLVM clang++ 在 PATH 中（推荐：winget install LLVM.LLVM）
#
# 效果：每个 examples\<NNN_name>\cpp\ 下生成 <name>.dll。

[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$Examples
)

$ErrorActionPreference = "Stop"

# 检测可用编译器（优先 clang++，回退 g++）
$CXX = $null
foreach ($compiler in @("clang++", "g++")) {
    if (Get-Command $compiler -ErrorAction SilentlyContinue) {
        $CXX = $compiler
        break
    }
}
if (-not $CXX) {
    Write-Error "未找到 C++ 编译器（clang++ 或 g++）。请安装 LLVM 或 MinGW/MSYS2。"
    exit 1
}

Write-Host "使用编译器: $CXX"

# 确定要处理的示例列表
if ($Examples.Count -eq 0) {
    $Examples = (Get-ChildItem -Path "examples" -Directory).Name
}

$okCount = 0
$skipCount = 0
$failCount = 0

foreach ($example in $Examples) {
    $cppDir = "examples\$example\cpp"
    if (-not (Test-Path $cppDir)) { continue }

    # 去掉 NNN_ 前缀
    $short = $example -replace '^\d+_', ''
    $dllPath = "$cppDir\$short.dll"

    if (Test-Path $dllPath) {
        Write-Host "  skip  $example ($short.dll 已存在)"
        $skipCount++
        continue
    }

    $cppFiles = Get-ChildItem -Path $cppDir -Filter "*.cpp" -File
    if ($cppFiles.Count -eq 0) {
        Write-Warning "  warn  $example: 没有找到 .cpp 文件，跳过"
        $skipCount++
        continue
    }

    Write-Host -NoNewline "  build $example ... "
    $fileArgs = $cppFiles | ForEach-Object { $_.FullName }
    $proc = Start-Process -FilePath $CXX `
        -ArgumentList (@("-shared") + $fileArgs + @("-o", $dllPath)) `
        -Wait -PassThru -NoNewWindow -RedirectStandardError "$env:TEMP\cpp2rust_err.txt"

    if ($proc.ExitCode -eq 0) {
        Write-Host "✓  ($short.dll)"
        $okCount++
    } else {
        Write-Host "✗  失败"
        if (Test-Path "$env:TEMP\cpp2rust_err.txt") {
            Get-Content "$env:TEMP\cpp2rust_err.txt" | Select-Object -First 20 | Write-Host
        }
        $failCount++
    }
}

Write-Host ""
Write-Host "完成：$okCount 个编译成功，$skipCount 个已跳过，$failCount 个失败"

if ($failCount -gt 0) { exit 1 }
