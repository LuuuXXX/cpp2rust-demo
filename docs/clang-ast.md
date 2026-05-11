# clang AST 处理说明

`cpp2rust-demo` 会对选中的 `*2rust` 中间件执行：

```bash
clang -Xclang -ast-dump=json -fsyntax-only -x c++ -std=c++14 <file>2rust
```

并将原始 AST JSON 保存到：

```text
.cpp2rust/<feature>/ast/<stem>.ast.json
```

## 提取内容

- 自由函数（`FunctionDecl`）
- 类与方法（`CXXRecordDecl` / `CXXMethodDecl`）
- 参数与返回类型（用于生成 hicc 签名）
- 命名空间限定名

## 目的

AST 提取结果用于自动生成：

- `hicc::import_lib!` 函数声明
- `hicc::import_class!` 类方法声明
- `init-interface-report.md` 接口清单
