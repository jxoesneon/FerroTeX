---
title: ".cargo/config.toml.example"
type: file
language: text
crate: none
layer: general
lines: 18
tags:
  - lang/text
  - layer/general
  - type/file
summary: "macOS Build Configuration Template — File file (text) in none subsystem (18 lines)."
---

# `.cargo/config.toml.example`

> [!info] File Overview
> - **Crate/Domain**: `none` (general)
> - **Language/Type**: `text` (file)
> - **Lines of Code**: `18`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **none** module within the `general` layer of the FerroTex architecture. Contains project configuration, workflows, or developer tooling.

## Document Sections

- macOS Build Configuration Template
- Copy this file to .cargo/config.toml and update paths to match your system
- Update VERSION to match your Homebrew harfbuzz version
- Find with: brew list --versions harfbuzz
- Update to ICU version 76 or 77 (not 78, which requires additional C++17 fixes)
- Switch versions with: brew unlink icu4c@78 && brew link --force icu4c@76

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]

