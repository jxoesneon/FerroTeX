# FerroTeX

![FerroTeX Hero](images/hero_banner.png)

> **"Stop fighting with your LaTeX editor. FerroTeX brings the intelligence of a modern IDE to your scientific writing."**

[![Marketplace Version](https://img.shields.io/visual-studio-marketplace/v/ferrotex.ferrotex?style=flat-square&color=blue)](https://marketplace.visualstudio.com/items?itemName=ferrotex.ferrotex)
[![Installs](https://img.shields.io/visual-studio-marketplace/i/ferrotex.ferrotex?style=flat-square)](https://marketplace.visualstudio.com/items?itemName=ferrotex.ferrotex)
[![License](https://img.shields.io/github/license/jxoesneon/FerroTeX?style=flat-square&color=green)](https://github.com/jxoesneon/FerroTeX)

---

FerroTeX is a next-generation LaTeX engine built in **Rust** 🦀. It replaces the slow, fragile, and silent LaTeX experience with a **fast**, **robust**, and **intelligent** coding environment.

## ✨ Why FerroTeX?

| ⚡ Performance                                              | 🧠 Intelligence                                                   | 👁️ Observability (v0.16.0)                       |
| :---------------------------------------------------------- | :---------------------------------------------------------------- | :----------------------------------------------- |
| **Instant Startup**: Powered by a hand-written Rust parser. | **Context-Aware**: Dynamic completion for packages and citations. | **Human-Readable Errors**: No more cryptic logs. |

## 🚀 Key Capabilities

### 1. Intelligent Completion

Stop visualizing the package documentation in your head. FerroTeX scans your `\usepackage` graph to provide accurate completions for environments and commands.

![Smart Completion](images/feature_completion.png)

### 2. Human-Readable Diagnostics

FerroTeX translates cryptic TeX logs into plain English. We identify over 50 common error patterns and provide actionable fixes directly in your editor.

![Diagnostics](images/feature_diagnostics.png)

### 3. Rich Hovers

Inspect your bibliography without leaving the file. Hover over any `\cite` key to see the full title, author, and year.

![Rich Hovers](images/feature_hover.png)

### 4. Zero-Config Build

**FerroTeX Just Works™.**
If you don't have a TeX distribution, we'll automatically set up **Tectonic**, a modern, lightweight engine.

---

## 📦 Installation

FerroTeX is uniquely designed to be **Zero Dependency**.

1. Install from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=ferrotex.ferrotex) or [Open VSX](https://open-vsx.org/extension/ferrotex/ferrotex).
2. Open a `.tex` file.
3. **That's it.**

## 🔧 Configuration

Customize your experience via VS Code Settings:

![Settings UI](images/feature_settings.png)

| Setting                 | Default     | Description                             |
| :---------------------- | :---------- | :-------------------------------------- |
| `ferrotex.serverPath`   | `ferrotexd` | Path to the language server executable. |
| `ferrotex.build.engine` | `tectonic`  | Choose between `tectonic` or `latexmk`. |
| `ferrotex.lint.enabled` | `true`      | Enable/Disable real-time linting.       |

## 🤝 Contributing

FerroTeX is open source and built with ❤️ in Rust.

- **Repository**: [github.com/jxoesneon/FerroTeX](https://github.com/jxoesneon/FerroTeX)
- **Issues**: [Report a Bug](https://github.com/jxoesneon/FerroTeX/issues)

---

_Powered by Rust and the FerroTeX Language Server._
