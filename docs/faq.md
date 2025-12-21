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

## What is the MVP?

An offline log parser that outputs a stable JSON schema for events and diagnostics, backed by golden tests.
