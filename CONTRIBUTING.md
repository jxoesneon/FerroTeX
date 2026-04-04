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

## System Dependencies

FerroTeX depends on several system libraries for text shaping, font rendering, and Unicode support. These are required to build the project.

### Required Libraries

| Library | Minimum Version | Purpose |
|---------|-----------------|---------|
| **HarfBuzz** | >= 2.7.4 | OpenType text shaping engine |
| **ICU** | >= 70.1 | International Components for Unicode |
| **FreeType** | >= 2.11.0 | Font rendering engine |
| **Fontconfig** | >= 2.13.0 | Font configuration and discovery |
| **OpenSSL** | >= 3.0 | Cryptography and SSL/TLS |
| **Graphite2** | >= 1.3.0 | Font rendering for complex scripts |
| **CMake** | >= 3.16 | Build system generator |
| **pkg-config** | any | Compilation flags helper |
| **nasm** | any | Netwide Assembler (for optimized builds) |

### Checking Installed Versions

**macOS:**
```bash
brew list --versions harfbuzz icu4c freetype fontconfig openssl
```

**Ubuntu/Debian:**
```bash
dpkg -l | grep -E "(harfbuzz|icu|freetype|fontconfig|openssl|graphite)"
```

**Windows (vcpkg):**
```bash
vcpkg list | findstr -i "harfbuzz icu freetype fontconfig openssl graphite"
```

### Ubuntu/Debian Installation

Install all required system dependencies:

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    cmake \
    nasm \
    libicu-dev \
    libharfbuzz-dev \
    libfontconfig1-dev \
    libssl-dev \
    libgraphite2-dev \
    libfreetype6-dev
```

### macOS Installation

Install dependencies via Homebrew:

```bash
brew install harfbuzz icu4c@76 freetype fontconfig
```

**Important:** ICU 76 is specifically required (not ICU 78, which introduces C++17 requirements that may cause build issues):

```bash
brew unlink icu4c@78  # if installed
brew link --force icu4c@76
```

**Configure build environment:**

```bash
cp .cargo/config.toml.example .cargo/config.toml
# Edit .cargo/config.toml and update harfbuzz VERSION to match:
brew list --versions harfbuzz
# Example: if you see "harfbuzz 12.2.0_1", use "12.2.0_1"
```

### Windows Installation

Install dependencies via vcpkg:

```powershell
vcpkg install icu:x64-windows harfbuzz[graphite2]:x64-windows fontconfig:x64-windows freetype:x64-windows graphite2:x64-windows openssl:x64-windows
```

Set environment variables:
```powershell
$env:TECTONIC_DEP_BACKEND = "vcpkg"
$env:VCPKG_ROOT = "C:/vcpkg"
$env:VCPKGRS_DYNAMIC = "1"
```

### Docker Alternative

Use Docker for a consistent build environment without installing system dependencies locally:

```bash
./scripts/ci-run.sh
```

This replicates the CI environment (Ubuntu 22.04) and avoids platform-specific issues.

### Verifying System Dependencies

After installation, verify the build works:

```bash
cargo build
cargo test
```

## macOS-Specific Setup

FerroTeX requires specific configuration on macOS due to dependencies on ICU and HarfBuzz. Follow the [macOS Installation](#macos-installation) steps above, then:

1. **Link ICU 76 (required, not 78):**

   ```bash
   brew unlink icu4c@78  # if installed
   brew link --force icu4c@76
   ```

2. **Configure build environment:**

   ```bash
   cp .cargo/config.toml.example .cargo/config.toml
   # Edit .cargo/config.toml and update harfbuzz VERSION to match:
   brew list --versions harfbuzz
   # Example: if you see "harfbuzz 12.2.0_1", use "12.2.0_1"
   ```

3. **Verify the build:**

   ```bash
   cargo build
   cargo test
   ```

**Alternative:** Use Docker for a consistent build environment:

```bash
./scripts/ci-run.sh
```

## Change Process

- **Discuss**: open an issue for non-trivial changes.
- **Decide**: record major decisions via ADRs in `docs/adrs/`.
- **Implement**: prefer small PRs with tests.
- **Validate**:
  - unit tests
  - golden tests for parser output
  - benchmarks for performance-sensitive changes

## Running Tests

### Running All Tests

To run the complete test suite across the entire workspace:

```bash
cargo test --workspace
```

### Running Tests for a Specific Crate

To run tests for a single crate (useful during development):

```bash
cargo test --package ferrotex-syntax
cargo test --package ferrotex-analysis
cargo test --package ferrotex-build
cargo test --package ferrotex-cli
cargo test --package ferrotex-core
cargo test --package ferrotex-dap
cargo test --package ferrotex-log
cargo test --package ferrotex-math-semantics
cargo test --package ferrotex-package
cargo test --package ferrotexd
```

### Running a Specific Test by Name

To run a single test by its exact name:

```bash
cargo test test_name
cargo test --package ferrotex-syntax test_name
```

To run tests matching a pattern (e.g., all parse module tests):

```bash
cargo test parse::
cargo test --package ferrotex-syntax parse::
```

### Running with Output

To see println! output during tests:

```bash
cargo test -- --nocapture
cargo test --package ferrotex-syntax -- --nocapture
```

### Running Ignored Tests

To run tests marked with `#[ignore]`:

```bash
cargo test -- --ignored
```

### Test Organization

Tests in FerroTeX are organized as follows:

- **Unit tests**: Located in source files under `#[cfg(test)]` modules. These test individual functions and modules in isolation.
- **Integration tests**: Located in `tests/` directories within each crate. These test the crate's public API.
- **Golden tests**: Test fixtures are in `fixtures/` directories (typically containing input files and expected output files).

Example of running golden tests specifically:

```bash
cargo test --package ferrotex-syntax golden
cargo test --package ferrotex-analysis golden
```

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
