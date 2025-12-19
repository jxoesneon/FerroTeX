# VS Code Extension Overview

## Purpose

The VS Code extension provides the editor-facing integration for FerroTeX.

The extension is intentionally thin:

- it manages server lifecycle
- it forwards text synchronization events
- it renders diagnostics and UX affordances

Core logic belongs in the Rust server.

## Responsibilities

- **Server lifecycle**
  - spawn `ferrotexd` per workspace
  - restart on configuration change when required

- **Configuration UX**
  - expose and validate settings

- **Commands**
  - compile
  - clean
  - open log excerpt

- **Language feature UX**
  - enable and surface LSP-backed features (completion, outline, navigation, formatting)
  - present confidence/uncertainty where provided by the server

- **Presentation**
  - show diagnostics and related information
  - surface confidence (e.g., "uncertain mapping")

## Non-Responsibilities

- Parsing logs (server responsibility)
- Parsing LaTeX source or maintaining indices (server responsibility)
- Implementing mapping heuristics (server responsibility)
- Maintaining engine-specific logic (server responsibility)

## UX Contract (principles)

- Diagnostics must be stable and non-flapping.
- Low-confidence diagnostics should be visually differentiated.
- Users should be able to inspect provenance quickly (log excerpt + file stack).
