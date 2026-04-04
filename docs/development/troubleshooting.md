---
layout: default
title: Troubleshooting
parent: Development
nav_order: 5
---

# Troubleshooting and Diagnostics

This guide provides institutional-grade procedures for diagnosing and resolving common issues with FerroTeX.

## Diagnostic Workflow

When encountering an issue, follow this standardized diagnostic sequence:

1. **Check LSP Status**: View the `FerroTeX Language Server` output channel in VS Code.
2. **Verify CI Parity**: Run `./scripts/ci-run.sh --check-only` to see if the issue is reproducible in the clean Docker environment.
3. **Inspect Logs**: Check the generated `.log` file in your build directory.
4. **Validate Dependencies**: Run `cargo check --workspace` to ensure all system libraries (ICU, HarfBuzz) are correctly linked.

## Common Issues

### 1. Missing System Dependencies (macOS)

**Symptom**: Compilation fails with "ICU library not found" or linking errors.
**Resolution**:

```bash
brew install icu4c
export LDFLAGS="-L/opt/homebrew/opt/icu4c/lib"
export CPPFLAGS="-I/opt/homebrew/opt/icu4c/include"
export PKG_CONFIG_PATH="/opt/homebrew/opt/icu4c/lib/pkgconfig"
```

_Note: FerroTeX requires ICU 76 or higher._

### 2. Docker CI Failures

**Symptom**: `./scripts/ci-run.sh` fails with "Docker daemon not running".
**Resolution**: Ensure Docker Desktop is open and the daemon has finished initializing. On Linux, ensure your user is in the `docker` group.

### 3. Diagnostic Mismatch

**Symptom**: Errors in the source file don't align with the actual line numbers.
**Resolution**:

- Capture the `.log` file.
- Run the offline parser: `ferrotex-cli parse <file>.log`.
- Compare the emitted event spans against the raw log excerpt to identify mapping offsets.

### 4. Language Server Crash

**Symptom**: "FerroTeX Server has crashed 5 times in the last 3 minutes."
**Resolution**: Check for infinite recursion in macros. The Abstract Machine has a `max_depth` (default 1000) to prevent stack overflows, but complex circular dependencies may still trigger issues.

## Debugging the Abstract Machine

To debug macro expansion issues without running a full TeX engine:

1. Use the [DAP debugger](../spec/dap.md) to step through expansion.
2. Enable ghost expansion by hovering over the problematic macro.

## Reporting Bugs

For institutional support, please include:

- The full `.log` output of the failing build.
- A minimal reproducible example (`.tex` file).
- The output of `ferrotex-cli --version` and `cargo --version`.
