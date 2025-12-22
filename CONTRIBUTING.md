# Contributing to FerroTeX

FerroTeX is intended to be both:

- an engineering project (robust tooling for TeX/LaTeX)
- a research artifact (reproducible evaluation of diagnostic parsing)

This document defines contribution standards to keep both goals aligned.

## Project Values

- **Correctness and clarity** over cleverness.
- **Measured performance** over assumed performance.
- **Explicit uncertainty** rather than silently wrong diagnostics.
- **Small, reviewable changes** that preserve maintainability.

## What You Can Contribute

- Parser improvements (new log constructs, better recovery).
- Test fixtures (real logs, synthetic stress logs).
- Benchmarks (performance and correctness).
- LSP features and VS Code UX.
- Documentation and ADRs.

See `docs/development/setup.md`.

### macOS-Specific Setup

FerroTeX requires specific configuration on macOS due to dependencies on ICU and HarfBuzz:

1. **Install dependencies via Homebrew:**

   ```bash
   brew install harfbuzz icu4c@76 freetype fontconfig
   ```

2. **Link ICU 76 (required, not 78):**

   ```bash
   brew unlink icu4c@78  # if installed
   brew link --force icu4c@76
   ```

3. **Configure build environment:**

   ```bash
   cp .cargo/config.toml.example .cargo/config.toml
   # Edit .cargo/config.toml and update harfbuzz VERSION to match:
   brew list --versions harfbuzz
   # Example: if you see "harfbuzz 12.2.0_1", use "12.2.0_1"
   ```

4. **Verify the build:**

   ```bash
   cargo build
   cargo test
   ```

**Alternative:** Use Docker for a consistent build environment:

```bash
./test-ci-locally.sh
```

This replicates the CI environment and avoids macOS-specific issues.

## Change Process

- **Discuss**: open an issue for non-trivial changes.
- **Decide**: record major decisions via ADRs in `docs/adrs/`.
- **Implement**: prefer small PRs with tests.
- **Validate**:
  - unit tests
  - golden tests for parser output
  - benchmarks for performance-sensitive changes

## Testing Expectations

- Parser changes must include:
  - at least one new fixture in `fixtures/` (or a documented reason)
  - an updated or new golden output
- LSP changes should include:
  - protocol-level tests where feasible
  - manual validation steps in the PR description

## Performance Discipline

If your change affects parsing or reconstruction:

- include before/after benchmark data
- call out allocation/latency changes

## Commit Style

- Use clear, descriptive commit messages.
- Prefer conventional scope prefixes when useful (not required):
  - `parser:`
  - `lsp:`
  - `engine:`
  - `docs:`

## Code of Conduct

By participating in this project you agree to abide by `CODE_OF_CONDUCT.md`.
