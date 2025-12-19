# Build Orchestration (latexmk reruns, biber/bibtex, makeindex)

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Real-world LaTeX builds are multi-step and may require reruns.

FerroTeX must provide a reproducible orchestration model that:

- works with `latexmk` (common workflow)
- works with direct engine invocation (explicit phases)
- supports bibliography and index tools

## Concepts

- **Build session**: one editor-triggered build invocation.
- **Phase**: a single tool invocation within a build session.

Examples of phases:

- `latex` / `pdflatex` / `xelatex` / `lualatex`
- `bibtex` / `biber`
- `makeindex` / `xindy`

## Orchestration Modes

### Mode 1: Delegated (latexmk)

- `latexmk` manages phases and reruns.
- FerroTeX must:
  - capture stdout/stderr per phase when possible
  - tail `.log` as canonical
  - classify final success/failure

### Mode 2: Explicit Pipeline (direct)

- FerroTeX determines the pipeline based on configuration.
- Supports:
  - N engine runs (reruns)
  - optional bibliography run
  - optional index run

## Rerun Policy

The system MUST expose rerun strategy settings.

Typical triggers:

- unresolved references (detected by engine output)
- bibliography changes

The runner SHOULD cap reruns to a configured maximum.

## Eventing

A build session SHOULD be representable as:

- start event
- phase start/phase end
- artifacts
- summary

Build/log parsing must remain correct even when phases interleave (e.g., latexmk).

## Failure Modes

- Tool missing (biber not installed)
- infinite rerun loop
- tool outputs incompatible with parser assumptions

FerroTeX MUST surface these as clear diagnostics or user-facing build errors.
