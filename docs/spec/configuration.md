# Configuration Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Goals

- Minimal configuration for typical users
- Explicit control for engine runner and parsing behavior
- Reproducible builds and diagnostics

## Configuration Layers

Configuration MAY be sourced from:

- VS Code settings (primary)
- workspace config file (optional; future)
- defaults

Precedence (highest first):

1. explicit command parameters
2. VS Code settings
3. workspace file
4. defaults

## Key Settings (proposed)

### Project / Workspace

- `ferrotex.project.roots`: string[] (optional override)
- `ferrotex.project.entrypoints`: string[] (optional; e.g., `main.tex`)
- `ferrotex.project.enableMultiProject`: boolean

### TeX Path Resolution

- `ferrotex.paths.mode`: `workspace | env | kpsewhich`
- `ferrotex.paths.texInputs`: string[] (optional; used in `env` mode)
- `ferrotex.paths.kpsewhich.enable`: boolean
- `ferrotex.paths.kpsewhich.timeoutMs`: number

### Source Parsing

- `ferrotex.source.enable`: boolean
- `ferrotex.source.incremental`: boolean
- `ferrotex.source.maxDocumentSizeBytes`: number

### Indexing

- `ferrotex.index.enable`: boolean
- `ferrotex.index.labels`: boolean
- `ferrotex.index.citations`: boolean
- `ferrotex.index.bibliography`: boolean
- `ferrotex.index.workspaceSymbols`: boolean

### Completion

- `ferrotex.completion.enable`: boolean
- `ferrotex.completion.commands`: boolean
- `ferrotex.completion.environments`: boolean
- `ferrotex.completion.labels`: boolean
- `ferrotex.completion.citations`: boolean
- `ferrotex.completion.paths`: boolean

### Semantic Tokens

- `ferrotex.semanticTokens.enable`: boolean

### Formatting

- `ferrotex.format.enable`: boolean
- `ferrotex.format.indentSize`: number
- `ferrotex.format.preserveMath`: boolean
- `ferrotex.format.preserveComments`: boolean

### Engine

- `ferrotex.engine.mode`: `latexmk | tectonic | direct`
- `ferrotex.engine.command`: string (for `direct`)
- `ferrotex.engine.args`: string[]
- `ferrotex.engine.workingDirectory`: string

### Export / Build Outputs

- `ferrotex.build.target`: `pdf | dvi | ps | html | svg`
- `ferrotex.build.outputDirectory`: string (optional)
- `ferrotex.build.openAfterBuild`: boolean

- `ferrotex.build.html.tool`: `make4ht | tex4ht | latexml | lwarp` (optional)
- `ferrotex.build.svg.tool`: `dvisvgm | pdf2svg` (optional)

### Build Orchestration

- `ferrotex.build.mode`: `latexmk | pipeline`
- `ferrotex.build.maxReruns`: number
- `ferrotex.build.bibliography.tool`: `biber | bibtex` (optional)
- `ferrotex.build.index.tool`: `makeindex | xindy` (optional)

### SyncTeX / PDF Workflow

- `ferrotex.synctex.enable`: boolean
- `ferrotex.synctex.inverseSearch.enable`: boolean
- `ferrotex.pdf.viewer`: `vscode | system | custom`
- `ferrotex.pdf.viewer.command`: string (for `custom`)

### Log Ingestion

- `ferrotex.log.source`: `file | stdout | both`
- `ferrotex.log.path`: string (optional)
- `ferrotex.log.follow`: boolean

### Parser

- `ferrotex.parser.maxJoinLines`: number
- `ferrotex.parser.confidenceThreshold`: number
- `ferrotex.parser.maxFileStackDepth`: number

### Diagnostics

- `ferrotex.diagnostics.publishInterim`: boolean
- `ferrotex.diagnostics.includeProvenance`: boolean

## Compatibility

Configuration keys MUST be versioned and changes documented.
