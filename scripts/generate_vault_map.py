#!/usr/bin/env python3
"""
FerroTex Comprehensive Knowledge Graph Generator for Obsidian Vault
Maps all repository files into Obsidian Flavored Markdown notes with bidirectional wikilinks,
AST symbol extraction, dependency graph resolution, and category tagging.
"""

import os
import re
import json
import subprocess
from pathlib import Path

REPO_ROOT = Path('/Users/mey/FerroTex')
VAULT_ROOT = Path('/Users/mey/FerroTex/vault')

IGNORE_EXTENSIONS = {'.png', '.svg', '.vsix', '.map', '.ico', '.tar.gz', '.log'}
IGNORE_PATHS = {'editors/vscode/out', '.git', 'target', 'node_modules', 'vault'}

def get_tracked_files():
    output = subprocess.check_output(['git', 'ls-files'], cwd=REPO_ROOT).decode('utf-8')
    files = []
    for line in output.splitlines():
        line = line.strip()
        if not line:
            continue
        if any(line.endswith(ext) for ext in IGNORE_EXTENSIONS):
            continue
        if any(part in line.split('/') for part in ['out', '.git', 'target', 'node_modules', 'vault']):
            continue
        files.append(line)
    return sorted(files)

def classify_file(rel_path):
    parts = rel_path.split('/')
    ext = Path(rel_path).suffix.lower()
    name = Path(rel_path).name

    crate = "none"
    layer = "general"
    file_type = "file"
    lang = "text"

    if rel_path.startswith('crates/'):
        crate = parts[1]
        layer = crate.replace('ferrotex-', '').replace('ferrotexd', 'daemon')
        if ext == '.rs':
            file_type = "source-code"
            lang = "rust"
        elif name == 'Cargo.toml':
            file_type = "manifest"
            lang = "toml"
        elif ext == '.md':
            file_type = "doc"
            lang = "markdown"
    elif rel_path.startswith('editors/vscode'):
        crate = "vscode-extension"
        layer = "editor"
        if ext in ('.ts', '.js'):
            file_type = "source-code"
            lang = "typescript" if ext == '.ts' else "javascript"
        elif name == 'package.json':
            file_type = "manifest"
            lang = "json"
        elif ext == '.md':
            file_type = "doc"
            lang = "markdown"
    elif rel_path.startswith('docs/adrs'):
        crate = "architecture"
        layer = "adr"
        file_type = "adr"
        lang = "markdown"
    elif rel_path.startswith('docs/spec'):
        crate = "specification"
        layer = "spec"
        file_type = "spec"
        lang = "markdown"
    elif rel_path.startswith('docs/architecture'):
        crate = "architecture"
        layer = "architecture"
        file_type = "architecture"
        lang = "markdown"
    elif rel_path.startswith('docs/'):
        crate = "documentation"
        layer = "docs"
        file_type = "doc"
        lang = "markdown"
    elif rel_path.startswith('scripts/'):
        crate = "tooling"
        layer = "scripts"
        file_type = "script"
        lang = "shell" if ext == '.sh' else "javascript" if ext in ('.js', '.mjs') else "python"
    elif rel_path.startswith('.github'):
        crate = "ci-cd"
        layer = "automation"
        file_type = "workflow"
        lang = "yaml"
    elif rel_path.startswith('fuzz'):
        crate = "fuzz"
        layer = "testing"
        file_type = "fuzz-target"
        lang = "rust"
    elif rel_path.startswith('wiki'):
        crate = "documentation"
        layer = "wiki"
        file_type = "doc"
        lang = "markdown"
    elif '/' not in rel_path:
        crate = "workspace-root"
        layer = "root"
        if name in ('Cargo.toml', 'Cargo.lock'):
            file_type = "manifest"
            lang = "toml"
        elif name in ('package.json', 'package-lock.json'):
            file_type = "manifest"
            lang = "json"
        elif ext == '.md':
            file_type = "doc"
            lang = "markdown"
        elif ext == '.yml' or ext == '.yaml':
            file_type = "config"
            lang = "yaml"

    return {
        "crate": crate,
        "layer": layer,
        "type": file_type,
        "lang": lang
    }

def extract_rust_symbols(content):
    symbols = []
    # Structs
    for m in re.finditer(r'pub(?:\([^)]+\))?\s+struct\s+([A-Za-z0-9_]+)', content):
        symbols.append(("struct", m.group(1)))
    # Enums
    for m in re.finditer(r'pub(?:\([^)]+\))?\s+enum\s+([A-Za-z0-9_]+)', content):
        symbols.append(("enum", m.group(1)))
    # Traits
    for m in re.finditer(r'pub(?:\([^)]+\))?\s+trait\s+([A-Za-z0-9_]+)', content):
        symbols.append(("trait", m.group(1)))
    # Type aliases
    for m in re.finditer(r'pub(?:\([^)]+\))?\s+type\s+([A-Za-z0-9_]+)', content):
        symbols.append(("type", m.group(1)))
    # Functions
    for m in re.finditer(r'pub(?:\([^)]+\))?\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)', content):
        symbols.append(("fn", m.group(1)))
    # Constants
    for m in re.finditer(r'pub(?:\([^)]+\))?\s+const\s+([A-Za-z0-9_]+)', content):
        symbols.append(("const", m.group(1)))
    return symbols

def extract_ts_symbols(content):
    symbols = []
    for m in re.finditer(r'export\s+(?:default\s+)?(?:async\s+)?function\s+([A-Za-z0-9_]+)', content):
        symbols.append(("function", m.group(1)))
    for m in re.finditer(r'export\s+class\s+([A-Za-z0-9_]+)', content):
        symbols.append(("class", m.group(1)))
    for m in re.finditer(r'export\s+interface\s+([A-Za-z0-9_]+)', content):
        symbols.append(("interface", m.group(1)))
    for m in re.finditer(r'export\s+type\s+([A-Za-z0-9_]+)', content):
        symbols.append(("type", m.group(1)))
    for m in re.finditer(r'export\s+const\s+([A-Za-z0-9_]+)', content):
        symbols.append(("const", m.group(1)))
    return symbols

def extract_rust_uses(content):
    uses = set()
    for m in re.finditer(r'use\s+([^;]+);', content):
        statement = m.group(1).strip()
        uses.add(statement)
    return uses

def extract_headings(content):
    headings = []
    for line in content.splitlines():
        if line.startswith('#'):
            h = line.lstrip('#').strip()
            if h:
                headings.append(h)
    return headings

def resolve_links(rel_path, content, info, all_files):
    links = set()
    
    # Link to parent crate manifest & README if in crate
    if rel_path.startswith('crates/'):
        crate_name = rel_path.split('/')[1]
        c_manifest = f'crates/{crate_name}/Cargo.toml'
        c_readme = f'crates/{crate_name}/README.md'
        if c_manifest in all_files and c_manifest != rel_path:
            links.add(c_manifest)
        if c_readme in all_files and c_readme != rel_path:
            links.add(c_readme)
        
        # Link to crate lib.rs / main.rs
        c_lib = f'crates/{crate_name}/src/lib.rs'
        c_main = f'crates/{crate_name}/src/main.rs'
        if c_lib in all_files and c_lib != rel_path:
            links.add(c_lib)
        if c_main in all_files and c_main != rel_path:
            links.add(c_main)

    # Link to VS Code package.json
    if rel_path.startswith('editors/vscode') and rel_path != 'editors/vscode/package.json':
        links.add('editors/vscode/package.json')
        if rel_path != 'editors/vscode/src/extension.ts':
            links.add('editors/vscode/src/extension.ts')

    # Link to Root Cargo.toml
    if rel_path != 'Cargo.toml':
        if info['crate'].startswith('ferrotex') or info['crate'] == 'workspace-root':
            links.add('Cargo.toml')

    # Detect inter-file references
    for other in all_files:
        if other == rel_path:
            continue
        basename = Path(other).stem
        if len(basename) > 4 and (basename in content or other in content):
            links.add(other)

    # Architectural Spec bindings
    if 'lsp' in rel_path.lower() or 'daemon' in rel_path.lower() or 'ferrotexd' in rel_path:
        links.add('docs/spec/lsp.md')
        links.add('docs/architecture/overview.md')
    if 'syntax' in rel_path.lower() or 'parser' in rel_path.lower() or 'lexer' in rel_path.lower():
        links.add('docs/spec/incremental-analysis.md')
        links.add('docs/adrs/0004-streaming-incremental-parsing.md')
    if 'log' in rel_path.lower():
        links.add('docs/spec/log-event-ir.md')
        links.add('docs/adrs/0001-typed-event-ir.md')
    if 'build' in rel_path.lower() or 'tectonic' in rel_path.lower() or 'latexmk' in rel_path:
        links.add('docs/spec/build-orchestration.md')
        links.add('docs/adrs/0005-universal-build-dag.md')
    if 'dap' in rel_path.lower() or 'debug' in rel_path.lower():
        links.add('docs/spec/dap.md')
    if 'math' in rel_path.lower():
        links.add('docs/spec/math-verification.md')
    if 'package' in rel_path.lower() or 'ctan' in rel_path.lower():
        links.add('docs/spec/tex-path-resolution.md')

    # Always link to FerroTex Overview
    links.add('FerroTex Overview')

    # Filter links to ensure they exist
    valid_links = set()
    for l in links:
        if l == 'FerroTex Overview' or l in all_files:
            valid_links.add(l)
    return sorted(valid_links)

def generate_note(rel_path, all_files):
    full_path = REPO_ROOT / rel_path
    if not full_path.exists() or full_path.is_dir():
        return None
    try:
        content = full_path.read_text(encoding='utf-8', errors='replace')
    except Exception as e:
        return None

    info = classify_file(rel_path)
    loc = len(content.splitlines())
    symbols = []
    if info['lang'] == 'rust':
        symbols = extract_rust_symbols(content)
    elif info['lang'] in ('typescript', 'javascript'):
        symbols = extract_ts_symbols(content)

    headings = extract_headings(content)
    links = resolve_links(rel_path, content, info, all_files)

    # First non-empty line or markdown heading as summary
    first_few_lines = [l.strip() for l in content.splitlines() if l.strip() and not l.startswith('//') and not l.startswith('/*')][:3]
    summary_text = f"{info['type'].capitalize()} file ({info['lang']}) in {info['crate']} subsystem ({loc} lines)."
    if headings:
        summary_text = f"{headings[0]} — {summary_text}"

    # Build Frontmatter
    tags = [f"type/{info['type']}", f"lang/{info['lang']}", f"layer/{info['layer']}"]
    if info['crate'] != 'none':
        tags.append(f"crate/{info['crate']}")

    fm = f"""---
title: "{rel_path}"
type: {info['type']}
language: {info['lang']}
crate: {info['crate']}
layer: {info['layer']}
lines: {loc}
tags:
"""
    for t in sorted(set(tags)):
        fm += f"  - {t}\n"
    fm += f'summary: "{summary_text}"\n---\n\n'

    # Note Body
    body = f"# `{rel_path}`\n\n"
    body += f"> [!info] File Overview\n"
    body += f"> - **Crate/Domain**: `{info['crate']}` ({info['layer']})\n"
    body += f"> - **Language/Type**: `{info['lang']}` ({info['type']})\n"
    body += f"> - **Lines of Code**: `{loc}`\n"
    body += f"> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]\n\n"

    # Purpose section
    body += "## Purpose & System Role\n"
    body += f"Part of the **{info['crate']}** module within the `{info['layer']}` layer of the FerroTex architecture. "
    if info['lang'] == 'rust':
        body += "Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.\n\n"
    elif info['lang'] in ('typescript', 'javascript'):
        body += "Implements client-side editor capabilities, Language Client connection management, and UI commands.\n\n"
    elif info['lang'] == 'markdown':
        body += "Defines formal specifications, architectural decision records, or project documentation.\n\n"
    else:
        body += "Contains project configuration, workflows, or developer tooling.\n\n"

    # Symbols section
    if symbols:
        body += "## Key Symbols & Interfaces\n\n"
        body += "| Kind | Identifier |\n|---|---|\n"
        for kind, name in symbols[:25]:
            body += f"| `{kind}` | `{name}` |\n"
        if len(symbols) > 25:
            body += f"| ... | *({len(symbols)-25} additional symbols)* |\n"
        body += "\n"

    # Headings if markdown
    elif headings and len(headings) > 1:
        body += "## Document Sections\n\n"
        for h in headings[:15]:
            body += f"- {h}\n"
        body += "\n"

    # Connected Nodes / Dependencies
    body += "## Graph Links & Dependencies\n\n"
    for l in links:
        if l == 'FerroTex Overview':
            body += f"- [[FerroTex Overview|Central Knowledge Hub]]\n"
        else:
            display = Path(l).name
            body += f"- [[{l}|{display}]] (`{l}`)\n"
    body += "\n"

    return fm + body

def main():
    all_files = get_tracked_files()
    print(f"Total candidate files to map: {len(all_files)}")

    mapped_count = 0
    preserved_subagent_count = 0

    for rel_path in all_files:
        target_note = VAULT_ROOT / f"{rel_path}.md"
        # If the file already exists and was authored by a deep subagent (has rich sections like Key Manifest Sections or Detailed breakdown), keep it!
        if target_note.exists():
            content = target_note.read_text(encoding='utf-8', errors='replace')
            if "Key Manifest Sections" in content or "Error Recovery Strategy" in content or "Invariants & Principles" in content:
                preserved_subagent_count += 1
                continue

        note_content = generate_note(rel_path, all_files)
        if note_content:
            target_note.parent.mkdir(parents=True, exist_ok=True)
            target_note.write_text(note_content, encoding='utf-8')
            mapped_count += 1

    print(f"Successfully mapped {mapped_count} files into {VAULT_ROOT} (preserved {preserved_subagent_count} enriched subagent notes).")

if __name__ == '__main__':
    main()
