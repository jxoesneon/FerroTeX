# Performance SLOs

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

To be industry-standard, FerroTeX must meet responsiveness expectations.

This document defines target Service Level Objectives (SLOs).

## Latency Targets (interactive)

On representative medium projects:

- completion p95: <= 100 ms
- hover p95: <= 100 ms
- definition/references p95: <= 200 ms
- semantic tokens full p95: <= 300 ms (with caching)

These targets are advisory until implementation benchmarks exist.

## Parsing and Indexing

- incremental parse update: proportional to change size
- index update on save: bounded by file size + dependency fanout

## Memory

- avoid duplicating large document strings
- bounded caches with eviction policies

## Cancellation

All long-running LSP requests MUST be cancellable.

## Benchmarking

Benchmarks must be recorded as per `docs/development/benchmarks.md`.
