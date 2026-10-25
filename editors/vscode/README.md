<p align="center">
  <img src="https://github.com/jxoesneon/FerroTex/raw/main/editors/vscode/images/hero_banner.png" alt="FerroTeX Hero Banner" width="100%">
</p>

# FerroTeX for Visual Studio Code

<p align="center">
  <strong>The Scientific LaTeX IDE Platform. Zero-Config. Instant Startup. Powered by Rust.</strong>
</p>

<p align="center">
  <a href="https://marketplace.visualstudio.com/items?itemName=ferrotex.ferrotex"><img src="https://img.shields.io/badge/VS%20Marketplace-v0.25.0-007ACC?style=flat-square&logo=visualstudiocode" alt="Visual Studio Marketplace Version"></a>
  <a href="https://open-vsx.org/extension/ferrotex/ferrotex"><img src="https://img.shields.io/badge/Open%20VSX-v0.25.0-purple?style=flat-square" alt="Open VSX Version"></a>
  <a href="https://github.com/jxoesneon/FerroTex"><img src="https://img.shields.io/badge/E2E%20Tests-88%2F88%20Passing-brightgreen?style=flat-square" alt="E2E Tests"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Core-Rust%20🦀-orange?style=flat-square&logo=rust" alt="Rust Core"></a>
  <a href="https://github.com/jxoesneon/FerroTex/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-green?style=flat-square" alt="License"></a>
</p>

> [!TIP]
> **No 5GB TeX Live Required. Zero Setup Friction.**  
> FerroTeX replaces fragmented scripts and massive compiler installations with an integrated, self-provisioning Rust engine. Open a `.tex` file and start writing immediately.

---

## ⚡ The FerroTeX Difference

| Feature                  | Legacy LaTeX Extensions          | FerroTeX (v0.25.0)                                       |
| :----------------------- | :------------------------------- | :------------------------------------------------------- |
| **Prerequisites**        | 5GB+ TeX Live / MacTeX + Perl    | **None.** Bundled on-demand engine                       |
| **Cold Startup Latency** | 3 – 8 seconds                    | **< 45 milliseconds**                                    |
| **Memory Footprint**     | 300 MB – 800 MB                  | **< 40 MB**                                              |
| **Math Verification**    | Superficial text regex           | **Deep AST dimension & delimiter tracking**              |
| **Compilation Errors**   | 100-line raw log dumps           | **Mapped inline diagnostics with 1-click Quick Fixes**   |
| **Live PDF Preview**     | External viewer or heavy plugins | **Built-in PDF.js viewer with instant auto-reload**      |
| **Step Debugging**       | None                             | **Full DAP register & macro stepping**                   |
| **Image Paste Wizard**   | Manual file copying & typing     | **1-click clipboard paste with auto-generated snippets** |
| **Reproducibility**      | Machine-dependent                | **Hermetic `ferrotex.lock` dependency tracking**         |

---

## 🎬 Real Feature Walkthrough

### 1. Zero-Config Build & Live PDF Preview with SyncTeX

Compile complex documents on a clean machine without installing external TeX distributions. The integrated PDF viewer automatically re-renders on save with responsive auto-fit scaling and SyncTeX synchronization.

> **Key Shortcut**: Press <kbd>Cmd</kbd>+<kbd>Alt</kbd>+<kbd>B</kbd> (macOS) or <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>B</kbd> (Windows/Linux) to build, and click the **PDF Preview** icon in the editor toolbar.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTex/raw/main/editors/vscode/images/feature_preview.gif" alt="Zero-Config Build and Live PDF Preview" width="100%">
</p>

---

### 2. Semantic Math Diagnostics & One-Click Quick Fixes

FerroTeX continuously validates your mathematical environments in real-time. Jagged matrix rows (dimension mismatches) and unmatched delimiters are surfaced with exact line squiggles, and deprecated LaTeX 2.09 commands (`\bf`, `\it`, `$$...$$`) offer one-click automatic modernizations.

> **Key Shortcut**: Place cursor on any warning squiggle and press <kbd>Cmd</kbd>+<kbd>.</kbd> / <kbd>Ctrl</kbd>+<kbd>.</kbd> to apply the suggested quick fix.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTex/raw/main/editors/vscode/images/feature_diagnostics.gif" alt="Semantic Math Diagnostics and Quick Fixes" width="100%">
</p>

---

### 3. Contextual Autocompletion & Tab-Stop Snippets

Intelligent completions adapt dynamically to your cursor context. Type `\begin{pm` and press Tab to expand a full `pmatrix` environment with pre-balanced delimiters and cursor tab stops positioned for immediate element entry.

> **Key Shortcut**: Press <kbd>Ctrl</kbd>+<kbd>Space</kbd> anywhere to trigger contextual suggestions.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTex/raw/main/editors/vscode/images/feature_completion.gif" alt="Contextual Autocompletion" width="100%">
</p>

---

### 4. Rich Citation & Macro Hover Documentation

Hover over any citation key (`\cite{...}`) to view author, title, publication venue, and DOI extracted directly from your `.bib` library. Hovering over LaTeX commands displays rich signatures, required packages, and parameter explanations.

<p align="center">
  <img src="https://github.com/jxoesneon/FerroTex/raw/main/editors/vscode/images/feature_hover.gif" alt="Rich Interactive Hovers" width="100%">
</p>

---

## 🚀 Getting Started in 10 Seconds

```text
Step 1: Open any folder with a .tex file
Step 2: Press Cmd+Alt+B (Ctrl+Alt+B on Windows/Linux) to build
Step 3: Click the split-view Preview icon in the editor title bar
```

> [!NOTE]
> If no local TeX engine is detected on your machine, FerroTeX will automatically offer to provision our bundled zero-config Tectonic compiler on the fly.

---

## ⌨️ Command Palette & Keybinding Reference

| Command Title                         | Command Identifier         |             Keybinding (macOS)             |           Keybinding (Win/Linux)            |
| :------------------------------------ | :------------------------- | :----------------------------------------: | :-----------------------------------------: |
| **FerroTeX: Build Document**          | `ferrotex.build`           | <kbd>Cmd</kbd>+<kbd>Alt</kbd>+<kbd>B</kbd> | <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>B</kbd> |
| **FerroTeX: Open PDF Preview**        | `ferrotex.openPreview`     |            _Editor Title Icon_             |             _Editor Title Icon_             |
| **FerroTeX: Sync to PDF**             | `ferrotex.syncToPdf`       | <kbd>Cmd</kbd>+<kbd>Alt</kbd>+<kbd>J</kbd> | <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>J</kbd> |
| **FerroTeX: Clean Auxiliary Files**   | `ferrotex.clean`           |             _Command Palette_              |              _Command Palette_              |
| **FerroTeX: Install Tectonic Engine** | `ferrotex.installTectonic` |             _Command Palette_              |              _Command Palette_              |

---

## 🖼️ Clipboard Image Paste Wizard

Pasting diagrams and screenshots into your LaTeX paper is effortless:

1. Copy any image to your clipboard (screenshot, diagram, web image).
2. Inside any `.tex` file, press <kbd>Cmd</kbd>+<kbd>V</kbd> / <kbd>Ctrl</kbd>+<kbd>V</kbd>.
3. FerroTeX automatically prompts for a clean filename, saves the PNG into your project's `figures/` directory, and inserts the formatted `\begin{figure}` snippet right at your cursor.

---

## ⚙️ Configuration Reference

<details open>
<summary><strong>Core & Build Engine Settings</strong></summary>

| Setting Key                          |   Type    |    Default    | Description                                                                                     |
| :----------------------------------- | :-------: | :-----------: | :---------------------------------------------------------------------------------------------- |
| `ferrotex.build.engine`              |  `enum`   |   `"auto"`    | Compiler engine (`"auto"`, `"tectonic"`, `"latexmk"`, `"pdflatex"`, `"xelatex"`, `"lualatex"`). |
| `ferrotex.build.autoBuildOnSave`     | `boolean` |    `false`    | Automatically recompile document when saving.                                                   |
| `ferrotex.build.outputDirectory`     | `string`  |     `""`      | Directory for build artifacts (e.g. `"build"`).                                                 |
| `ferrotex.build.cleanAuxiliaryFiles` | `boolean` |    `false`    | Automatically clean auxiliary files (`.aux`, `.log`, etc.) after build.                         |
| `ferrotex.build.showOutputPanel`     |  `enum`   |  `"onError"`  | When to reveal the build output panel (`"always"`, `"onError"`, `"never"`).                     |
| `ferrotex.serverPath`                | `string`  | `"ferrotexd"` | Custom path to the `ferrotexd` binary.                                                          |

</details>

<details>
<summary><strong>Diagnostics, Linting & Semantic Verification Settings (Click to Expand)</strong></summary>

| Setting Key                                |   Type    | Default | Description                                                   |
| :----------------------------------------- | :-------: | :-----: | :------------------------------------------------------------ |
| `ferrotex.lint.enabled`                    | `boolean` | `true`  | Master toggle for background document linting.                |
| `ferrotex.lint.onType`                     | `boolean` | `true`  | Validate document continuously as you type.                   |
| `ferrotex.lint.checkMath`                  | `boolean` | `true`  | Enable algebraic matrix dimension and delimiter validation.   |
| `ferrotex.lint.checkCitations`             | `boolean` | `true`  | Validate citation keys against loaded `.bib` files.           |
| `ferrotex.lint.checkPackages`              | `boolean` | `true`  | Verify package availability and imports.                      |
| `ferrotex.diagnostics.humanReadableErrors` | `boolean` | `true`  | Translate low-level TeX errors into plain, actionable advice. |
| `ferrotex.diagnostics.showCode`            | `boolean` | `true`  | Show error codes in diagnostic tooltips.                      |

</details>

<details>
<summary><strong>Editor, Hover & Image Paste Settings (Click to Expand)</strong></summary>

| Setting Key                            |   Type    |   Default   | Description                                                         |
| :------------------------------------- | :-------: | :---------: | :------------------------------------------------------------------ |
| `ferrotex.hover.enabled`               | `boolean` |   `true`    | Show rich hover tooltips for commands and packages.                 |
| `ferrotex.hover.citations`             | `boolean` |   `true`    | Extract and preview BibTeX metadata on hover.                       |
| `ferrotex.imagePaste.enabled`          | `boolean` |   `true`    | Enable clipboard image paste wizard on <kbd>Cmd</kbd>+<kbd>V</kbd>. |
| `ferrotex.imagePaste.defaultDirectory` | `string`  | `"figures"` | Relative destination folder for pasted images.                      |
| `ferrotex.format.enabled`              | `boolean` |   `true`    | Enable document formatting.                                         |
| `ferrotex.format.indentSize`           | `number`  |     `2`     | Number of spaces per indentation level.                             |

</details>

---

## ❓ Frequently Asked Questions (FAQ)

<details>
<summary><strong>Do I need to install MacTeX, MikTeX, or TeX Live?</strong></summary>
No! FerroTeX includes a built-in on-demand compiler engine (Tectonic) that downloads only the specific packages required by your document on the fly. You can start writing on a brand-new computer in seconds.
</details>

<details>
<summary><strong>Can I still use my existing pdflatex, xelatex, or latexmk setup?</strong></summary>
Yes. If you have an established custom workflow or institutional TeX Live setup, simply change `"ferrotex.build.engine": "pdflatex"` (or `"latexmk"` / `"xelatex"` / `"lualatex"`) in your settings.
</details>

<details>
<summary><strong>Does SyncTeX work in both directions?</strong></summary>
Yes. Press <kbd>Cmd</kbd>+<kbd>Alt</kbd>+<kbd>J</kbd> in your LaTeX source to jump forward to the corresponding position in the PDF preview. In the PDF preview, hold <kbd>Ctrl</kbd> or <kbd>Cmd</kbd> and click on any text to jump back to the exact source line.
</details>

<details>
<summary><strong>How does FerroTeX handle multi-file projects?</strong></summary>
FerroTeX automatically recognizes `% !TEX root = main.tex` magic comments at the top of sub-files, or you can specify `"ferrotex.build.mainFile": "main.tex"` in your workspace settings.
</details>

---

## 🤝 Community & Support

FerroTeX is open-source software built with ❤️ in Rust for researchers, academics, and authors worldwide.

- 🐛 **Report an Issue**: [GitHub Issues](https://github.com/jxoesneon/FerroTex/issues)
- 💬 **Ask a Question**: [GitHub Discussions](https://github.com/jxoesneon/FerroTex/discussions)
- 💖 **Sponsor the Project**: [Ko-fi](https://ko-fi.com/jxoesneon)
- 📜 **License**: [Apache 2.0](LICENSE.md)
