---
layout: default
title: Formal Backends
parent: Specifications
nav_order: 27
---

# Formal Backends Integration Specification

## Status

- **Type:** Research / Exploratory
- **Stability:** Research / Exploratory (v0.20.0 Target)

## Purpose

The Formal Backends pillar in FerroTeX bridges the gap between scientific typesetting and rigorous mathematical verification. This is achieved by integrating FerroTeX with mature formal proof checkers like **Lean 4** and **Coq**, enabling users to certify that the mathematical claims made in their LaTeX documents are formally correct.

## Strategic Objectives

- **Automated Document Certification**: Automatically check that equations in the document correspond to theorems in a formal library.
- **Proof-State Reflection**: Display the internal state of a formal proof engine directly within the editor alongside the source math.
- **Zero-Friction Translation**: Convert LaTeX-style mathematical syntax into formal language counterparts without manual rewrite.

## Supported Formal Backends

### 1. Lean 4 (Primary Integration)

Lean 4 is the primary target for formal verification due to its rich mathematical library (`mathlib4`) and modern, extensible architecture.

- **Integration Layer**: Uses the Lean LSP or a custom RPC bridge to communicate with a Lean server.
- **Target Libraries**: Verification against `mathlib4` definitions for topology, analysis, and algebra.

### 2. Coq (Alternative Backend)

Coq is supported for specialized fields like computer science and formal software verification.

- **Integration Layer**: Communicates with `coqtop` or `vscoq` protocol extensions.
- **Target Libraries**: Verification using standard libraries like `CompCert` or `Coquelicot`.

## Verification Pipeline

The integration follows a structured pipeline:

1. **Extraction**: The FerroTeX server identifies "certifiable" regions in the LaTeX source (e.g., theorems, lemmas, or calculations).
2. **Translation**: The extracted Math IR is translated into the formal language of the target backend.
3. **Goal Dispatch**: The translated code is sent to the formal backend as a verification request.
4. **Result Propagation**: Results (Pass/Fail/Undecided) and proof states are returned to the LSP client as diagnostics or hover information.

## Workflow: `certified-math` Environment

Users can opt-in to formal verification using a specialized LaTeX environment (provided by the FerroTeX core package):

```latex
\begin{certified-math}[backend=lean, theorem=fundamental_theorem_of_calculus]
    \int_a^b f'(x) dx = f(b) - f(a)
\end{certified-math}
```

In this environment, FerroTeX will:

- Map the LaTeX symbols to their Lean counterparts.
- Attempt to verify that the equation holds given the specified theorem in the Lean library.
- Emit a `FBT-001: Certification Passed` diagnostic upon success.

## Error Reporting & Feedback

| Diagnostic | Level                 | Description                                                             |
| :--------- | :-------------------- | :---------------------------------------------------------------------- |
| FBT-001    | Certification Passed  | Successful verification by the formal backend.                          |
| FBT-002    | Certification Failed  | The backend could not verify the claim or found a contradiction.        |
| FBT-003    | Backend Unavailable   | Specified formal checker (e.g., Lean) is not installed or reachable.    |
| FBT-004    | Translation Ambiguity | Math IR cannot be uniquely translated into the formal backend's syntax. |

## Performance and Resource Management

Formal verification can be computationally expensive. FerroTeX manages this by:

- **Lazy Verification**: Only triggering formal checks upon document save or explicit user request.
- **Result Caching**: Storing verification tokens indexed by a hash of the math content and environment parameters.
- **Isolated Execution**: Running backends in lower-priority background processes to ensure editor responsiveness.
