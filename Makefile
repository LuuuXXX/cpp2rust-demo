# Makefile — cpp2rust-demo 开发者便捷入口
#
# 常用目标：
#   make test-l1      运行 L1 黄金文件测试
#   make test-l2      运行 L2 单元测试（包含 merger 模板合并测试）
#   make test-l3      一键编译 C++ 共享库并运行 L3 集成测试
#   make test-l4      运行 L4 RapidJSON E2E 测试（需要 libclang）
#   make test-all     运行 L1 + L2（不含 ignored 测试）
#   make test-e2e     运行扩展 E2E 测试（需要 libfmt/libabsl/libeigen3）
#   make clean        清理编译产物

.PHONY: test-l1 test-l2 test-l3 test-l4 test-all test-e2e clean help

# 默认目标
help:
	@echo "cpp2rust-demo 开发者命令"
	@echo ""
	@echo "  make test-l1     L1 黄金文件测试"
	@echo "  make test-l2     L2 单元测试（lib 测试，含模板合并）"
	@echo "  make test-l3     L3 集成测试（一键编译共享库 + 运行）"
	@echo "  make test-l4     L4 RapidJSON E2E 测试（需要 libclang）"
	@echo "  make test-all    L1 + L2（常规开发验证）"
	@echo "  make test-e2e    扩展 E2E（需要 libfmt/libabsl/libeigen3）"
	@echo "  make clean       清理编译产物"
	@echo ""
	@echo "提示：L3 会自动编译 examples/*/cpp/ 下的共享库。"
	@echo "      使用 FILTER=001 缩小范围：make test-l3 FILTER=001"

# L1 黄金文件测试
test-l1:
	cargo test --test l1_golden_tests

# L2 单元测试（lib 内部单元测试 + integration tests 不含 ignored）
test-l2:
	cargo test --lib

# L3 集成测试（先编译共享库，再运行）
# 支持 FILTER 参数：make test-l3 FILTER=001
FILTER ?=
THREADS ?= 4

test-l3:
ifdef FILTER
	./scripts/run_l3_local.sh --filter "$(FILTER)" --threads $(THREADS)
else
	./scripts/run_l3_local.sh --threads $(THREADS)
endif

# L4 RapidJSON E2E 测试
test-l4:
	cargo test --test rapidjson_e2e_test -- --ignored --nocapture

# 日常完整测试（L1 + L2，快速反馈）
test-all: test-l1 test-l2
	@echo ""
	@echo "✓ L1 + L2 全部通过"

# 扩展 E2E 测试（需要系统安装对应库）
test-e2e:
	cargo test --test fmt_e2e_test     -- --ignored --nocapture
	cargo test --test abseil_e2e_test  -- --ignored --nocapture
	cargo test --test eigen_e2e_test   -- --ignored --nocapture

# 清理共享库编译产物
clean:
	./scripts/run_l3_local.sh --clean
	cargo clean
