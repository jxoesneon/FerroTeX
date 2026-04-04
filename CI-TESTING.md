# Local CI Testing Guide

This guide explains how to test FerroTeX builds locally using a Docker environment that replicates the GitHub Actions CI setup.

## Why Use Docker for CI Testing?

The CI environment runs on Ubuntu 22.04 with specific system dependencies. Local development on macOS or other systems may have different library versions or configurations that can cause build issues not present in CI (or vice versa).

Docker allows you to test in an environment identical to CI **before** pushing to GitHub.

## Quick Start

### Prerequisites

- Docker installed and running
- At least 4GB of free disk space for the image

### Running the Test (Recommended)

Use the unified CI script for the easiest experience:

```bash
# Run the full CI workflow (check, build, test, clippy)
./scripts/ci-run.sh

# Quick check only (fastest feedback)
./scripts/ci-run.sh --check-only

# See all available options
./scripts/ci-run.sh --help
```

The script will:

1. Check Docker is installed and running
2. Build a Docker image matching the CI environment (cached on subsequent runs)
3. Run the full CI workflow inside the container
4. Report success or failure with timing information

### Legacy Script

If you have an older `./test-ci-locally.sh` script, it will continue to work but is deprecated in favor of `./scripts/ci-run.sh`.

### Manual Docker Commands

If you prefer more control than the `./scripts/ci-run.sh` wrapper provides:

```bash
# Build the Docker image
docker build -f Dockerfile.ci-test -t ferrotex-ci-test .

# Run a full build
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo build --workspace --verbose

# Run cargo check (faster)
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo check --workspace

# Run tests
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --workspace

# Run tests for a specific crate (faster iteration)
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --package ferrotex-syntax
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --package ferrotex-analysis

# Run a specific test by name
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --package ferrotex-syntax test_name

# Run tests matching a pattern
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --package ferrotex-syntax parse::

# Run tests with output visible
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --package ferrotex-syntax -- --nocapture

# Get an interactive shell for debugging
docker run --rm -it -v $(pwd):/workspace -w /workspace ferrotex-ci-test bash
```

## What's Included in the CI Image?

The Docker image (`Dockerfile.ci-test`) includes:

- **Base**: Ubuntu 22.04 (matching CI)
- **Rust**: Stable toolchain
- **System Dependencies**:
  - `pkg-config`
  - `cmake`, `nasm`
  - `libharfbuzz-dev`
  - `libfreetype6-dev`
  - `libfontconfig1-dev`
  - `libgraphite2-dev`
  - `libicu-dev`
  - Build essentials

All of these match the GitHub Actions workflow defined in `.github/workflows/ci.yml`.

## Troubleshooting

### Build Image Pull Errors

If you see Docker pull errors, ensure you have a stable internet connection and sufficient disk space.

### Permission Issues

On Linux, you may need to run Docker commands with `sudo` or add your user to the `docker` group.

### Stale Cache

To rebuild the image from scratch (if dependencies change):

```bash
# Using the ci-run.sh script
./scripts/ci-run.sh --no-cache

# Or manually with Docker
docker build --no-cache -f Dockerfile.ci-test -t ferrotex-ci-test .
```

## Notes

- The `Dockerfile.ci-test` and supporting files are .gitignored to keep them local
- Build artifacts created inside Docker are written to your local `target/` directory
- For faster iteration, use `cargo check` instead of `cargo build`
