---
layout: default
title: FAQ
nav_order: 7
---

# FAQ

## What problem does FerroTeX solve?

TeX engines emit diagnostics in unstructured logs that are difficult to map deterministically to source files and locations. FerroTeX reconstructs compilation context and produces structured diagnostics suitable for IDEs.

## Why not just use regex?

Regex-based parsers are brittle under:

- nested file contexts
- line wrapping
- distribution differences
- package-emitted noise

FerroTeX uses a typed, state-machine approach with explicit recovery and confidence.

## Is FerroTeX a TeX engine?

No. FerroTeX invokes existing engines and build tools.

## Does FerroTeX guarantee perfect mappings?

No. Some ambiguity is intrinsic to TeX logs and macro execution. FerroTeX makes uncertainty explicit rather than silently guessing.

## Will FerroTeX support all TeX engines?

The intent is to support pdfTeX, XeTeX, and LuaTeX. Some advanced introspection features may be LuaTeX-first.

## What version is current?

v0.24.0 — "Diagnostic Completeness". See the [CHANGELOG](https://github.com/jxoesneon/FerroTeX/blob/main/CHANGELOG.md) for full history.

## Does FerroTeX warn about outdated LaTeX commands?

Yes, as of v0.24.0. LaTeX 2.09 font commands (`\bf`, `\it`, `\rm`, etc.), `$$...$$` display math, and obsolete packages (`times`, `a4wide`, `epsfig`, `psfig`) produce `warning` diagnostics on every `didOpen`/`didChange`. Each warning includes a one-click quick-fix code action.

## Does FerroTeX support BibLaTeX?

At the IDE layer, yes. `\addbibresource{refs.bib}` is indexed for citation key completion and cross-file validation, identical to `\bibliography`. `\RequirePackage` is scanned alongside `\usepackage`. The compile-time bibliography backend (Biber vs BibTeX) is determined by your build runner, not FerroTeX.
