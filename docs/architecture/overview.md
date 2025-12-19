# Architecture Overview

## Purpose

FerroTeX is a **LaTeX language platform** and structured observability layer for the TeX toolchain.

Its core function is to provide two complementary pipelines:

- **Source analysis**: parse LaTeX source into a fault-tolerant CST/AST, build a workspace index, and serve industry-standard IDE features.
- **Build observability**: transform unstructured engine output into a typed event stream and deterministic diagnostics with provenance.

This document is normative at the level of **component responsibilities** and **system invariants**.

## System Artifacts

- **Rust server** (`ferrotexd`)
  - source ingestion (LSP text sync + filesystem)
  - source parsing (incremental, error-tolerant)
  - project model (workspace roots, include graph)
  - symbol and reference indexing
  - log ingestion (streaming and offline)
  - log parsing + recovery
  - build context reconstruction
  - diagnostics mapping (source + build)
  - LSP transport and request handling
  - engine invocation (adapter boundary)

- **VS Code extension**
  - workspace configuration and server lifecycle
  - LSP client wiring
  - UX rendering (diagnostics, commands, logs)
  - minimal policy; logic should live in the Rust server

## Core Invariants

- **I1: Provenance-preserving parsing**
  - Every emitted build/log event MUST carry a byte-span into the source log buffer.
  - Source diagnostics SHOULD carry provenance to a source range and, where applicable, a CST node identity.

- **I2: Typed, versioned IR**
  - The event IR and diagnostic representation MUST be versioned.
  - Any breaking change MUST bump the schema version.

- **I3: Uncertainty is first-class**
  - When ambiguity exists (e.g., parenthesis in filenames), outputs MUST represent uncertainty via explicit confidence and/or "unmapped" states rather than silently guessing.

- **I4: Parser must be panic-free on hostile input**
  - Log parsing MUST not panic on arbitrary bytes; it must either recover or produce bounded, well-formed error output.

- **I5: Incremental friendliness**
  - The system SHOULD support:
    - append-only log growth (recomputing without reparsing from byte 0)
    - incremental source updates (reparsing minimal affected regions)

## Key Data Structures

- **Log buffer**: immutable bytes with append snapshots
- **Event stream**: typed sequence with byte spans
- **File context stack**: push/pop model, reconstructed from log tokens
- **Compilation graph**: optional higher-level structure (files/packages/edges)

- **Document store**: current text snapshots per open document
- **CST/AST**: fault-tolerant parse tree per document
- **Project model**: workspace roots, include graph, file classification
- **Symbol index**: labels, citations, commands/environments, and references

## Engine Boundary

FerroTeX does not replace TeX engines.

- Engine invocation MUST be treated as an adapter layer.
- All parsing/reconstruction logic MUST remain independent of a particular runner (latexmk, tectonic, direct engines).

## Editor Contract

The user experience is mediated through LSP.

- The extension SHOULD remain thin.
- The server SHOULD be the single source of truth for diagnostics, including confidence.

## Diagram

```mermaid
graph TD
  U[User / VS Code] <--> E[VS Code Extension]
  E <--> |LSP over stdio| S[FerroTeX Server (Rust)]
  S --> |spawn / manage| R[Engine Runner]
  R --> |stdout/stderr| S
  R --> |writes| L[(.log file)]
  S --> |tail/parse| L
  S --> |diagnostics| E
```
