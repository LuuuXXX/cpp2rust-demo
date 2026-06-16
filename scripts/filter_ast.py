#!/usr/bin/env python3
"""把 clang ast-dump 的 JSON 过滤为「用户自有」声明（即项目自己的 .h/.cpp 中的节点）。

带「父文件上下文」遍历 AST：节点的 location 在嵌套于已带 file 的父声明时
可能省略 `file` 字段，故沿树向下传播当前文件路径。当前文件以用户头文件或
实现文件 basename 结尾的节点会被保留。

用法: python3 filter_ast.py <ast.json> <header_basename> <cpp_basename> <out.json>
"""
import json
import sys
from pathlib import Path


def resolve_file(node, parent_file):
    loc = node.get("loc") if isinstance(node, dict) else None
    if isinstance(loc, dict):
        f = loc.get("file")
        if f:
            return f
    return parent_file  # 继承父节点文件


def short_loc(node):
    loc = node.get("loc") or node.get("range", {}).get("begin", {})
    if isinstance(loc, dict):
        f = loc.get("file") or loc.get("includedFile") or ""
        line = loc.get("line", "?")
        col = loc.get("col", "?")
        return f"{Path(f).name}:{line}:{col}" if f else f"?:{line}:{col}"
    return "<unknown>"


KEY_KINDS = {
    "FunctionDecl", "CXXRecordDecl", "CXXMethodDecl", "ClassTemplateDecl",
    "ClassTemplateSpecializationDecl", "FunctionTemplateDecl", "VarDecl",
    "FieldDecl", "EnumDecl", "EnumConstantDecl", "NamespaceDecl",
    "CXXConstructorDecl", "CXXDestructorDecl", "TypedefDecl", "RecordDecl",
    "TypeAliasDecl", "UsingDirectiveDecl",
}


def walk(node, out, parent_file, header_bn, cpp_bn):
    if isinstance(node, dict):
        cur_file = resolve_file(node, parent_file)
        in_user = cur_file and (
            Path(cur_file).name == header_bn or Path(cur_file).name == cpp_bn
        )
        if in_user and node.get("kind") in KEY_KINDS:
            entry = {
                "kind": node["kind"],
                "name": node.get("name") or node.get("declKind") or "",
                "file": Path(cur_file).name,
                "loc": short_loc(node),
            }
            if node["kind"] in (
                "CXXMethodDecl", "FunctionDecl",
                "CXXConstructorDecl", "CXXDestructorDecl",
                "FunctionTemplateDecl",
            ):
                entry["type"] = node.get("type", {}).get("qualType", "")
            if node["kind"] in ("FieldDecl", "VarDecl"):
                entry["type"] = node.get("type", {}).get("qualType", "")
            if node["kind"] in (
                "CXXRecordDecl", "RecordDecl",
                "ClassTemplateDecl", "ClassTemplateSpecializationDecl",
            ):
                entry["tagUsed"] = node.get("tagUsed", "")
            out.append(entry)
        for v in node.values():
            walk(v, out, cur_file, header_bn, cpp_bn)
    elif isinstance(node, list):
        for v in node:
            walk(v, out, parent_file, header_bn, cpp_bn)


def main():
    ast_path = sys.argv[1]
    header_bn = sys.argv[2]
    cpp_bn = sys.argv[3]
    out_path = sys.argv[4]
    with open(ast_path) as f:
        d = json.load(f)
    out = []
    walk(d, out, "", header_bn, cpp_bn)
    seen = set()
    uniq = []
    for e in out:
        key = (e["kind"], e["name"], e["loc"])
        if key in seen:
            continue
        seen.add(key)
        uniq.append(e)
    with open(out_path, "w") as f:
        json.dump({"header": header_bn, "count": len(uniq), "decls": uniq}, f, indent=2)
    print(f"[filter_ast] {header_bn}: {len(uniq)} user decls → {out_path}")


if __name__ == "__main__":
    main()
