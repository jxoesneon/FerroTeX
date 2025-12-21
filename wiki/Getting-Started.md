# Getting Started with FerroTeX

## Prerequisites

- **Rust** (stable toolchain)
- **Node.js** (LTS)
- **TeX Live**, MiKTeX, or Tectonic

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/jxoesneon/FerroTeX.git
cd FerroTeX
```

### 2. Build the Server

```bash
cargo build --release -p ferrotexd
```

### 3. Install the VS Code Extension

```bash
cd editors/vscode
npm install
npm run compile
```

### 4. Run the Extension

Open VS Code and use the "Run Extension" debug task.

## Next Steps

- [LSP Features](https://jxoesneon.github.io/FerroTeX/spec/lsp.html)
- [Configuration](https://jxoesneon.github.io/FerroTeX/spec/configuration.html)
