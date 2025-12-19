# Datasets

## Purpose

Define, curate, and document datasets used to evaluate FerroTeX.

This document is intentionally separate from `evaluation-plan.md` to keep:

- dataset definitions stable
- evaluation methodology explicit

## Dataset Categories

## 1) Real-World Projects

Target characteristics:

- multi-file structure
- diverse package usage
- presence of common warnings and occasional errors

Required metadata per project:

- source origin and license
- build instructions
- engine/distribution versions used

## 2) Synthetic Stress Fixtures

Synthetic fixtures should be generated to isolate failure modes:

- deep nesting of `\input`
- parentheses and spaces in filenames
- forced wrap scenarios
- noise that resembles log tokens

## 3) Labeled Diagnostic Subset

For correctness scoring, maintain a labeled subset:

- log excerpt
- expected file
- expected line (if applicable)
- notes on ambiguity

## Redaction Policy

Real-world logs may contain absolute paths and usernames.

- Replace absolute paths with workspace-relative placeholders.
- Preserve structure needed for parsing (parentheses nesting, line refs).
- Document any semantic changes introduced by redaction.
