# 工作流决策：何时直出 / 何时写 shim

cpp2rust-demo 把 C++ 库转换为 Rust safe FFI 时，有两条工作流。本文用一张决策表说明
如何为目标库选择合适的路径，并链接到对应的本地验证脚本。

## 一、两类工作流

| 工作流 | 适用对象 | 做法 | 产物 |
|--------|----------|------|------|
| **直出（默认）** | 含命名空间类 / 自由函数的常规 C++ 库，以及纯 C 接口库 | 直接对库的实现 `.cpp`（或最小驱动 `.cpp`）运行 `init`，工具自动绑定 | `import_class!`（类方法）/ `import_lib!`（自由函数、C 接口） |
| **shim** | 纯 C++、无可导出类，或需精确控制 ABI 的库 | 先手写一层 `extern "C"` 不透明句柄包装层（shim），再对 shim 运行 `init` | `import_lib!`（shim 导出的 extern-C 函数） |

## 二、决策表

| 目标库特征 | 推荐工作流 | 说明 |
|------------|-----------|------|
| 有命名空间作用域的公有类（含方法） | **直出** | 工具直出 `import_class!` 绑定类方法；构造经 `import_lib!` 的工厂 |
| 有命名空间作用域、在实现 `.cpp` 中定义的自由函数 | **直出** | 工具直出 `import_lib!`（仅头文件 inline 定义的函数不会被收集） |
| 纯 C 接口（`extern "C"` 头，如 sqlite3） | **直出** | 用 `extern "C" { #include <...> }` 的 wrapper 即可生成 `import_lib!` |
| header-only、重度模板 / constexpr（nlohmann-json、magic_enum、toml++、fmt header 模式） | **直出** | 写仅声明、标量/std 签名的最小驱动 `.cpp` 触发解析，验证可编译性 |
| 纯 C++ 库、对外只暴露模板/内联、几乎无可绑定的具体类 | **shim** | 手写 extern-C 句柄包装层暴露稳定 ABI（如 rapidjson） |
| 需要严格隔离 ABI / 跨编译器稳定符号 | **shim** | 由 shim 显式定义导出符号与生命周期管理 |

## 三、各库对应的验证脚本

| 库 | 工作流 | 本地验证脚本 |
|----|--------|--------------|
| tinyxml2 | 直出 | [`usage/verify-tinyxml2.sh`](../usage/verify-tinyxml2.sh) |
| pugixml | 直出 | [`usage/verify-pugixml.sh`](../usage/verify-pugixml.sh) |
| nlohmann/json | 直出（header-only） | [`usage/verify-nlohmann-json.sh`](../usage/verify-nlohmann-json.sh) |
| fmtlib | 直出 | [`usage/verify-fmtlib.sh`](../usage/verify-fmtlib.sh) |
| magic_enum | 直出（header-only） | [`usage/verify-magic-enum.sh`](../usage/verify-magic-enum.sh) |
| tomlplusplus | 直出（header-only） | [`usage/verify-tomlplusplus.sh`](../usage/verify-tomlplusplus.sh) |
| sqlite3 | C 接口 | [`usage/verify-sqlite3.sh`](../usage/verify-sqlite3.sh) |
| rapidjson | shim | [`usage/verify-rapidjson-ffi.sh`](../usage/verify-rapidjson-ffi.sh) |

> 一键批量运行全部直出/ C 接口脚本：`SKIP_INSTALL=1 bash usage/verify-all.sh`
> （可用 `ONLY=tinyxml2,fmtlib` 选择子集）。

更详细的脚本用法、环境变量与符号验证说明见 [`usage/README.md`](../usage/README.md)。
