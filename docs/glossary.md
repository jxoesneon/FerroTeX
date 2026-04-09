---
layout: default
title: Glossary
nav_order: 8
---

# Glossary

## Terms

### TeX / LaTeX

The TeX typesetting system and the LaTeX macro package ecosystem.

### Engine

A TeX executable such as pdfTeX, XeTeX, or LuaTeX.

### Log

The textual stream emitted by the engine (and/or auxiliary build tools). Typically materialized as a `.log` file.

### Event IR

FerroTeX’s typed intermediate representation of log semantics (file transitions, diagnostics, warnings, etc.) with provenance spans.

### Provenance

The ability to trace an emitted event/diagnostic back to an exact byte span in the source log buffer.

### File Context Stack

The reconstructed nesting of file inputs inferred from the log’s parenthesis structure.

### Confidence

A numeric score in `[0, 1]` indicating how strongly FerroTeX believes a mapping or event interpretation is correct.

### LSP

Language Server Protocol. FerroTeX uses LSP to publish diagnostics and editor features.

### DAP

Debug Adapter Protocol. FerroTeX may use DAP for exploratory “debugging” semantics for TeX.

### Concrete Syntax Tree (CST)

A lossless, fault-tolerant tree representation of the source code. Unlike an AST, it preserves all tokens, including whitespace and comments, ensuring that formatting is maintained during refactoring.

### Abstract Machine

A specialized engine (`ferrotex-analysis`) that performs abstract interpretation of TeX macros. It models the "stomach" of TeX to predict macro expansion behavior without requiring a full engine run.

### Virtual File System (VFS)

An abstraction layer that intercepts file I/O to isolate the TeX engine within the project root, preventing unauthorized filesystem access.

### Capabilities-Based Security

A security model where permissions (e.g., read, write, execute) are explicitly granted as "capabilities" to a build context, rather than being determined by the user's global system permissions.

### Incremental Analysis

The process of re-analyzing only the portions of a document affected by a specific change, enabling keystroke-level feedback by minimizing re-computation.

### Time-Travel Debugging

The ability to step backward through the macro expansion process. Enabled by snapshotting the state of the Abstract Machine at every expansion step.

### Symbolic Execution

A method of analyzing mathematical environments by treating variables as symbols rather than concrete values, allowing for the detection of structural inconsistencies like matrix dimension mismatches.

### Deprecated Command Diagnostic

A `warning`-severity LSP diagnostic emitted by `ferrotexd` when it detects a LaTeX 2.09 font command (`\bf`, `\it`, `\rm`, etc.), obsolete display math syntax (`$$...$$`), or a known-obsolete package (`times`, `a4wide`). Each diagnostic is paired with a `quickfix` code action that performs the mechanical replacement (e.g., `{\bf text}` → `\textbf{text}`).
