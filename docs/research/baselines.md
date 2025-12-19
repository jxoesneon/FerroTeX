# Baselines

## Purpose

Define baseline tools and comparison methodology for evaluating FerroTeX.

## Baseline Categories

## 1) Regex-Based Log Parsers

Select representative parsers from widely used editor tooling.

Comparison goals:

- localization correctness
- robustness under nesting and wrapping

## 2) Build Tool Outputs

Where available, compare against build tools that provide structured or semi-structured diagnostics.

## 3) Human Interpretation

For ambiguous cases, human labeling can serve as a reference, with explicit labeling criteria.

## Selection Criteria

- popularity (widely used)
- diversity of parsing approaches
- reproducibility (easy to run in CI)

## Reporting

For each baseline, document:

- version
- configuration
- known limitations
- how outputs are mapped into the evaluation metrics
