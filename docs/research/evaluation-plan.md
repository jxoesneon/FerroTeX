# Evaluation Plan (Research-Grade)

## Purpose

This document defines how FerroTeX will be evaluated as both:

- an engineering system (correctness, performance, UX)
- a research contribution (methods, reproducibility, validity)

## Hypotheses

- **H1 (Localization accuracy):** FerroTeX improves file/line mapping accuracy over regex baselines.
- **H2 (Robustness):** FerroTeX maintains parsing correctness across engine/distribution variations.
- **H3 (Performance):** FerroTeX achieves lower latency to usable diagnostics and better incremental behavior.

## Datasets

### Real-World Corpora

- multi-file theses and dissertations
- package-heavy projects (TikZ, minted, biblatex)
- arXiv-style sources (when licensing permits)

### Synthetic Corpora

Generate fixtures targeting known failure modes:

- long path segments forcing wrap
- parentheses in filenames
- deep nesting of `\input`
- interleaved warnings and errors

## Ground Truth

Ground truth is defined as:

- the correct file containing the line referenced by the engine (when present)
- the correct line number (from `l.<n>`)

For diagnostics without explicit line references, ground truth may be:

- human-labeled
- or excluded from strict line-level scoring

## Metrics

### Localization

- **File@1 accuracy**
- **Line exact accuracy**
- **Line±k accuracy** (k = 1, 2, 5)
- **Unmapped rate** (how often the system declines to guess)

### Calibration

- Compare confidence to empirical correctness:
  - high-confidence predictions should be correct more often than low-confidence predictions

### Performance

- time to first diagnostic
- time to stable diagnostic set
- incremental update latency as a function of appended bytes
- peak RSS and allocation counts

### UX (optional study)

- time-to-fix on controlled tasks
- subjective usability questionnaires

## Baselines

At least two baselines should be used:

- a regex-based parser from common editor tooling
- a widely-used build tool’s parsing output (where applicable)

## Experimental Design

- Same corpus across all tools.
- Multiple TeX engines (pdfTeX/XeTeX/LuaTeX) when feasible.
- Multiple distributions (TeX Live, MiKTeX) when feasible.

## Threats to Validity

- Engine logs are not a complete execution trace.
- Packages can emit text that mimics log tokens.
- Human labeling can introduce bias.

Mitigations:

- keep datasets and labeling criteria explicit
- report failure modes and unmapped cases
