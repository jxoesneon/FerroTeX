# Compatibility Matrix (v1.0.0 Target)

## Status

- **Type:** Normative
- **Stability:** Stable (v1.0.0)

## Purpose

Define what is officially supported at `v1.0.0`.

## Engines

Supported (target):

- pdfTeX (pdflatex)
- XeTeX (xelatex)
- LuaTeX (lualatex)

## Runners

Supported (target):

- latexmk
- direct engine
- tectonic (PDF-focused)

## Export Targets

Supported (target):

- PDF (required)
- DVI (supported where toolchain permits)
- PS (supported via dvips pipeline)
- HTML (supported via configured toolchain)
- SVG (supported via configured toolchain)

## Platforms

Target support:

- macOS
- Linux
- Windows

## Viewers / SyncTeX

- Forward search: required
- Inverse search: best-effort; viewer-dependent

## Known Limitations (must be documented)

- Dynamic macro expansion and catcode changes limit semantic precision.
- Completion and rename are confidence-gated and may be conservative.
- Package metadata coverage depends on bundled inventories.
