# Engine Adapters (Runner Interface)

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

FerroTeX does not implement TeX engines. Instead, it invokes existing tools and normalizes their output into a consistent event stream.

This document specifies the adapter boundary between:

- the **core** (parsing, reconstruction, diagnostics)
- the **runner** (latexmk / tectonic / direct engine invocation)

## Requirements

- **R1: Deterministic command construction**
  - Command lines MUST be constructed from explicit configuration.

- **R2: Safe argument passing**
  - The runner MUST avoid shell interpolation.
  - Arguments MUST be passed as structured arrays (process APIs), not concatenated strings.

- **R3: Controlled working directory**
  - The runner MUST set a working directory (usually workspace root or configured).

- **R4: Output capture**
  - The runner MUST capture stdout/stderr.
  - The runner SHOULD also support tailing `.log` files when available.

- **R5: Cancellation**
  - The runner SHOULD support cancellation (terminate process group where possible).

- **R6: Build targets (export formats)**
  - The runner MUST support an explicit build target selection where the underlying toolchain permits it.
  - Supported targets are specified in `export.md`.

- **R7: Artifact discovery**
  - The runner SHOULD publish primary and secondary artifacts as structured events.
  - Artifact events SHOULD include `format` and `role` (see `log-event-ir.md`).

- **R8: Build orchestration**
  - The runner MUST support either:
    - delegated orchestration (`latexmk`), or
    - an explicit multi-phase pipeline mode.
  - Orchestration semantics are specified in `build-orchestration.md`.

- **R9: SyncTeX workflow**
  - For PDF targets, the runner SHOULD support enabling SyncTeX (see `synctex.md`).

## Adapter Interface (conceptual)

An adapter provides:

- **spawn**: start a compilation
- **events**: stream raw outputs (stdout/stderr) and/or log path discovery
- **status**: exit code, duration, success classification

The core consumes:

- raw byte stream(s)
- optional `.log` file path

## Supported Modes

### latexmk

- Pros: common workflow, handles reruns
- Cons: log output can interleave steps; requires careful correlation

### tectonic

- Pros: deterministic and self-contained builds
- Cons: output format differs; fewer legacy behaviors

Note: `tectonic` is primarily a PDF producer; non-PDF targets require explicit external toolchains.

### direct engine

- Pros: minimal abstraction, useful for research
- Cons: user must manage reruns (bibtex, makeindex)

## Security Notes

- Treat `.tex` sources and produced `.log` content as untrusted.
- Do not auto-execute arbitrary user-specified commands without clear user action.
- Prefer allowlisted known runners with explicit args.
