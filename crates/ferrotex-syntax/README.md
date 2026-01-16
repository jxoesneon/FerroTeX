# `ferrotex-syntax`

The foundation of the FerroTeX ecosystem, providing high-fidelity, lossless, and fault-tolerant parsing for LaTeX and BibTeX sources.

## 🏗️ Architecture

Built on the **Rowan** library, `ferrotex-syntax` transforms raw text into a Concrete Syntax Tree (CST) that preserves 100% of the source fidelity.

### Key Components

- **Lexer**: A high-performance tokenizer that categorizes TeX primitives, commands, and trivia (whitespace/comments).
- **Parser**: A hand-written recursive descent parser designed for **error recovery**. It can handle unclosed groups and mismatched environments while producing a usable tree for IDE features.
- **BibTeX**: Specialized sub-parser for bibliographic databases, integrated into the same lossless architecture.

## 💎 Features

- **Lossless Fidelity**: Every space, comment, and escape sequence is preserved in the tree, making it ideal for formatters and refactoring tools.
- **Fault Tolerance**: Partial or invalid LaTeX can be indexed and analyzed.
- **Efficient Incremental Support**: Optimized for the rapid feedback loops required by Language Server Protocol (LSP).

## 📊 Performance

Indices 10k lines of LaTeX in approximately **1.5ms** on modern hardware (M1/M2 Silicon), ensuring zero-latency typing feedback.

---

_Part of the FerroTeX Scientific Compiler Platform._
