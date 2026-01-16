# ADR 0007: Multi-Platform VSIX and Binary Bundling

- **Status:** Accepted
- **Date:** 2026-01-03

## Context

FerroTeX aims for a "Zero-Config" user experience. However, the Language Server (`ferrotexd`) is a native Rust binary. Requiring users to have a Rust toolchain or manually download binaries is a significant friction point.

VS Code and Open VSX support platform-specific extensions, but the distribution logic must be automated and robust.

## Decision

Adopt the **Platform-Specific VSIX Bundling** strategy using a GitHub Actions matrix.

Key requirements:

- **Matrix Build**: Compile `ferrotexd` for `linux-x64`, `win32-x64`, `darwin-arm64`, and `darwin-x64`.
- **Pre-Processing**: Strip and optimize binaries (LTO) before bundling to minimize VSIX size.
- **Binary Staging**: Move the targeted binary to `editors/vscode/bin/ferrotexd` (with `.exe` for Windows).
- **Targeted Packaging**: Use `vsce package --target <target>` to produce distinct VSIX files for each platform.

## Alternatives Considered

- **Runtime Download**: Download the binary from GitHub Releases on the first run.
  - rejected: Brittle (relies on network), higher failure rate, and creates security/permission headaches on Linux/macOS.
- **Universal VSIX (Single package with 4 binaries)**:
  - rejected: Package size becomes too large (~40MB+), exceeding recommended extension limits.

## Consequences

- **"Just Works"**: Users get the correct binary automatically via their extension marketplace.
- **Isolation**: Each VSIX is smaller and only contains the code necessary for the target platform.
- **CI Dependency**: The release process is now heavily dependent on the GitHub Actions matrix being synchronized with the `package.json` version.
