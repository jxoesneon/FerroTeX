# FerroTeX: The Scientific LaTeX Compiler

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/hero_banner.png" alt="FerroTeX Hero Banner" width="100%">
</p>

> **"Stop fighting with your TeX environment. FerroTeX brings the intelligence, speed, and reproducibility of modern IDEs to scientific writing."**

[![Open VSX Version](https://img.shields.io/open-vsx/v/ferrotex/ferrotex?style=flat-square&color=blue&logo=open-vsx)](https://open-vsx.org/extension/ferrotex/ferrotex)
[![Installs](https://img.shields.io/open-vsx/dt/ferrotex/ferrotex?style=flat-square&logo=open-vsx)](https://open-vsx.org/extension/ferrotex/ferrotex)
[![Rust](https://img.shields.io/badge/Powered%20by-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/jxoesneon/FerroTeX?style=flat-square&color=green)](https://github.com/jxoesneon/FerroTeX)

## v0.22.0: The Debug & Reliability Update

FerroTeX v0.22.0 introduces the **Tectonic Debug Driver**, allowing you to inspect TeX internals (`\count`, `\dimen`) and step through macros like code. This release also marks a major milestone in stability with **>95% Semantic Engine Test Coverage**.

### 🪟 Now on Windows!

FerroTeX is strictly tested on **Windows**, ensuring a first-class experience for all developers. No WSL required.

## 💎 The FerroTeX Advantage

| Indexing Speed                                                                    | Semantic Precision                                                                    | Zero-Config Portability                                                       |
| :-------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------ | :---------------------------------------------------------------------------- |
| **Instant Startup**: Hand-written Rust parser indexes 100+ files in milliseconds. | **Math Awareness**: We don't just find text; we understand matrices and environments. | **Bundled Runtime**: No 5GB TeX Live required. We bundle everything you need. |

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

<img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/feature_diagnostics.png" alt="Semantic Math Diagnostics" width="100%">

### 3. Integrated Professional Toolchain

- **Zero-Config Build**: Automatically sets up **Tectonic** (the modern TeX engine) if no local TeX distribution is found.
- **Bundled Binary**: Comes with pre-compiled, optimized Rust binaries for Linux, macOS, and Windows. **It just works.**
- **Integrated PDF Preview**: Powered by PDF.js with full **SyncTeX** bidirectional search (Ctrl+Click to jump between code and PDF).

<img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/feature_completion.png" alt="Intelligent Context-Aware Autocompletion" width="100%">

### 4. Human-Readable Diagnostics

Stop deciphering cryptic `.log` files. FerroTeX translates low-level TeX errors into actionable advice with "actionable fixes" suggested for over 50 common patterns.

<img src="https://github.com/jxoesneon/FerroTeX/raw/main/editors/vscode/images/feature_hover.png" alt="Rich Hover & Documentation" width="100%">

### 5. Interactive Debugging (DAP)

Debug your LaTeX source like actual code.

- **Variables View**: Inspect the real-time values of TeX registers (`\count`, `\dimen`, `\skip`) and macros.
- **Stepping**: Step-in/Step-over macro expansions and file inclusions.
- **Watch**: Monitor specific control sequences as your document builds.

---

## 📦 Getting Started

FerroTeX is designed for **Frictionless Onboarding**.

1. **Install**: Search for `FerroTeX` in the VS Code Extensions view (or find it on [Open VSX](https://open-vsx.org/extension/ferrotex/ferrotex)).
2. **Open**: Open any `.tex` or `.latex` file.
3. **Build**: Press `Cmd+Alt+B` (macOS) or `Ctrl+Alt+B` (Windows/Linux) to generate your PDF.
4. **Preview**: Click the **Preview Icon** in the editor title menu to view your document in the integrated viewer.

## 🔧 Configuration

FerroTeX works out-of-the-box, but you can fine-tune it via VS Code Settings:

| Setting                          | Default | Description                                                   |
| :------------------------------- | :------ | :------------------------------------------------------------ |
| `ferrotex.build.engine`          | `auto`  | Choose between `tectonic`, `latexmk`, or auto-detection.      |
| `ferrotex.build.autoBuildOnSave` | `true`  | Update your PDF preview in real-time on every save.           |
| `ferrotex.imagePaste.enabled`    | `true`  | Integrated wizard for pasting images directly from clipboard. |
| `ferrotex.lint.enabled`          | `true`  | Toggle the high-fidelity semantic math linters.               |

## 🤝 Community & Contributing

FerroTeX is open source and built with ❤️ in Rust. We welcome contributions from researchers, developers, and TeX enthusiasts!

- **Repository**: [github.com/jxoesneon/FerroTeX](https://github.com/jxoesneon/FerroTeX)
  If you encounter any issues or have feature requests, please file an issue on [GitHub](https://github.com/jxoesneon/FerroTeX/issues).
- **Issues**: [Report a Bug or Feature Request](https://github.com/jxoesneon/FerroTeX/issues)

---

_Powered by the FerroTeX Language Server and the speed of Rust._
