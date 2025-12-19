# Architecture Decision Records (ADRs)

ADRs document major design decisions, alternatives considered, and consequences.

## Naming

- `NNNN-title.md` where `NNNN` is a zero-padded sequence number.

## When to Write an ADR

Write an ADR when a decision:

- affects public interfaces (IR schema, LSP behaviors)
- significantly constrains future design
- trades off correctness vs performance vs complexity

## ADR Index

- `0001-typed-event-ir.md`
- `0002-uncertainty-as-first-class.md`
- `0003-thin-extension-thick-server.md`
- `0004-streaming-incremental-parsing.md`
