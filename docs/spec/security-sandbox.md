# Safe-TeX Sandbox Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

The Safe-TeX Sandbox provides a secure environment for compiling LaTeX documents, particularly in sensitive research or multi-user environments. It addresses the inherent security risks of TeX engines (e.g., arbitrary file access, `shell-escape` vulnerabilities) by implementing a strict capabilities-based security model.

## Core Principles

- **Zero-Trust Defaults**: By default, no external side-effects are permitted. All capabilities must be explicitly granted.
- **Project Isolation**: Compilations are confined to the project root and designated temporary/output directories.
- **Least Privilege**: The TeX engine and associated tools only receive the minimum permissions necessary for the current task.
- **Explicit Consent**: High-risk operations (like full `shell-escape`) require explicit user confirmation or workspace trust.

## Capabilities-Based Security Model

Permissions are modeled as **Capabilities** granted to a specific build context.

### 1. File System Capabilities

- `fs:read`: Permission to read files.
- `fs:write`: Permission to write files.
- `fs:list`: Permission to list directory contents.

_Note: These are mediated by the [Virtual File System (VFS)](vfs.md)._

### 2. Execution Capabilities (`shell-escape`)

- `exec:restricted`: Permission to run a pre-defined allowlist of safe commands (e.g., `gnuplot`, `pygmentize`).
- `exec:unrestricted`: Permission to run arbitrary commands (Full `shell-escape`).

### 3. Network Capabilities

- `net:connect`: Permission to establish outbound network connections (e.g., for package downloading or remote resource fetching).

### 4. Environment Capabilities

- `env:read`: Permission to read specific environment variables.

## Granular Permissions for `shell-escape`

FerroTeX defines three levels of `shell-escape` security:

| Level             | Argument            | Description                                                                 |
| :---------------- | :------------------ | :-------------------------------------------------------------------------- |
| **Off** (Default) | `-no-shell-escape`  | No external command execution permitted.                                    |
| **Restricted**    | `-shell-restricted` | Only allowlisted, known-safe commands can be executed.                      |
| **On**            | `-shell-escape`     | Full execution permitted. Requires explicit "Trust Workspace" confirmation. |

### Command Allowlist (`exec:restricted`)

The default allowlist includes common LaTeX-adjacent utilities:

- `bibtex`, `biber`
- `makeindex`, `xindy`
- `kpsewhich`
- `gnuplot` (if used by `pgfplots`)
- `pygmentize` (if used by `minted`)

## Zero-Trust Implementation

1. **Environment Scrubbing**: Before spawning any process, the runner MUST scrub the environment, leaving only a minimal set of safe variables (e.g., `PATH` containing only the TeX distribution and allowlisted tools).
2. **Path Sanitization**: All paths passed to the engine MUST be sanitized and resolved through the VFS.
3. **Process Isolation**: Where supported by the OS, processes SHOULD be run in a low-privilege service account or a containerized environment.

## User Interaction

- **Permission Prompts**: When a document requires a capability not currently granted, FerroTeX SHOULD prompt the user with a clear explanation of the risk.
- **Workspace Trust**: Users can designate a workspace as "Trusted" to persist capability grants.

## Interactions

- **Virtual File System**: The sandbox relies on the [VFS](vfs.md) for file-level enforcement.
- **Engine Adapters**: Adapters MUST implement the capability checks before spawning processes.
- **Configuration**: Permissions are managed via `ferrotex.security.*` settings.
