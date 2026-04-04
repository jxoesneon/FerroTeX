---
layout: default
title: Setup
parent: Development
nav_order: 1
---

# Development Setup

This document describes a development environment suitable for implementing FerroTeX.

## Prerequisites

### System

- macOS / Linux / Windows supported (with platform-specific path handling)

### Required System Libraries

FerroTeX depends on several system libraries for text shaping, font rendering, and Unicode support. These are required to build the project.

| Library        | Minimum Version | Purpose                                  |
| -------------- | --------------- | ---------------------------------------- |
| **HarfBuzz**   | >= 2.7.4        | OpenType text shaping engine             |
| **ICU**        | >= 70.1         | International Components for Unicode     |
| **FreeType**   | >= 2.11.0       | Font rendering engine                    |
| **Fontconfig** | >= 2.13.0       | Font configuration and discovery         |
| **OpenSSL**    | >= 3.0          | Cryptography and SSL/TLS                 |
| **Graphite2**  | >= 1.3.0        | Font rendering for complex scripts       |
| **CMake**      | >= 3.16         | Build system generator                   |
| **pkg-config** | any             | Compilation flags helper                 |
| **nasm**       | any             | Netwide Assembler (for optimized builds) |

### Installation by Platform

#### Ubuntu/Debian

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

#### macOS

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
```

#### Windows (vcpkg)

```powershell
vcpkg install icu:x64-windows harfbuzz[graphite2]:x64-windows fontconfig:x64-windows freetype:x64-windows graphite2:x64-windows openssl:x64-windows
```

Set environment variables:

```powershell
$env:TECTONIC_DEP_BACKEND = "vcpkg"
$env:VCPKG_ROOT = "C:/vcpkg"
$env:VCPKGRS_DYNAMIC = "1"
```

### Docker Alternative (Recommended for CI reproduction)

Use Docker for a consistent build environment without installing system dependencies locally:

```bash
./scripts/ci-run.sh
```

This replicates the CI environment (Ubuntu 22.04) and avoids platform-specific issues.

### Rust

- Rust stable toolchain (recommended)
- `cargo` and `rustfmt`

Suggested:

- `cargo clippy`
- `cargo nextest` (optional)

### Node.js (VS Code extension)

- Node.js LTS
- `npm` or `pnpm`

### TeX Toolchain

At least one of:

- TeX Live
- MiKTeX
- Tectonic

## Repository Conventions (planned)

When implementation begins, the repository is expected to contain:

- Rust workspace for the server and parser crates
- an extension package (TypeScript)
- fixtures and golden test outputs

## Local Workflow (intended)

### 1) Build the server

- `cargo build -p ferrotexd`

### 2) Run the server (stdio)

- `cargo run -p ferrotexd -- --stdio`

### 3) Build the extension

- `npm install`
- `npm run build`

### 4) Run the extension in VS Code

- Use the standard VS Code “Run Extension” debug task.

## Configuration During Development

- Use the configuration keys defined in `docs/spec/configuration.md`.
- Prefer workspace-local settings to keep reproductions minimal.

## Troubleshooting

- If diagnostics appear wrong:
  - capture the `.log` file
  - run the offline parser (`ferrotex-cli parse`) once available
  - compare the emitted event spans against the raw log excerpt
