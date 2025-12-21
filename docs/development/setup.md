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
