---
title: "crates/ferrotex-math-semantics/src/analysis.rs"
type: source-code
language: rust
crate: ferrotex-math-semantics
layer: math
tags:
  - crate/ferrotex-math-semantics
  - type/rust
  - layer/math
summary: "Math semantic analyzer providing matrix dimension inference, shape consistency validation, multicolumn handling, and environment structural taxonomy in FerroTeX."
---

# `crates/ferrotex-math-semantics/src/analysis.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-math-semantics` (math)
> - **Language/Type**: `rust` (source-code)
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]
> - **Module Role**: Math Semantic Analysis & Dimensional Inference

## Purpose & Architecture Role

`crates/ferrotex-math-semantics/src/analysis.rs` implements the core math semantic analysis algorithms for FerroTeX. While standard LaTeX rendering engines treat mathematical notation purely as visual glyph layouts, FerroTeX evaluates mathematical environments through a structured semantic lens to identify structural, dimensional, and syntactic errors during document authoring.

Within the **math** layer of FerroTeX:
- **Structural Pre-Audit**: Operates over Rowan-based concrete syntax trees (`ferrotex-syntax`) without requiring full LaTeX macro expansion or execution.
- **Matrix Dimension Inference**: Traverses mathematical alignment blocks to compute exact row and column dimensions ($M \times N$), validating consistency across rows to identify jagged matrices.
- **Multicolumn Resolution**: Accounts for spanning cell constructs (`\multicolumn{n}{...}{...}`), dynamically adjusting row column counts by $n - 1$.
- **Protected Group Isolation**: Shields nested groups (`{...}`) so internal alignment characters (`&`) do not corrupt outer matrix structural calculations.
- **Diagnostic Feeder**: Supplies inferred shapes to [[crates/ferrotexd/src/diagnostics/math.rs|diagnostics math]] in the LSP daemon for real-time diagnostic reporting.

---

## Key Symbols & Interfaces

### Functions

| Identifier | Signature | Visibility | Description |
|---|---|---|---|
| `infer_shape` | `pub fn infer_shape(node: &SyntaxNode) -> Shape` | `pub` | Analyzes a mathematical syntax node (typically an `Environment`) to determine its structural shape (`Matrix`, `Invalid`, or `Unknown`). |

### Algorithm Breakdown: `infer_shape`

1. **Token Stream Scanning**:
   - Iterates through `node.children_with_tokens().peekable()`.
   - **Text Tokens (`SyntaxKind::Text`)**: Evaluates individual characters. Each unescaped ampersand `&` increments the current row's column counter (`current_row_cols += 1`).
   - **Row Break Commands (`\\\\`)**: Appends `current_row_cols + 1` to the row dimension buffer and resets `current_row_cols` to zero.
2. **Multicolumn Parsing (`\multicolumn`)**:
   - Detects `\multicolumn` command tokens.
   - Skips whitespace and comment trivia to locate the first `SyntaxKind::Group`.
   - Parses the group's inner text as an integer column count $n$.
   - If $n > 1$, increases `current_row_cols += n - 1`.
   - Skips the subsequent two argument groups (`{alignment}` and `{cell content}`).
3. **Nested Group Shielding**:
   - Sub-nodes of kind `SyntaxElement::Node(Group)` are ignored at the top-level traversal, preventing inner ampersands from affecting outer row column counts.
4. **Terminal Row Finalization**:
   - Pushes the final row count (`current_row_cols + 1`) into the `rows` list.
5. **Rectangular Consistency Validation**:
   - Compares every row's column count against row 1 (`first_row_cols`).
   - If any row deviates, constructs a `Shape::Invalid(format!("Jagged matrix: row 1 has {} columns, but row {} has {}", ...))` error message.
   - Returns `Shape::Matrix { rows: Dimension::Finite(rows.len()), cols: Dimension::Finite(first_row_cols) }`.

---

## Math Environment Taxonomy & Semantic Validation

The math verification engine analyzes environments according to their structural and layout semantics:

### 1. Matrix & Grid Environments
Environments whose cells form a rigid 2D grid where all rows must have identical column dimensions:
- `matrix`: Plain unbracketed matrix.
- `pmatrix`: Parenthesized matrix `(...)`.
- `bmatrix`: Square-bracketed matrix `[...]`.
- `vmatrix`: Single-vertical bar determinant `|...|`.
- `Vmatrix`: Double-vertical bar norm `||...||`.
- `smallmatrix`: Compact inline matrix layout.

### 2. Multi-line Equation & Alignment Environments
Environments designed for multi-step equations or split relations:
- `align` / `aligned` / `align*`: Multi-line alignment where columns alternate between relation alignment points (`&`) and spacing.
- `gather` / `gathered` / `gather*`: Multi-line equation block with centered lines and no intra-line column alignment (`&` typically invalid).
- `multline` / `multline*`: Single long equation split across multiple lines without alignment markers.
- `cases` / `dcases`: Piecewise conditions where rows typically expect exactly 2 columns (`expression & condition`).
- `eqnarray` / `eqnarray*`: Legacy 3-column alignment (`left & relation & right`).

### 3. Syntax & Semantic Validation Spectrum
- **Mismatched Delimiters**: Coordinated with [[crates/ferrotex-math-semantics/src/delimiters.rs|delimiters.rs]], detecting unclosed `(`, `[`, `{`, and unbalanced `\left` / `\right` directives.
- **Unescaped Operators & Alignment Markers**: Detects stray alignment markers (`&`) in environments that forbid them (e.g., `gather` or `multline`) or unescaped macro operators.
- **Invalid Superscripts & Subscripts**: Flags misplaced or consecutive `^` and `_` operators that lack enclosing grouping.
- **Dimensional Homogeneity**: In conjunction with [[docs/spec/math-verification.md|Math Verification Spec]], verifies compatibility between dimensional terms (e.g., vector addition vs matrix-vector multiplication).

---

## Dependencies

- [[crates/ferrotex-math-semantics/src/lib.rs|math lib.rs]] (`Dimension`, `Shape`)
- [[crates/ferrotex-math-semantics/src/delimiters.rs|delimiters.rs]] (`check_delimiters`, `DelimiterError`)
- [[crates/ferrotex-syntax/src/lib.rs|ferrotex-syntax]] (`SyntaxElement`, `SyntaxKind`, `SyntaxNode`)

---

## Consumers & Dependents

- [[crates/ferrotexd/src/diagnostics/math.rs|diagnostics math]] — Consumes `infer_shape` to publish `math-semantics` LSP diagnostics (FMT-002 Jagged Matrix Structure) for invalid alignments.
- [[crates/ferrotex-math-semantics/src/lib.rs|math lib.rs]] — Exports `pub mod analysis;`.

---

## Related Specs & ADRs

- [[docs/spec/math-verification.md|Math Verification Spec]] — Specifies the Math Intermediate Representation (Math IR), dimension checking, and error codes (`FMT-001`, `FMT-002`).
- [[docs/spec/lsp.md|LSP Spec]] — Language server diagnostics pipeline and document synchronization.
- [[docs/spec/incremental-analysis.md|Incremental Analysis Spec]] — Architecture for fast, incremental updates to mathematical syntax trees.
