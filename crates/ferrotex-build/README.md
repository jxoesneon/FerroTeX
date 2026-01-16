# `ferrotex-build`

The orchestration layer for reliable, reproducible, and observable LaTeX builds.

## 🏗️ The Build DAG

`ferrotex-build` models the LaTeX build process as a Directed Acyclic Graph (DAG) of **Artifacts** and **Transforms**.

### Core Concepts

- **Artifact**: A versioned primitive of the build (e.g., `.tex` source, `.pdf` output, `.bib` database).
- **Transform**: A function that turns a set of input Artifacts into outputs (e.g., executing Tectonic, running BibTeX).
- **Build Graph**: A topological sort of transforms that ensures optimal execution and cycle detection.

## 💎 Scientific Reproducibility

This crate implements the **Content-Addressable** storage model to guarantee hermeticity.

- **`ferrotex.lock`**: A SHA256-based lockfile that records the exact state of all inputs.
- **Fingerprinting**: Every artifact is fingerprinted, allowing the system to skip redundant transforms and detect environmental drift.

## 🚀 Key Traits

- `Artifact`: Interface for anything that participates in the build graph.
- `Transform`: Logic for the actual compilation steps.
- `Compiler`: High-level driver that executes the DAG.

---

_Part of the FerroTeX Scientific Compiler Platform._
