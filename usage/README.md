# usage — 本地验证脚本

本目录包含对 tinyxml2 项目进行 cpp2rust-demo FFI 转换的本地验证脚本：

| 文件 | 说明 |
|------|------|
| [`verify-tinyxml2-ffi.sh`](verify-tinyxml2-ffi.sh) | 全自动 Shell 脚本（CLI 方式，适合批量/CI 场景） |

---

## 快速开始

```bash
# 系统依赖（Ubuntu/Debian）
sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 初始化 tinyxml2 子模块
git submodule update --init references/tinyxml2

# 运行验证脚本
bash usage/verify-tinyxml2-ffi.sh
```

---

## 脚本说明（verify-tinyxml2-ffi.sh）

### 可配置环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `TINYXML2_DIR` | 仓库内 `references/tinyxml2` | tinyxml2 源码目录 |
| `FEATURE` | `tinyxml2_ffi` | cpp2rust-demo feature 名称 |
| `SKIP_INSTALL` | `0` | 置 `1` 跳过 `cargo install` |

### 脚本执行阶段

1. 环境检查 & 依赖安装
2. 安装 cpp2rust-demo（`cargo install --git ...`）
3. 确认 tinyxml2 子模块已初始化
4. `cpp2rust-demo init` — 编译拦截 + FFI 脚手架生成
5. `cpp2rust-demo merge` — 整理输出目录
6. `cargo check` — 验证生成项目可编译
7. 结果汇报（捕获文件数 / 生成 .rs 文件数 / 降级标记统计）

---

## 常见问题

**Q: 脚本提示"未找到命令：cpp2rust-demo"**

```bash
cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked
export PATH="$HOME/.cargo/bin:$PATH"
```

**Q: 子模块未初始化**

```bash
git submodule update --init references/tinyxml2
```
