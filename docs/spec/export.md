# Export and Build Outputs Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

FerroTeX must support building and exporting LaTeX projects to **common output formats** in a way that is:

- explicit (user chooses target)
- reproducible (deterministic command construction)
- observable (artifacts are discovered and reported)
- editor-friendly (artifacts can be opened/previewed)

This is a build-layer concern and is complementary to source analysis.

## Terminology

- **Build target**: the requested output format for a build.
- **Artifact**: a file produced by the toolchain (primary output or auxiliary).

## Common Output Targets

FerroTeX accounts for the following targets:

- **PDF** (primary industry standard)
- **DVI**
- **PS** (typically via DVI → PS)
- **HTML** (via TeX-to-HTML toolchains)
- **SVG** (typically via DVI/PDF conversion toolchains)

Other formats (EPUB/DOCX/Markdown) are possible via external conversion tools; they are considered **optional extensions** and must be implemented as explicit adapters.

## Target Semantics

### PDF

- Produced directly by `pdflatex`, `xelatex`, `lualatex`, or `tectonic`.

### DVI

- Produced by `latex` (classic route).

### PS

- Usually produced by:
  - `latex` → DVI
  - `dvips` → PS

### HTML

- Common toolchains include:
  - `make4ht` / `tex4ht`
  - latexmk HTML mode if configured to call `make4ht`

### SVG

- Common toolchains include:
  - `dvisvgm` (DVI → SVG)
  - `pdf2svg` (PDF → SVG)

## Artifact Discovery

The runner MUST publish artifacts as structured events (see `log-event-ir.md`):

- primary output (one or more)
- secondary outputs (auxiliary files)

Artifact discovery strategies (ordered preference):

1. Runner knows output path(s) deterministically from configuration and build target.
2. Parse tool output lines that declare outputs (e.g., “Output written on …”).
3. Filesystem scan within configured output directory (bounded, pattern-based).

## LSP/UX Behaviors

- The server SHOULD expose a command to open the primary artifact:
  - `ferrotex.openOutput`
- The server MAY expose additional commands:
  - `ferrotex.revealOutputInExplorer`
  - `ferrotex.openOutputDirectory`

The extension SHOULD:

- provide a quick action after successful builds (optional)
- show the build target used

## Configuration

Export-related keys are specified in `configuration.md`.

## Security

- Do not execute arbitrary converters implicitly.
- External converters (HTML/SVG) MUST be opt-in and configured explicitly.
