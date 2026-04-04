# Incremental Analysis Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

To achieve "Incremental Everything," FerroTeX must move beyond per-file analysis. This specification defines a reactive architecture that ensures diagnostic feedback and semantic intelligence respond within the human-perception threshold (e.g., 50ms) even in large projects.

## Core Architecture: The Reactive Dependency Graph

FerroTeX uses a **reactive dependency graph** (inspired by the **Salsa** library) to manage state. This graph tracks relationships between source inputs, intermediate results (CST, AST, indices), and final outputs (diagnostics, completion items).

### Key Concepts

1. **Inputs**: Leaf nodes representing source file contents or configuration.
2. **Queries**: Functions that transform one or more nodes into a derived result.
3. **Memoization**: Results of queries are cached. If inputs have not changed, the cached result is returned instantly.
4. **Invalidation**: When a source file changes, only the affected nodes in the dependency graph are marked as "dirty."

### Benefits for LaTeX

- **Partial Reparsing**: Only the modified section of a file (e.g., a specific environment or group) is re-parsed.
- **Cross-File Propagation**: Changes to a `\label` in one file only trigger re-analysis of `\ref` nodes that specifically depend on that label name.
- **Lazy Execution**: Analysis is only performed for what is currently visible or requested by the user.

## The Incremental Abstract Machine

The **Abstract Machine (AM)** is responsible for analyzing TeX macro behavior and register states. Unlike traditional engines that run linearly, the FerroTeX AM is **persistent and incremental**.

### Persistent State Maintenance

The AM maintains a snapshot of the state (registers, input stack, expansion depth) at strategic "checkpoints" throughout the document (e.g., at the start of every top-level environment).

- **Checkpointing**: When a change occurs, the AM restores the state from the nearest preceding checkpoint rather than starting from the beginning of the project.
- **State Hashing**: Each checkpoint's state is hashed. If a change in a preceding section results in the same abstract state at a checkpoint, the subsequent graph remains valid and re-analysis is pruned.

### Abstract Values

The AM operates on **Abstract Values** rather than concrete strings/tokens when possible to reduce the sensitivity of the dependency graph to trivial changes (like whitespace inside a register assignment).

## Keystroke-Level Feedback Loop

The goal is to provide **diagnostics on every keystroke**.

1. **LSP `didChange`**: The editor sends a notification with a text delta.
2. **Incremental CST Update**: The fault-tolerant parser updates the CST in-place.
3. **Graph Invalidation**: The reactive engine marks the `parse_file` query as dirty.
4. **Background Analysis**: The engine kicks off a query for `diagnostics`.
5. **Pruned Execution**: Salsa ensures that only the minimal path through the graph is re-evaluated.
6. **LSP `publishDiagnostics`**: Updated diagnostics are sent back to the editor.

## Implementation Details

- **Concurrency**: Queries can be executed in parallel where dependencies allow.
- **Cancellation**: If a new keystroke arrives before the previous analysis completes, the ongoing query is immediately cancelled.
- **Memory Management**: The graph uses an LRU (Least Recently Used) policy for memoized results to prevent unbounded memory growth in massive projects.
