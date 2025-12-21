---
layout: home
title: Home
nav_order: 1
---

# FerroTeX

**A high-performance, type-safe LaTeX language platform written in Rust.**

FerroTeX provides industry-standard IDE features via the Language Server Protocol (LSP) and structured build observability for the TeX ecosystem.

---

## Features

| Feature              | Description                                             |
| -------------------- | ------------------------------------------------------- |
| 🔍 **Smart Editing** | Completion, hover, go-to-definition, references, rename |
| 📄 **PDF Preview**   | Integrated viewer with SyncTeX support                  |
| 🔨 **Build System**  | Latexmk integration with progress feedback              |
| 🧪 **Diagnostics**   | Real-time syntax errors and missing package detection   |

---

## Quick Links

- [Getting Started](development/setup.md) — Set up your development environment
- [Architecture Overview](architecture/overview.md) — Understand the system design
- [LSP Specification](spec/lsp.md) — Language Server Protocol features
- [Roadmap](https://github.com/jxoesneon/FerroTeX/blob/main/ROADMAP.md) — What's coming next

---

## Installation

```bash
# Clone the repository
git clone https://github.com/jxoesneon/FerroTeX.git
cd FerroTeX

# Build the server
cargo build --release

# Install the VS Code extension
cd editors/vscode && npm install && npm run compile
```

---

## License

FerroTeX is dual-licensed under [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT).
