# Glossary

## Terms

### TeX / LaTeX

The TeX typesetting system and the LaTeX macro package ecosystem.

### Engine

A TeX executable such as pdfTeX, XeTeX, or LuaTeX.

### Log

The textual stream emitted by the engine (and/or auxiliary build tools). Typically materialized as a `.log` file.

### Event IR

FerroTeX’s typed intermediate representation of log semantics (file transitions, diagnostics, warnings, etc.) with provenance spans.

### Provenance

The ability to trace an emitted event/diagnostic back to an exact byte span in the source log buffer.

### File Context Stack

The reconstructed nesting of file inputs inferred from the log’s parenthesis structure.

### Confidence

A numeric score in `[0, 1]` indicating how strongly FerroTeX believes a mapping or event interpretation is correct.

### LSP

Language Server Protocol. FerroTeX uses LSP to publish diagnostics and editor features.

### DAP

Debug Adapter Protocol. FerroTeX may use DAP for exploratory “debugging” semantics for TeX.

### Golden Test

A test in which the expected output (e.g., JSON event stream) is stored and compared for regression detection.
