# `ferrotex-math-semantics`

A specialized domain engine for verifying the structural integrity of LaTeX mathematical environments.

## 📐 The Problem

Standard LaTeX compilers often provide cryptic errors for structural math mistakes (e.g., "Extra alignment tab has been changed to \cr"). `ferrotex-math-semantics` solves this by performing a semantic pre-audit of the math DOM.

## 💎 Core Verification

### Jagged Matrix Detection

Intelligently counts alignment tabs (`&`) and row breaks (`\\`) to detect inconsistent dimensions in environments such as:

- `matrix`, `pmatrix`, `bmatrix`, etc.
- `aligned`, `gather`, `cases`.

### Advanced Awareness

- **`\multicolumn` Awareness**: Correctly calculates cell spans to avoid false-positive diagnostics in complex tables.
- **Nested Group Protection**: Accurately tracks delimiters to ensure nested math environments are handled atomically.

## 🚀 Integration

This crate provides the logic used by `ferrotex-analysis` to populate the `check_math` diagnostic pass in the Language Server.

---

_Part of the FerroTeX Scientific Compiler Platform._
