# ADR 0006: Tectonic Stepping I/O Provider for DAP

- **Status:** Accepted
- **Date:** 2026-01-02

## Context

The Tectonic engine provides a high-level `ProcessingSession` but does not expose granular hooks to "pause" execution in the middle of a compilation pass (e.g., when a macro expansion happens or a file is opened).

Standard Debug Adapter Protocol (DAP) features like **Stepping** and **Variables Inspection** require the ability to halt the engine's execution safely without killing the process.

## Decision

Implement a custom **`SteppingIoProvider`** wrapper for Tectonic's `IoProvider` trait.

Key implementation details:

- **Interception**: Every internal I/O request from the Tectonic/XeTeX engine (reading a `.tex` file, loading a font, writing to a buffer) is routed through this provider.
- **Blocking Mechanism**: The provider checks a thread-safe control channel managed by the `DebugAdapter`. If a "Stop" command is active, the I/O request blocks until a "Continue" signal is received.
- **Event Forwarding**: The provider emits DAP `stopped` and `output` events before and after blocking, providing the debugger with context on the engine's current activity.

## Alternatives Considered

- **Signal-based Halting (SIGSTOP/SIGCONT)**:
  - rejected: Too crude; doesn't allow for fine-grained state extraction and creates platform-specific stability issues.
- **Modifying Tectonic/XeTeX source**:
  - rejected: High maintenance burden and difficult to sync with upstream security patches.

## Consequences

- We achieve "Premium" stepping capabilities without modifying the core C/C++ engine.
- Every compilation pass becomes "observable" by nature of its I/O pattern.
- Threading complexity increases as I/O happens on a different thread than the DAP message loop.
