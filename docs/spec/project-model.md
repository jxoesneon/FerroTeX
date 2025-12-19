# Project Model Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Define how FerroTeX models a LaTeX workspace:

- workspace roots
- file classification
- include graph
- entrypoints
- configuration resolution

## Workspace Roots

A workspace may contain:

- one or more root folders
- multiple independent LaTeX projects

The server MUST:

- detect candidate project roots
- allow user configuration to override detection

## File Classification

FerroTeX distinguishes:

- **source**: `.tex`, `.sty`, `.cls`
- **bibliography**: `.bib`
- **auxiliary outputs**: `.aux`, `.toc`, `.bbl`, etc.
- **assets**: images and other included resources

Classification affects indexing and diagnostics.

## Entry Points

An entrypoint is a root document used to build include graphs.

Entrypoints may be discovered via:

- configured `main.tex`
- heuristics (presence of `\\documentclass`)
- build runner configuration

## Include Graph

The include graph is a directed graph:

- nodes are file URIs
- edges represent inclusion via:
  - `\\input{...}`
  - `\\include{...}`
  - `\\subfile{...}` (best-effort)

### Resolution Rules

Path resolution is engine-like but conservative:

- resolve relative to including file directory
- respect configured `TEXINPUTS` where possible (optional)
- try extension inference (`.tex`) when omitted

### Cycles

Cycles are possible (user error).

- detect cycles
- emit diagnostics and stop expanding the cycle

## Multi-Project Workspaces

If multiple entrypoints exist:

- maintain separate project graphs
- expose project selection in configuration

## Outputs and Derived Files

The project model MAY track:

- expected output PDF
- log file location
- auxiliary file locations

These are primarily used by build observability and UX.
