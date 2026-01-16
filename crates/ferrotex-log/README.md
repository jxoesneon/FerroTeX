# `ferrotex-log`

A high-performance, zero-allocation parser designed to transform TeX engine output into structured, actionable event streams.

## 🏗️ The Problem

Traditional TeX engine output (typically written to `.log` files or `stdout`) is monolithic and notoriously difficult for IDEs to parse accurately. `ferrotex-log` solves this by treating the output as a typed event stream.

## 💎 Features

- **Zero-Allocation Parsing**: Leverages streaming techniques to process massive logs (>50MB/s) without excessive memory overhead.
- **Typed Event Stream**: Converts cryptic text patterns into structured `LogEvent` objects (e.g., `Error`, `Warning`, `FileOpen`, `FontLoad`).
- **Real-Time Watching**: Supports streaming updates as the compiler writes them, enabling instant diagnostic feedback in the editor.

## 🚀 Capabilities

- **Provenance Tracking**: Identifies which file and line an error _actually_ refers to, bypassing several common TeX log ambiguity traps.
- **Fault Tolerance**: Safely handles partial or corrupted log files during active builds.

---

_Part of the FerroTeX Scientific Compiler Platform._
