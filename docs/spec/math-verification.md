---
layout: default
title: Math Verification
parent: Specifications
nav_order: 26
---

# Math Verification Specification

## Status

- **Type:** Normative
- **Stability:** Research / Exploratory (v0.20.0 Target)

## Purpose

The Math Verification pillar in FerroTeX provides deep semantic analysis of LaTeX mathematical environments. While standard LaTeX compilers treat math as a series of glyphs for layout, FerroTeX treats math as a structured expression that can be symbolically executed to detect dimensional, structural, and logical inconsistencies.

## Symbolic Execution Engine

FerroTeX transforms LaTeX math environments into a **Math Intermediate Representation (Math IR)**. This IR is then processed by a symbolic execution engine to validate the following properties:

### 1. Structural Consistency

- **Matrix Dimensions**: Validates that matrix operations (addition, multiplication, etc.) are performed on compatible dimensions.
- **Jagged Arrays**: Detects mismatched row/column counts in `matrix`, `align`, `gather`, and `cases` environments.
- **Nested Delimiters**: Ensures that parenthesis, brackets, and braces are balanced across multi-line environments.

### 2. Dimensional Analysis

FerroTeX can perform lightweight dimensional tracking for physical quantities:

- **Unit Propagation**: Infers units from explicit `\unit{}` or `\si{}` commands (using `siunitx` patterns).
- **Dimensional Homogeneity**: Ensures that equations like $A = B + C$ have consistent dimensions across all terms.
- **Scale Mismatches**: Flags potential errors when adding quantities with incompatible units (e.g., meters and seconds).

### 3. Variable Tracking & Type Inference

- **Scope Analysis**: Tracks variable definitions and usage within a document or across multiple files.
- **Type Checking**: Infers whether a symbol represents a scalar, vector, matrix, or tensor based on its context and decoration (e.g., `\vec`, `\mathbf`).
- **Undefined Symbols**: Diagnostics for symbols used in equations that have not been defined in the surrounding text or preamble.

## Detection Algorithms

### Matrix Dimension Validation

When a matrix multiplication $C = AB$ is detected in the Math IR:

1. Extract dimensions $dim(A) = (m, n)$ and $dim(B) = (p, q)$.
2. Verify $n = p$.
3. If $n \neq p$, emit a `FMT-001: Incompatible Matrix Dimensions` diagnostic.

### Structural Pre-Audit

The `ferrotex-math-semantics` engine performs a non-evaluative pass to:

- Count `&` and `\\` tokens while respecting `\multicolumn` and nested groups.
- Compare counts against the expected structure for the environment type.
- Emit `FMT-002: Jagged Matrix Structure` if rows have inconsistent cell counts.

## Diagnostic Levels

| Code    | Name                           | Level   | Description                                                  |
| :------ | :----------------------------- | :------ | :----------------------------------------------------------- |
| FMT-001 | Incompatible Matrix Dimensions | Error   | Matrix multiplication or addition with mismatched sizes.     |
| FMT-002 | Jagged Matrix Structure        | Warning | Inconsistent number of columns across rows in an alignment.  |
| FMT-003 | Dimensional Inconsistency      | Error   | Adding or equating terms with different physical dimensions. |
| FMT-004 | Undefined Math Symbol          | Info    | Use of a symbol that does not appear in the symbol index.    |

## Future Work: Range Analysis

Future versions of the engine will support **Range Analysis**, checking for potential division-by-zero or out-of-bounds errors in symbolic expressions where variable constraints are provided (e.g., via comments or metadata).
