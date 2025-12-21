---
layout: home
title: Home
nav_order: 1
---

<div class="hero" markdown="1">

# FerroTeX

<p class="tagline">
A high-performance, type-safe LaTeX language platform written in Rust. Industry-standard IDE features meet structured build observability.
</p>

<div class="badges">
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/LSP-3.17-blue?style=flat-square" alt="LSP 3.17">
  <img src="https://img.shields.io/badge/license-Apache--2.0%20%2F%20MIT-green?style=flat-square" alt="License">
  <img src="https://img.shields.io/github/stars/jxoesneon/FerroTeX?style=flat-square" alt="Stars">
</div>

<div style="margin-top: 2rem;">
  <a href="development/setup.md" class="btn btn-primary">Get Started →</a>
  <a href="https://github.com/jxoesneon/FerroTeX" class="btn btn-secondary">View on GitHub</a>
</div>

</div>

---

## Why FerroTeX?

Modern LaTeX development deserves modern tooling. FerroTeX bridges the gap between LaTeX's powerful typesetting and contemporary IDE experiences.

<div class="features" markdown="1">

<div class="feature-card" markdown="1">
<div class="icon">🔍</div>

### Smart Editing

Code completion, hover documentation, go-to-definition, find references, and intelligent rename refactoring.

</div>

<div class="feature-card" markdown="1">
<div class="icon">📄</div>

### Integrated PDF Preview

Built-in PDF viewer with SyncTeX support. Click in the PDF to jump to source, or compile and scroll to your cursor.

</div>

<div class="feature-card" markdown="1">
<div class="icon">🔨</div>

### Build Orchestration

Latexmk integration with real-time progress feedback, magic comment support, and intelligent build-on-save.

</div>

<div class="feature-card" markdown="1">
<div class="icon">🧪</div>

### Smart Diagnostics

Real-time syntax errors, missing package detection with one-click installation, and structured log parsing.

</div>

</div>

---

## Architecture

FerroTeX follows a **thin client, thick server** architecture. The Rust language server (`ferrotexd`) handles all analysis, while the VS Code extension provides a minimal UI layer.

```text
┌─────────────────────────────────────────────────────────────────┐
│                        VS Code Extension                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ LSP Client  │  │ PDF Viewer  │  │  SyncTeX / Build UI     │  │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘  │
└─────────┼────────────────┼──────────────────────┼───────────────┘
          │                │                      │
          ▼                ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                      ferrotexd (Rust LSP)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Parser    │  │    Index    │  │   Build Orchestrator    │  │
│  │  (rowan)    │  │  (symbols)  │  │      (latexmk)          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/jxoesneon/FerroTeX.git
cd FerroTeX

# Build the language server
cargo build --release -p ferrotexd

# Install the VS Code extension
cd editors/vscode && npm install && npm run compile
```

Then open VS Code, press `F5` to launch the Extension Development Host, and open any `.tex` file.

---

## Documentation

<div class="features" markdown="1">

<div class="feature-card" markdown="1">

### [Architecture](architecture/overview.md)

System design, data flow, and component responsibilities.

</div>

<div class="feature-card" markdown="1">

### [Specifications](spec/lsp.md)

LSP features, diagnostics, SyncTeX, and configuration.

</div>

<div class="feature-card" markdown="1">

### [Development](development/setup.md)

Local setup, testing, benchmarks, and release process.

</div>

<div class="feature-card" markdown="1">

### [Research](research/baselines.md)

Evaluation methodology, datasets, and reproducibility.

</div>

</div>

---

## License

FerroTeX is dual-licensed under [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT) at your option.

<p style="text-align: center; color: #64748b; margin-top: 3rem;">
  Made with 🦀 by the FerroTeX Contributors
</p>
