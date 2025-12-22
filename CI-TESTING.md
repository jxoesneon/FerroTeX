# Local CI Testing Guide

This guide explains how to test FerroTeX builds locally using a Docker environment that replicates the GitHub Actions CI setup.

## Why Use Docker for CI Testing?

The CI environment runs on Ubuntu 22.04 with specific system dependencies. Local development on macOS or other systems may have different library versions or configurations that can cause build issues not present in CI (or vice versa).

Docker allows you to test in an environment identical to CI **before** pushing to GitHub.

## Quick Start

### Prerequisites

- Docker installed and running
- At least 4GB of free disk space for the image

### Running the Test

```bash
./test-ci-locally.sh
```

This script will:

1. Build a Docker image matching the CI environment
2. Run `cargo build --workspace --verbose` inside the container
3. Report success or failure

### Manual Docker Commands

If you prefer more control:

```bash
# Build the Docker image
docker build -f Dockerfile.ci-test -t ferrotex-ci-test .

# Run a full build
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo build --workspace --verbose

# Run cargo check (faster)
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo check --workspace

# Run tests
docker run --rm -v $(pwd):/workspace -w /workspace ferrotex-ci-test cargo test --workspace

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
docker build --no-cache -f Dockerfile.ci-test -t ferrotex-ci-test .
```

## Notes

- The `Dockerfile.ci-test` and supporting files are .gitignored to keep them local
- Build artifacts created inside Docker are written to your local `target/` directory
- For faster iteration, use `cargo check` instead of `cargo build`
