# Log Grammar and Parsing Rules

## Status

- **Type:** Normative
- **Stability:** Draft

## Problem Statement

TeX logs are not line-oriented data. They are a byte stream containing multiple interleaved constructs:

- file context nesting via parentheses
- diagnostics blocks
- warnings with variable phrasing
- legacy line wrapping that can split paths and messages

The grammar below defines how FerroTeX interprets this stream.

## Normalization

Before tokenization:

- Normalize CRLF to LF.
- Preserve original byte offsets by maintaining a mapping table if normalization changes indices.
  - Alternatively, operate on raw bytes and interpret newlines as-is.

## Token Classes (informal)

- `LPAREN` = `(`
- `RPAREN` = `)`
- `BANG` = `!` (error)
- `LINEREF` = `l.<digits>`
- `WARNING_PREFIX` = matches known warning introducers
  - `LaTeX Warning:`
  - `Package .* Warning:`
  - `Overfull \hbox`
  - `Underfull \hbox`
- `PROMPT` = `?` at start-of-line in interactive contexts

## File Path Recognition

A `FileEnter` event MUST only be emitted when the parser recognizes a plausible path following `LPAREN`.

### Path Heuristics (MUST/SHOULD)

- MUST accept:
  - `./relative/path.tex`
  - `relative/path.tex`
  - absolute POSIX paths
  - Windows-style paths (drive letters) if running on Windows
- SHOULD accept:
  - `.sty`, `.cls`, `.bib`, `.aux`, `.toc` and other auxiliary extensions
- MUST handle spaces conservatively:
  - treat spaces as terminators unless the engine clearly emits quoted paths (rare)

## Line Wrapping Handling

Classic TeX wraps output around ~79 characters.

FerroTeX uses _guarded joining_:

- Join line fragments only when both conditions hold:
  - the first line ends in a token that is syntactically incomplete (e.g., path missing extension or closing delimiter)
  - the next line begins with characters consistent with a continuation (not a diagnostic boundary)

The join algorithm MUST be bounded (e.g., at most N lines joined for a single token).

## Error Blocks

A minimal error block:

- `BANG` followed by message line
- optionally `LINEREF` line
- optionally context lines

The parser MUST treat these as a cohesive structure, not independent lines.

## Recovery

Recovery is required due to ambiguous or malformed logs.

- If `RPAREN` occurs with empty stack, emit `Info` and continue.
- If `LPAREN` appears but no plausible path follows, treat as text.
- If repeated ambiguity is detected, reduce confidence and emit recovery events.

## Output

The parser emits:

- an event stream (see `log-event-ir.md`)
- file stack updates via `FileEnter`/`FileExit`

The reconstruction layer attaches diagnostics using the current stack top.
