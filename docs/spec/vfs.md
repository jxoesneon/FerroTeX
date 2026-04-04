# Virtual File System (VFS) Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

The FerroTeX Virtual File System (VFS) provides an abstraction layer over the physical file system to ensure document compilation is isolated within the project root. It prevents unauthorized file access, simplifies path resolution across different environments, and enables secure, reproducible builds.

## Core Concepts

- **Isolation**: By default, the VFS only allows access to files within the project root and its subdirectories.
- **Path Mapping**: The VFS translates between virtual paths (used by the engine and LSP) and physical paths (on the host file system).
- **Mount Points**: External resources, such as TeX Live distribution trees, can be "mounted" as read-only virtual directories.
- **Project Root**: The root directory of the current TeX project, which serves as the base for the virtual namespace.

## Security Invariants

- **Root Isolation**: No operation SHOULD be able to read or write files outside the project root unless explicitly permitted via a mount point or capability.
- **Directory Traversal Prevention**: Path components like `..` MUST be resolved and validated to ensure they do not escape the virtual root.
- **Symlink Safety**: Symbolic links originating from within the project root MUST only be followed if they point to a location within the virtual namespace. Links pointing to external locations SHOULD be ignored or treated as broken.
- **Write Restriction**: By default, the VFS SHOULD enforce read-only access to all files within the project root, except for a designated `output` directory and a `tmp` directory.

## VFS Operations

The VFS layer intercepts all file-related operations from FerroTeX components:

| Operation           | VFS Behavior                                                                           |
| :------------------ | :------------------------------------------------------------------------------------- |
| `read(path)`        | Resolve virtual path, check `fs:read` capability, and return file contents.            |
| `write(path, data)` | Resolve virtual path, check `fs:write` capability, and write to the physical location. |
| `list(path)`        | Resolve virtual path, check `fs:list` capability, and return directory listing.        |
| `stat(path)`        | Resolve virtual path and return metadata (size, modified time).                        |

## Mount Points

The VFS supports mounting physical directories into the virtual namespace:

- **`/project`**: Points to the workspace root (Read/Write).
- **`/texlive`**: Points to the host's TeX distribution (Read-Only).
- **`/tmp`**: Points to a secure, per-session temporary directory (Read/Write).

## Path Resolution Logic

1. **Absolute Virtual Path**: If a path starts with `/`, it is resolved directly from the virtual root.
2. **Relative Path**: If a path does not start with `/`, it is resolved relative to the virtual path of the "current file" or the project root.
3. **External Resolution**: If a path cannot be resolved within `/project`, the VFS MAY check mounted resources (like `/texlive`) if the appropriate resolution mode is enabled (see [TeX Path Resolution](tex-path-resolution.md)).

## Implementation Strategy

- **Rust Crate**: The VFS SHOULD be implemented as a dedicated Rust crate (`ferrotex-vfs`) to be used by both the LSP server and engine runners.
- **Integration**:
  - **LSP**: All file lookups for `documentLink`, `definition`, and `references` MUST go through the VFS.
  - **Engine Adapters**: Adapters SHOULD pass virtualized paths to the TeX engine or use OS-level mechanisms (like `chroot` or `namespaces` on Linux) to enforce VFS constraints.

## Interactions

- **Safe-TeX Sandbox**: The sandbox provides the [capability grants](security-sandbox.md) that the VFS enforces.
- **TeX Path Resolution**: Path resolution logic is detailed in [tex-path-resolution.md](tex-path-resolution.md).
- **Engine Adapters**: Adapters must respect VFS path mappings.
