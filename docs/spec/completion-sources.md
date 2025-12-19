# Completion Data Sources

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

High-quality completion requires authoritative sources for:

- commands
- environments
- packages
- labels/citations

This document defines how FerroTeX obtains these sources.

## Sources

### S1: Built-in Inventory (Required)

FerroTeX MUST ship a baseline inventory of:

- common LaTeX commands
- common environments

This inventory is versioned and deterministic.

### S2: Workspace Discovery (Required)

FerroTeX MUST discover a safe subset of local definitions:

- `\newcommand`, `\renewcommand` (best-effort)
- `\newenvironment` (best-effort)

Discovery MUST be confidence-gated and should avoid false positives.

### S3: Package/CTAN Metadata (Optional pre-1.0, allowed by v1.0.0)

FerroTeX MAY support a package metadata database for completions.

If included:

- it MUST be bundled (no network required at runtime)
- it MUST be versioned

### S4: Engine-assisted explanation (Optional)

Engine-assisted introspection (LuaTeX) is out of scope for correctness-critical completion.

## Ranking

Ranking should combine:

- local definitions
- widely used commands
- project/package context

## Non-Goals

- Perfect completion for arbitrary packages with dynamic macros.
