# Marketplace Overhaul Proposal (v0.16.0)

To establish **FerroTeX** as a premium, "Gold standard" extension, we need to completely overhaul the [Open VSX listing](https://open-vsx.org/extension/ferrotex/ferrotex).

## 1. Visual Identity (The "Wow" Factor)

The current text-only README is functional but forgettable. We will introduce:

### A. Hero Banner (Banner.png)

A sleek, dark-mode banner at the very top.

- **Concept**: Glassmorphic abstract "Engine" visualization with the FerroTeX logo.
- _Attributes_: Premium, Fast, Intelligent.

### B. Feature GIFs (Show, Don't Tell)

Instead of bullet points, we will use optimized GIFs to demonstrate the "Intelligence" features:

1.  **"Smart Completion"**: Showing `\usepackage{tikz}` -> `\node` completion.
2.  **"Magic Comments"**: Typing `%!TEX root` and seeing the build target change.
3.  **"Snippet Pack"**: Expanding `@g` to `\gamma` and `pmat` to a matrix.

## 2. Structural Changes

We will restructure `editors/vscode/README.md` into a marketing-first layout:

### **Section 1: The Pitch**

> "Stop fighting with your LaTeX editor. FerroTeX brings the intelligence of a modern IDE to your scientific writing."

### **Section 2: Social Proof (Badges)**

A row of shields.io badges:

- `[Marketplace Version]`
- `[Installs]`
- `[Build Status]`
- `[Code Coverage]`
- `[License: Apache 2.0]`

### **Section 3: Feature Highlights (The "Grid")**

A table or grid layout showcasing the pillars:

- ⚡ **Performance**: "Bundled for instant startup."
- 🧠 **Intelligence**: "Dynamic completion engine."
- 👁️ **Observability**: "Real-time log streaming." (Coming in v0.16.0)

## 3. "Observability" Branding (v0.16.0 Specific)

Since v0.16.0 is the "Observability" update, the store page should highlight the **Real-Time Log Store**.

- **Mockup**: A split-screen showing a PDF on the right and a _clean, readable_ error log on the left (replacing the cryptic LaTeX console).

## 4. Proposed Asset List

- `raw/images/hero_v1.png`
- `raw/images/demo_completion.gif`
- `raw/images/demo_snippets.gif`
- `raw/images/icon_premium.png` (Update extension icon if needed)

## Implementation Plan

1.  **Design Assets**: Create high-fidelity screenshots/GIFs.
2.  **Rewrite README**: Implement the new markdown structure.
3.  **Bundle Assets**: Ensure images are included in the `.vsix` (via `vscodeignore` exceptions for the `images` folder) so they render on Open VSX.
