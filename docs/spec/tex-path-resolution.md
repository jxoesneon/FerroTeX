# TeX Path Resolution (kpathsea / TEXINPUTS)

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Accurate project graphs and `documentLink` behavior require that FerroTeX resolve paths similarly to TeX.

This specification defines a conservative, configurable strategy for:

- `\input`, `\include`
- package/class discovery
- bibliography/resource discovery

## Resolution Modes

### Mode A: Workspace-Relative (Baseline)

- Resolve relative to the including file’s directory.
- Infer `.tex` extension when omitted.
- Do not search outside workspace roots.

### Mode B: Environment-Aware (Target)

In addition to Mode A:

- respect configured `TEXINPUTS` (from settings)
- respect build runner working directory
- respect output directory settings where relevant

### Mode C: kpathsea-assisted (Opt-in)

- call `kpsewhich` (or equivalent) to resolve inputs
- treat results as authoritative for the engine environment

This mode is powerful but must be explicitly enabled.

## Security Constraints

- External resolution tools (`kpsewhich`) MUST be opt-in.
- Calls MUST be bounded (timeouts) and cancellable.
- Inputs MUST be treated as untrusted.

## Precedence Rules (recommended)

1. explicit absolute path (if present)
2. directory of including file
3. workspace roots (if configured)
4. configured `TEXINPUTS` entries
5. `kpsewhich` (if enabled)

## Diagnostics

The server SHOULD emit diagnostics when resolution fails:

- `FTX0401` — IncludeResolutionFailed

and provide related information showing:

- attempted search paths
- resolution mode

## Interactions

- Path resolution impacts:
  - project include graph (`project-model.md`)
  - document links (`lsp.md`)
  - build runner orchestration (`build-orchestration.md`)
