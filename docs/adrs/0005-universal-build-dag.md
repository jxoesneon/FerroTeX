# ADR 0005: Universal Build DAG & Content-Addressing

- **Status:** Accepted
- **Date:** 2026-01-02

## Context

Traditional LaTeX build tools (e.g., `latexmk`, `arara`) are often script-based and rely on side-effect observation (log file monitoring). This leads to non-determinism, circular dependency issues, and "it works on my machine" syndrome in collaborative scientific environments.

FerroTeX requires a build model that supports:

- **Incrementalism**: Only re-run what is necessary.
- **Reproducibility**: Identical inputs must yield identical outputs.
- **Observability**: Every step of the build must be trackable via structured events.

## Decision

Implement a **Directed Acyclic Graph (DAG)** execution model where nodes are either **Artifacts** (data) or **Transforms** (compute).

Key components:

- **Content-Addressing**: Use SHA256 hashes of all input artifacts to determine if a transform needs execution.
- **`ferrotex.lock`**: A canonical workspace mapping of artifact identifiers to their content fingerprints.
- **Trait-based Abstraction**: Define `Artifact` and `Transform` traits to allow pluggable build steps (e.g., Tectonic, Shell commands, PDF optimization).

## Alternatives Considered

- **Subprocess Monitoring only**: Watch file changes and run `latexmk`.
  - rejected: Hard to guarantee hermeticity and doesn't support the DAP debugging goals.
- **Internal Tectonic Logic only**:
  - rejected: Users still need to run external tools (BibTeX, specialized scripts) which must be part of the same build graph.

## Consequences

- Build logic is decoupled from any specific TeX distribution.
- Collaborative reproducibility is enforced via the lockfile.
- The build graph can be visualized and debugged using standard DAG tools.
- Fingerprinting introduces a minor overhead in I/O but significantly reduces redundant compilation cycles.
