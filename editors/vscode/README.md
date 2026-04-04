# FerroTeX: The Scientific LaTeX Compiler

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/hero_banner.png" alt="FerroTeX Hero Banner" width="100%">
</p>

> **"Stop fighting with your TeX environment. FerroTeX brings the intelligence, speed, and reproducibility of modern IDEs to scientific writing."**

[![Open VSX Version](https://img.shields.io/open-vsx/v/ferrotex/ferrotex?style=flat-square&color=blue&logo=open-vsx)](https://open-vsx.org/extension/ferrotex/ferrotex)
[![Installs](https://img.shields.io/open-vsx/dt/ferrotex/ferrotex?style=flat-square&logo=open-vsx)](https://open-vsx.org/extension/ferrotex/ferrotex)
[![Rust](https://img.shields.io/badge/Powered%20by-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/jxoesneon/FerroTeX?style=flat-square&color=green)](https://github.com/jxoesneon/FerroTeX)

## 🚀 Status: v0.23.1 (The "Institutional Excellence" Update)

FerroTeX v0.23.1 marks the platform's transition to an **institutional-grade** system, featuring a **Unified Error Architecture**, hardened **Security Sandboxing (VFS)**, and a robust **Local CI infrastructure** for guaranteed reproducibility.

### 🪟 First-Class Windows Support

FerroTeX is strictly tested on **Windows**, ensuring a professional experience for all researchers. No WSL or complex configuration required.

---

## 💎 The FerroTeX Advantage

| Indexing Speed                                                                    | Semantic Precision                                                                    | Zero-Config Portability                                                       |
| :-------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------ | :---------------------------------------------------------------------------- |
| **Instant Startup**: Hand-written Rust parser indexes 100+ files in milliseconds. | **Math Awareness**: We don't just find text; we understand matrices and environments. | **Bundled Runtime**: No 5GB TeX Live required. We bundle everything you need. |

---

## 🚀 Key Features

### 1. Scientific Reproducibility (`ferrotex.lock`)

The first LaTeX environment with built-in hermeticity. FerroTeX generates a SHA256 content-addressable lockfile for every build.

- **Guarantee**: If it builds on your machine, it builds on your co-author's machine.
- **Security**: Verify the integrity of your package dependencies automatically.

### 2. Semantic Math Analysis

FerroTeX goes beyond regex. Our engine understands the structural properties of math.

- **Jagged Matrix Detection**: Instantly flags row/column dimension mismatches.
- **Delimiter Balancing**: Real-time diagnostics for unclosed or mismatched math brackets.
- **Multicolumn Intelligence**: Correctly handles spanning cells in complex tables.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/feature_diagnostics.png" alt="Semantic Math Diagnostics" width="100%">
</p>

### 3. Unified Error Architecture

Stop deciphering cryptic `.log` files. FerroTeX translates low-level TeX errors into actionable advice with "actionable fixes" suggested for over 50 common patterns.

- **Structured Truth**: All errors are machine-readable and precisely mapped to source spans.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/feature_hover.png" alt="Rich Hover & Documentation" width="100%">
</p>

### 4. Integrated Professional Toolchain

- **Zero-Config Build**: Automatically sets up **Tectonic** if no local distribution is found.
- **Integrated PDF Preview**: Powered by PDF.js with full **SyncTeX** bidirectional search.
- **Image Paste Wizard**: Seamlessly paste images from clipboard with automatic snippet insertion.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/feature_completion.png" alt="Intelligent Context-Aware Autocompletion" width="100%">
</p>

### 5. Interactive Debugging (DAP)

Debug your LaTeX source like actual code.

- **Variables View**: Inspect real-time values of TeX registers (\`\\count\`, \`\\dimen\`, \`\\skip\`).
- **Stepping**: Step-in/Step-over macro expansions.

---

## 📦 Getting Started

1. **Install**: Search for `FerroTeX` in the VS Code Extensions view.
2. **Open**: Open any `.tex` file.
3. **Build**: Press `Cmd+Alt+B` (macOS) or `Ctrl+Alt+B` (Windows/Linux).
4. **Preview**: Click the **Preview Icon** in the editor title menu.

---

## 🤝 Community & Contributing

FerroTeX is open source and built with ❤️ in Rust. We welcome contributions from researchers and developers!

- **Repository**: [github.com/jxoesneon/FerroTeX](https://github.com/jxoesneon/FerroTeX)
- **Issues**: [Report a Bug or Feature Request](https://github.com/jxoesneon/FerroTeX/issues)

---

_Powered by the FerroTeX Language Server and the speed of Rust._
