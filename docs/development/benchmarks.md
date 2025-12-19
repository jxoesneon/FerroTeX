# Benchmarks and Performance Gates

FerroTeX claims measurable improvements in latency and robustness; benchmarks must be first-class.

## Benchmark Types

## 1) Microbenchmarks

Target:

- tokenizer throughput
- path recognition
- error-block parsing
- wrap-join heuristics

Recommended tooling:

- `criterion` for Rust

## 2) Macrobenchmarks

Target:

- full parse time for multi-MB logs
- incremental parse time for appended log chunks
- peak RSS and allocation counts

Recommended tooling:

- `hyperfine` for CLI benchmarks
- allocation profiling via Rust tooling (exact choice TBD)

## Representative Workloads

- small project logs (< 200 KB)
- medium logs (1–5 MB)
- large logs (10+ MB)

Include stress fixtures:

- deep nested inputs
- long paths (wrap pressure)
- noisy package output

## Performance Gates (initial targets)

Targets will be refined during implementation:

- Full parse is O(n) with low constant factors.
- Incremental update should reparse only from synchronization anchors.
- Memory overhead should be bounded; avoid copying large substrings.

## Reporting

Benchmarks should record:

- CPU model, RAM, OS
- TeX distribution if relevant
- exact command lines
- Git commit SHA
