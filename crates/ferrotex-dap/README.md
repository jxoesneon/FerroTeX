# `ferrotex-dap`

Bringing industry-standard observability to TeX through the **Debug Adapter Protocol (DAP)**.

## 🏗️ Architecture

`ferrotex-dap` acts as a bridge between modern IDE debuggers and the internal state of TeX engines.

### Components

- **Debug Session**: Handles the DAP message loop (JSON-RPC over stdin/stdout).
- **Debug Adapter**: Abstract trait for controlling different backend engines.
- **Tectonic Shim**: A deep integration with the Tectonic engine that allows for pass-level stepping and register inspection.

## 💎 Capabilities

- **Step-by-Step Execution**: Pause compilation at specific TeX passes or file inclusions.
- **State Inspection**: View live values of TeX registers (`\count`, `\dimen`, `\skip`) in the IDE's Variables view.
- **Macro Shadowing**: Inspect macro definitions as they change during the expansion process.

## 🚀 Engine Shims

Currently supports:

- **Tectonic**: High-fidelity native integration.
- **Mock**: For testing protocol compliance and UI frontends.

---

_Part of the FerroTeX Scientific Compiler Platform._
