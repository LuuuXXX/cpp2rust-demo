# Clang AST JSON Parsing

This document explains how `cpp2rust-demo` uses the clang AST JSON output to
extract C++ declarations.

## Generating the AST JSON

The tool runs the following command internally for each input header:

```bash
clang -Xclang -ast-dump=json -fsyntax-only -x c++ -std=c++14 [extra-args] myheader.hpp
```

The JSON output goes to stdout and is parsed in memory. A copy is saved to
`.cpp2rust/<feature>/ast/<header>.ast.json` for debugging.

## Key AST Node Types

### `TranslationUnitDecl`

The root node. Contains all top-level declarations.

### `NamespaceDecl`

A C++ namespace. The tool recurses into it, tracking the namespace path.

```json
{
  "kind": "NamespaceDecl",
  "name": "mylib",
  "inner": [ /* declarations */ ]
}
```

### `FunctionDecl`

A free C++ function declaration.

```json
{
  "kind": "FunctionDecl",
  "name": "add",
  "type": { "qualType": "int (int, int)" },
  "inner": [
    { "kind": "ParmVarDecl", "name": "a", "type": { "qualType": "int" } },
    { "kind": "ParmVarDecl", "name": "b", "type": { "qualType": "int" } }
  ]
}
```

Key fields:
- `name` – function name
- `type.qualType` – function type in the form `"ReturnType (ParamTypes...)"` or
  `"ReturnType (ParamTypes...) const"` for const methods
- `inner` – `ParmVarDecl` children for parameters
- `isImplicit` – if `true`, skip (compiler-generated)
- `storageClass` – `"static"` for static functions/methods

### `CXXRecordDecl`

A C++ class or struct declaration.

```json
{
  "kind": "CXXRecordDecl",
  "name": "Widget",
  "tagUsed": "class",
  "completeDefinition": true,
  "inner": [
    { "kind": "AccessSpecDecl", "access": "public" },
    { "kind": "CXXMethodDecl", "name": "update", ... },
    { "kind": "CXXConstructorDecl", "name": "Widget", ... },
    { "kind": "CXXDestructorDecl", "name": "~Widget", ... }
  ]
}
```

Key fields:
- `completeDefinition` – `true` only for full definitions (not forward declarations)
- `tagUsed` – `"class"` or `"struct"` (affects default access)
- `inner` – contains `AccessSpecDecl`, `CXXMethodDecl`, etc.

### `CXXMethodDecl`

A C++ class method.

```json
{
  "kind": "CXXMethodDecl",
  "name": "getId",
  "type": { "qualType": "int () const" },
  "storageClass": null
}
```

- `isVirtual` – if `true`, this is a virtual method
- `isPure` – if `true`, this is a pure-virtual method
- `storageClass: "static"` – static method

### `CXXConstructorDecl` / `CXXDestructorDecl`

Constructors and destructors. `cpp2rust-demo` skips these by default because
they require special handling in hicc (factory functions or `make_unique`).

### `AccessSpecDecl`

Marks an access level change within a class:

```json
{ "kind": "AccessSpecDecl", "access": "public" }
```

The tool tracks the current access level and only extracts `public` members.

### `ParmVarDecl`

A function/method parameter:

```json
{ "kind": "ParmVarDecl", "name": "value", "type": { "qualType": "int" } }
```

## Location Tracking (`loc`)

Clang AST JSON uses an *incremental* location format: the `loc.file` field is
only present when the file changes. The tool tracks the current file as it
traverses the tree:

```json
// First node from the target header: loc has `file`
{
  "kind": "NamespaceDecl",
  "loc": { "file": "/path/to/mylib.hpp", "line": 2 },
  ...
}
// Subsequent nodes in the same file: loc has no `file`
{
  "kind": "FunctionDecl",
  "loc": { "line": 5, "col": 4 },
  ...
}
```

The tool filters declarations to only keep those where the currently tracked
file matches one of the input headers. This eliminates system header nodes.

## Function Type Parsing

The `type.qualType` for a function is:

```
"ReturnType (ParamType1, ParamType2, ...) [const]"
```

The tool splits on the first ` (` to separate the return type:

```rust
fn parse_fn_qual_type(qual_type: &str) -> Option<(String, String)> {
    let sep = qual_type.find(" (")?;
    let return_type = qual_type[..sep].trim().to_string();
    let after_open = &qual_type[sep + 2..];
    let close = after_open.find(')')?;
    let params_str = after_open[..close].trim().to_string();
    Some((return_type, params_str))
}
```

> **Note**: The parameter types in `qualType` are used for the C++ signature in
> `#[cpp(func = "...")]`. Individual parameter NAMES come from the `ParmVarDecl`
> children.

## Debugging

To inspect the raw AST JSON yourself:

```bash
clang -Xclang -ast-dump=json -fsyntax-only -x c++ -std=c++14 myheader.hpp \
  | python3 -m json.tool | less
```

Or use the saved copy:

```bash
cat .cpp2rust/default/ast/myheader.ast.json | python3 -m json.tool | less
```
