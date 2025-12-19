# Reproducibility

## Goal

Ensure that reported results and behaviors are reproducible across environments.

## Environment Pinning

FerroTeX evaluation should pin:

- TeX distribution version
- engine versions
- OS image / container base image
- Rust toolchain version
- Node.js version (extension)

## Artifacts

The following artifacts should be published for each evaluation:

- dataset manifest
- fixture logs (or scripts to generate them)
- labeled ground truth subsets (where permissible)
- benchmark scripts and exact command lines
- raw result outputs (JSON/CSV)

## CI Expectations (target)

- run unit and golden tests on every PR
- run benchmarks on demand or nightly
- store benchmark history

## Privacy and Licensing

For real-world logs:

- ensure licensing permits redistribution
- redact sensitive file paths and user information
- maintain a documented redaction process
