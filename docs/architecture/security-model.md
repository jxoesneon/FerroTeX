---
layout: default
title: Security Model
parent: Architecture
nav_order: 3
---

# Security Model and Threat Analysis

FerroTeX is designed for high-security research environments where TeX documents may contain or execute untrusted macros. This document outlines the security architecture and threat model.

## Security Stance

FerroTeX adopts a **Zero-Trust** stance toward TeX engine execution. Unlike traditional distributions that allow arbitrary filesystem and process access via `shell-escape`, FerroTeX treats the TeX engine as an untrusted guest process.

## Architecture of Isolation

### 1. Capabilities-Based Security

Permissions are not global but granted per build context.

- **`fs`**: Mediated by the [Virtual File System (VFS)](../spec/vfs.md).
- **`exec`**: Mediated by a command allowlist and process-level isolation.
- **`net`**: Denied by default; requires explicit workspace trust.

### 2. Virtual File System (VFS)

The VFS layer (`ferrotex-vfs`) acts as a secure proxy for all file I/O.

- **Root Jail**: The TeX engine only sees virtual paths (e.g., `/project`, `/texlive`).
- **Traversal Prevention**: Hardened path resolution prevents `../` escaping.
- **Symlink Scrubbing**: External symlinks are treated as opaque or invalid.

### 3. Environment Sanitization

Before spawning a runner, FerroTeX scrubs the environment block:

- **Clean PATH**: Only contains the specific TeX distribution and allowlisted tools.
- **Removal of Sensitive Vars**: Variables like `SECRET_KEY`, `KUBECONFIG`, or `SSH_AUTH_SOCK` are stripped.

## Threat Model

| Threat Actor                 | Vector                                                           | Mitigation                                                                          |
| :--------------------------- | :--------------------------------------------------------------- | :---------------------------------------------------------------------------------- |
| **Malicious Package**        | A CTAN package using `\write18` to exfiltrate `~/.ssh/id_rsa`.   | **VFS Root Isolation**: The process cannot see the host's home directory.           |
| **Document-Embedded Script** | A `.tex` file using `shell-escape` to run a reverse shell.       | **Restricted Shell Allowlist**: Only known utilities (e.g., `gnuplot`) are allowed. |
| **Path Traversal**           | Using `\input{../../etc/passwd}` to read sensitive system files. | **VFS Path Validation**: Resolve all paths against the virtual root.                |
| **Environment Leak**         | TeX engine reading environment variables to leak secrets.        | **Environment Scrubbing**: Only a minimal, safe environment is provided.            |

## Compliance and Auditing

- **Audit Logs**: Every capability request and VFS violation is logged with provenance.
- **Reproducibility**: Docker-based CI ensures that security constraints are identical across environments.

## Future Hardening

- **User-Namespace Isolation (Linux)**: Running the engine in a dedicated PID and Mount namespace.
- **Wasm-Based Execution**: Future aspirations to run the TeX stomach entirely within a WebAssembly sandbox for total memory isolation.
