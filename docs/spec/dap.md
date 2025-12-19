# Debug Adapter Protocol (DAP) — Exploratory Specification

## Status

- **Type:** Research / Exploratory
- **Stability:** Experimental

## Scope

DAP support is not required for the initial FerroTeX MVP.

This document defines a feasible interpretation of “debugging” for TeX.

## Debugging Semantics for TeX

Unlike conventional programs, TeX is a macro expansion machine with limited runtime introspection.

FerroTeX considers the following debugging primitives:

- **Pause points**
  - before/after `\input`
  - before/after loading packages
  - at expansion of a specific control sequence (engine-dependent)

- **Inspection**
  - show macro definitions (e.g., `\meaning`)
  - show token lists / expansion traces
  - show selected counters and registers

## Feasibility Notes

- LuaTeX is the primary candidate for meaningful introspection.
- pdfTeX/XeTeX may only support limited tracing modes.

## Safety and UX Constraints

- Debugging MUST not execute arbitrary commands beyond TeX engine invocation.
- Debug sessions SHOULD be opt-in and clearly marked experimental.

## Deliverable Definition

A minimal DAP prototype is considered successful if it can:

- run compilation in an interactive mode
- pause at a predefined instrumentation point
- return an inspection payload to the client

Further work may define breakpoints as source annotations translated into instrumentation macros.
