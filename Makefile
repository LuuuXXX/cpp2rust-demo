# cpp2rust-demo 快捷目标
#
# 使用方式：
#   make l3-setup   — 编译所有 L3 测试所需的 C++ 动态库
#   make l3-test    — 编译库 + 运行所有 L3 运行测试
#   make l4-test    — 运行全部 L4 E2E 集成测试（需要已初始化子模块）
#   make submodules — 初始化/更新所有 E2E 测试子模块

.PHONY: l3-setup l3-test l4-test submodules

## 编译所有 L3 测试所需的 C++ 动态库
l3-setup:
	bash scripts/build_cpp_libs.sh

## 编译库 + 运行所有 L3 运行测试（单线程，避免动态库加载冲突）
l3-test: l3-setup
	cargo test --test l3_run_tests -- --include-ignored --test-threads=1

## 初始化 E2E 测试所需的外部库子模块
submodules:
	git submodule update --init \
		references/tinyxml2 \
		references/pugixml \
		references/nlohmann-json \
		references/fmtlib

## 运行全部 L4 E2E 集成测试（包括 rapidjson + 五个新项目）
l4-test: submodules
	cargo test --test rapidjson_e2e_test
	cargo test --test tinyxml2_e2e_test
	cargo test --test pugixml_e2e_test
	cargo test --test sqlite3_e2e_test
	cargo test --test nlohmann_json_e2e_test
	cargo test --test fmtlib_e2e_test
