# `ferrotex-analysis`

The semantic intelligence engine of FerroTeX. It bridges the gap between raw syntax trees and high-level logical diagnostics.

## 🧠 Responsibilities

This crate is responsible for analyzing the _meaning_ of the LaTeX source, rather than just its structure.

### Key Analysis Passes

- **Reference Resolution**: Resolves labels, citations, and section identifiers across multi-file workspaces.
- **Workspace Indexing**: Maintains a live, incremental index of symbols and project structure.
- **Diagnostic Synthesis**: Transforms raw parser errors and semantic violations into structured, actionable diagnostics.

## 📐 Semantic Math

Integrates with `ferrotex-math-semantics` to perform deep structural audits of math environments, ensuring consistency in matrices and alignment blocks.

## 🚀 Engine Integration

Acts as the primary data provider for the `ferrotexd` Language Server, supplying the metadata needed for:

- Context-aware completion.
- Hover tooltips.
- Global symbol search.

---

_Part of the FerroTeX Scientific Compiler Platform._
