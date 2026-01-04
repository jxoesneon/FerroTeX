# `ferrotex-core`

The shared infrastructure and common type definitions used across the entire FerroTeX workspace.

## 🏗️ Role

`ferrotex-core` serves as the "standard library" for the project, preventing circular dependencies by centralizing ubiquitous traits and data structures.

### Key Components

- **Common Traits**: Defines core interfaces like `Transform` and `Artifact` (re-exported by `ferrotex-build` but grounded here).
- **Error Handling**: Centralized error types and result aliases for consistent failure reporting across crates.
- **Logging & Spans**: Integration with the `tracing` ecosystem to provide detailed execution telemetry.
- **Project Model**: Shared definitions of workspace structures, file paths, and URI resolution.

## 💎 Design Goals

- **Minimal Dependencies**: Kept lightweight to ensure fast compile times for downstream crates.
- **Stability**: Provides the stable API surface that links the parser, build system, and LSP.

---

_Part of the FerroTeX Scientific Compiler Platform._
