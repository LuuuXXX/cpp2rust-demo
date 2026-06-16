#!/usr/bin/env bash
# build_cpp_libs.sh — 批量编译所有 L3 运行测试所需的 C++ 动态库
#
# 用法：
#   bash scripts/build_cpp_libs.sh          # 编译全部
#   bash scripts/build_cpp_libs.sh 001_hello_world  # 只编译指定示例（可传多个）
#
# 效果：
#   每个 examples/<NNN_name>/cpp/ 目录下会生成 lib<name>.so（Linux）。
#
# 已存在的库文件会被跳过（增量编译）。若想强制重新编译，先删除对应的 .so。

set -euo pipefail

# 选定编译器和链接选项
CXX="${CXX:-g++}"
SHARED_FLAGS="-shared -fPIC"
LIB_EXT="so"

# 如果传入了具体示例名，只处理这些；否则扫描全部
if [ $# -gt 0 ]; then
    EXAMPLES=("$@")
else
    EXAMPLES=()
    for dir in examples/*/; do
        EXAMPLES+=("$(basename "$dir")")
    done
fi

ok_count=0
skip_count=0
fail_count=0

for example in "${EXAMPLES[@]}"; do
    cpp_dir="examples/${example}/cpp"
    if [ ! -d "$cpp_dir" ]; then
        continue
    fi

    # 去掉 NNN_ 前缀得到短名
    short="${example#*_}"
    lib_file="${cpp_dir}/lib${short}.${LIB_EXT}"

    if [ -f "$lib_file" ]; then
        echo "  skip  ${example} (已存在 lib${short}.${LIB_EXT})"
        skip_count=$((skip_count + 1))
        continue
    fi

    # 收集 .cpp 文件
    cpp_files=()
    while IFS= read -r -d '' f; do
        cpp_files+=("$f")
    done < <(find "$cpp_dir" -maxdepth 1 -name "*.cpp" -print0)

    if [ ${#cpp_files[@]} -eq 0 ]; then
        echo "  warn  ${example}: 没有找到 .cpp 文件，跳过"
        skip_count=$((skip_count + 1))
        continue
    fi

    echo -n "  build ${example} ... "
    # shellcheck disable=SC2086
    if $CXX $SHARED_FLAGS "${cpp_files[@]}" -o "$lib_file" 2>/tmp/cpp2rust_build_err.txt; then
        echo "✓  (lib${short}.${LIB_EXT})"
        ok_count=$((ok_count + 1))
    else
        echo "✗  失败"
        head -20 /tmp/cpp2rust_build_err.txt
        fail_count=$((fail_count + 1))
    fi
done

echo ""
echo "完成：${ok_count} 个编译成功，${skip_count} 个已跳过，${fail_count} 个失败"

if [ "$fail_count" -gt 0 ]; then
    exit 1
fi
