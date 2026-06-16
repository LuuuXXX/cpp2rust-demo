# cpp2rust-demo 快捷目标
#
# 使用方式：
#   make l3-setup   — 编译所有 L3 测试所需的 C++ 动态库
#   make l3-test    — 编译库 + 运行所有 L3 运行测试
#   make l4-test    — 运行全部 L4 E2E 集成测试（需要已初始化子模块）
#   make submodules — 初始化/更新所有 E2E 测试子模块
#   make dump-ast DIR=examples/006_class_basic/cpp — 转储某示例的精简 AST（本地可追溯，产物不入库）

.PHONY: l3-setup l3-test l4-test submodules dump-ast

## 编译所有 L3 测试所需的 C++ 动态库
l3-setup:
	bash scripts/build_cpp_libs.sh

## 编译库 + 运行所有 L3 运行测试（单线程，避免动态库加载冲突）
l3-test: l3-setup
	cargo test --test l3_run_tests --features full-test -- --test-threads=1

## 初始化 E2E 测试所需的外部库子模块
submodules:
	git submodule update --init \
		references/tinyxml2 \
		references/pugixml \
		references/nlohmann-json \
		references/fmtlib \
		references/magic_enum \
		references/tomlplusplus

## 运行全部 L4 E2E 集成测试（包括 rapidjson + 七个真实项目）
l4-test: submodules
	cargo test --test rapidjson_e2e_test
	cargo test --test tinyxml2_e2e_test
	cargo test --test pugixml_e2e_test
	cargo test --test sqlite3_e2e_test
	cargo test --test nlohmann_json_e2e_test
	cargo test --test fmtlib_e2e_test
	cargo test --test magic_enum_e2e_test -- --test-threads=1
	cargo test --test tomlplusplus_e2e_test -- --test-threads=1

## 转储某示例的「AST → hicc」可追溯产物（宏展开 .i + 完整 ast.json + 过滤后的 user-ast.json）
## 产物写入 <DIR>/../ast/（已被 .gitignore 忽略，绝不入库）。DIR 默认 006_class_basic。
dump-ast:
	bash scripts/dump_ast.sh $(or $(DIR),examples/006_class_basic/cpp)
