# FerroTeX Public API Reference

This document provides a comprehensive reference for the public API surface of all core FerroTeX crates.

## Table of Contents

- [Overview](#overview)
- [Crate Dependency Graph](#crate-dependency-graph)
- [ferrotex-syntax](#ferrotex-syntax)
- [ferrotex-analysis](#ferrotex-analysis)
- [ferrotex-core](#ferrotex-core)
- [ferrotex-dap](#ferrotex-dap)
- [ferrotex-build](#ferrotex-build)
- [ferrotex-package](#ferrotex-package)
- [ferrotex-log](#ferrotex-log)
- [ferrotex-math-semantics](#ferrotex-math-semantics)

---

## Overview

FerroTeX is a modular, research-grade LaTeX platform built with Rust. The architecture follows a layered approach:

1. **Syntax Layer** (`ferrotex-syntax`): Provides fault-tolerant parsing of LaTeX source code
2. **Log Analysis Layer** (`ferrotex-log`): Streaming parser for LaTeX engine logs
3. **Analysis Layer** (`ferrotex-analysis`): Abstract interpretation for macro analysis
4. **Core Layer** (`ferrotex-core`): Package management and validation utilities
5. **Build Layer** (`ferrotex-build`): Content-addressable build system
6. **Debug Layer** (`ferrotex-dap`): Debug Adapter Protocol implementation
7. **Package Layer** (`ferrotex-package`): LaTeX package indexing and metadata
8. **Math Semantics** (`ferrotex-math-semantics`): Formal semantics for math environments

---

## Crate Dependency Graph

```
                          ┌─────────────────┐
                          │   ferrotex-cli  │
                          └────────┬────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
        ▼                          ▼                          ▼
┌───────────────┐        ┌─────────────────┐        ┌─────────────────┐
│ ferrotex-dap  │        │ferrotex-analysis│        │ ferrotex-build  │
└────────┬──────┘        └────────┬────────┘        └────────┬────────┘
       │                         │                          │
       │                         ▼                          │
       │                ┌─────────────────┐                   │
       │                │ ferrotex-syntax │◄──────────────────┘
       │                └─────────────────┘                   │
       │                         ▲                          │
       │                         │                          ▼
       │                ┌─────────┴─────────┐        ┌─────────────────┐
       └───────────────►│ferrotex-math-semantics│   │   ferrotex-core   │
                        └─────────────────────────┘   └────────┬────────┘
                                                               │
                                                               ▼
                                                      ┌─────────────────┐
                                                      │ ferrotex-package│
                                                      └─────────────────┘
                                                               │
                                                               ▼
                                                      ┌─────────────────┐
                                                      │   ferrotex-log  │
                                                      └─────────────────┘
```

**Dependency Summary:**

| Crate | Dependencies |
|-------|-------------|
| `ferrotex-syntax` | `rowan` |
| `ferrotex-log` | `serde`, `serde_json`, `thiserror`, `regex` |
| `ferrotex-analysis` | `ferrotex-syntax`, `serde`, `thiserror` |
| `ferrotex-math-semantics` | `ferrotex-syntax`, `serde`, `thiserror`, `rowan` |
| `ferrotex-build` | `serde`, `anyhow`, `sha2`, `hex` |
| `ferrotex-package` | `serde`, `regex`, `walkdir`, `dirs`, `log`, `anyhow` |
| `ferrotex-core` | `ferrotex-build`, `tokio`, `anyhow`, `serde`, `which`, `once_cell`, `async-trait` |
| `ferrotex-dap` | `ferrotex-build`, `serde`, `anyhow`, `log` (optional: `tectonic`) |

---

## ferrotex-syntax

Fault-tolerant LaTeX parser providing a lossless Concrete Syntax Tree (CST).

### Primary Entry Points

#### `parse(source: &str) -> ParseResult`

Parses LaTeX source code into a syntax tree.

```rust
use ferrotex_syntax::parse;

let source = r"\section{Introduction}";
let result = parse(source);
let root = result.syntax();
```

**Returns:** `ParseResult` containing the root `SyntaxNode` and any parse errors.

### Key Types

#### `SyntaxKind`

Enumeration of all token and node types in the LaTeX grammar.

```rust
pub enum SyntaxKind {
    // Tokens
    LBrace,      // Left brace `{`
    RBrace,      // Right brace `}`
    LBracket,    // Left bracket `[`
    RBracket,    // Right bracket `]`
    Command,     // LaTeX command (e.g., `\section`)
    Dollar,      // Dollar sign `$` for math
    Whitespace,  // Whitespace characters
    Comment,     // Comment starting with `%`
    Text,        // Regular text content
    Error,       // Lexer error token

    // Composite Nodes
    Root,              // Root of the syntax tree
    Group,             // Braced group `{ ... }`
    Environment,       // `\begin{...} ... \end{...}`
    Section,           // Section command
    Include,           // `\input`, `\include`
    LabelDefinition,   // `\label{...}`
    LabelReference,    // `\ref{...}`
    Citation,          // `\cite{...}`
    Bibliography,      // `\bibliography{...}`
    Eof,               // End of file
}
```

#### `SyntaxNode`

A node in the Concrete Syntax Tree. Provides traversal and inspection methods.

```rust
pub type SyntaxNode = rowan::SyntaxNode<FerroTexLanguage>;

// Example usage
let result = parse(source);
let root = result.syntax();

for child in root.children() {
    println!("{:?} at {:?}", child.kind(), child.text_range());
}
```

#### `SyntaxToken`

A terminal token (leaf) in the CST.

```rust
pub type SyntaxToken = rowan::SyntaxToken<FerroTexLanguage>;
```

#### `SyntaxElement`

A syntax element that can be either a node or a token.

```rust
pub type SyntaxElement = rowan::SyntaxElement<FerroTexLanguage>;
```

### Re-exports from `rowan`

- `TextRange`: Represents a range of text positions
- `TextSize`: Represents a position in the source text

### Error Types

Parse errors are collected in `ParseResult::errors` as `SyntaxError` structs containing:
- `message`: Error description
- `range`: `TextRange` where the error occurred

### Modules

- **`lexer`**: Tokenization (`Lexer::new(source)`)
- **`parser`**: Parsing implementation (`Parser`, `ParseResult`)
- **`bibtex`**: Specialized parsing for BibTeX files

---

## ferrotex-analysis

Abstract interpretation framework for analyzing TeX macro behavior.

### Primary Entry Points

#### `AbstractMachine::new() -> Self`

Creates a new abstract machine for macro analysis.

```rust
use ferrotex_analysis::AbstractMachine;

let mut machine = AbstractMachine::new();
```

### Key Types

#### `AbstractValue`

Represents a set of possible concrete TeX values in abstract interpretation.

```rust
pub enum AbstractValue {
    Any,                              // Top: any possible token list
    ControlSequence(String),          // Specific control sequence
    Group,                            // Braced group `{ ... }`
    Dimension,                        // Dimension value
    Integer,                          // Integer value
    Bottom,                           // Bottom: unreachable
    Token(String),                    // Simple token
    AnalysisError(String),            // Analysis error
}
```

**Example:**
```rust
use ferrotex_analysis::AbstractValue;

let cs = AbstractValue::ControlSequence("\\def".to_string());
let dim = AbstractValue::Dimension;
```

#### `AbstractState`

The state of the abstract machine during analysis.

```rust
pub struct AbstractState {
    pub input_stack: Vec<AbstractValue>,
    pub registers: HashMap<String, AbstractValue>,
}
```

#### `AbstractMachine`

Simplified abstract machine for analyzing TeX macro expansion.

```rust
pub struct AbstractMachine {
    pub state: AbstractState,
    pub expansion_depth: usize,
    pub max_depth: usize,
    pub call_stack: Vec<String>,
}

impl AbstractMachine {
    pub fn new() -> Self;
    pub fn step(&mut self) -> Option<AbstractValue>;
}
```

**Example:**
```rust
use ferrotex_analysis::{AbstractMachine, AbstractValue};

let mut machine = AbstractMachine::new();
machine.state.input_stack.push(AbstractValue::ControlSequence("\\def".to_string()));

while let Some(token) = machine.step() {
    println!("Processed: {:?}", token);
}
```

### Error Types

- **`AbstractValue::AnalysisError(String)`**: Reports analysis errors including:
  - Maximum recursion depth exceeded
  - Infinite recursion detected

---

## ferrotex-core

Core utilities including package management and math validation.

### Module: `package_manager`

TeX distribution abstraction supporting tlmgr and MiKTeX.

#### Primary Entry Points

##### `PackageManager::new() -> Self`

Auto-detects and creates a package manager for the available TeX distribution.

```rust
use ferrotex_core::package_manager::PackageManager;

let pm = PackageManager::new();
if pm.is_available() {
    let status = pm.install("amsmath")?;
}
```

#### Key Types

##### `PackageManager`

High-level facade for package management operations.

```rust
impl PackageManager {
    pub fn new() -> Self;                                    // Auto-detect backend
    pub fn with_backend(backend: Arc<dyn PackageBackend>) -> Self;  // Custom backend
    pub fn install(&self, package: &str) -> Result<InstallStatus>;
    pub fn search(&self, query: &str) -> Result<Vec<String>>;
    pub fn is_available(&self) -> bool;
    pub fn get_ctan_link(filename: &str) -> Option<String>;
}
```

##### `PackageBackend` (Trait)

Interface for TeX package manager backends.

```rust
pub trait PackageBackend: Debug + Send + Sync {
    fn install(&self, package: &str) -> Result<InstallStatus>;
    fn search(&self, query: &str) -> Result<Vec<String>>;
    fn name(&self) -> &'static str;
}
```

**Implementations:**
- `TlmgrBackend`: TeX Live Manager backend
- `MiktexBackend`: MiKTeX Package Manager backend
- `NoOpBackend`: Fallback when no package manager is available

##### `InstallStatus`

Result of a package installation attempt.

```rust
pub struct InstallStatus {
    pub name: String,
    pub state: InstallState,
    pub message: Option<String>,
}

pub enum InstallState {
    Complete,
    Pending,
    Failed,
    Unknown,
}
```

##### `CommandExecutor` (Trait)

Abstraction for executing system commands (enables testing).

```rust
pub trait CommandExecutor: Send + Sync + Debug {
    fn execute(&self, program: &Path, args: &[&str]) -> Result<std::process::Output>;
}
```

### Module: `math_validator`

Validation tools for LaTeX mathematical expressions.

#### Key Types

##### `DelimiterValidator`

Stack-based delimiter matching validator.

```rust
use ferrotex_core::math_validator::{Delimiter, DelimiterKind, DelimiterValidator};

let mut validator = DelimiterValidator::new();
let delimiters = vec![
    Delimiter { kind: DelimiterKind::LeftParen, position: 0, is_left_command: false },
    Delimiter { kind: DelimiterKind::RightParen, position: 10, is_left_command: false },
];

validator.validate(&delimiters);
if validator.has_errors() {
    for error in validator.errors() {
        eprintln!("{}", error.to_diagnostic_message());
    }
}
```

##### `DelimiterKind`

Enumeration of delimiter types.

```rust
pub enum DelimiterKind {
    LeftParen, RightParen,
    LeftBracket, RightBracket,
    LeftBrace, RightBrace,
    LeftAngle, RightAngle,      // \langle, \rangle
    LeftFloor, RightFloor,      // \lfloor, \rfloor
    LeftCeil, RightCeil,        // \lceil, \rceil
}
```

##### `Delimiter`

Represents a delimiter occurrence in source code.

```rust
pub struct Delimiter {
    pub kind: DelimiterKind,
    pub position: usize,
    pub is_left_command: bool,  // Was created with \left/\right
### Error Types

#### `MathError`

Math validation errors with diagnostic information.

```rust
pub enum MathError {
    MismatchedDelimiter { left_pos, right_pos, left_kind, right_kind },
    UnmatchedOpening { pos, kind },
    UnmatchedClosing { pos, kind },
    IncorrectArgumentCount { command, position, expected, actual },
}
```

#### Unified Error System

The core crate re-exports the unified error types from `ferrotex-build` for convenience across the platform:

- **`FerroTeXError`**: Unified error type
- **`FerroTeXResult<T>`**: Alias for `Result<T, FerroTeXError>`
- **`SourceLocation`**: Line and column
- **`SourceSpan`**: Source range

---

## ferrotex-dap

Returns expected argument count for common math commands.

```rust
use ferrotex_core::math_validator::get_expected_args;

assert_eq!(get_expected_args("frac"), Some(2));
assert_eq!(get_expected_args("sqrt"), Some(1));
assert_eq!(get_expected_args("unknown"), None);
```

---

## ferrotex-dap

Debug Adapter Protocol (DAP) implementation for TeX debugging.

### Primary Entry Points

#### `run_mock_session() -> Result<()>`

Starts a standard DAP loop on stdin/stdout for testing.

```rust
use ferrotex_dap::run_mock_session;

run_mock_session()?;
```

### Key Types

#### `ProtocolMessage`

DAP message types for requests, responses, and events.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    Request {
        seq: i64,
        command: String,
        arguments: Option<serde_json::Value>,
    },
    Response {
        seq: i64,
        request_seq: i64,
        success: bool,
        command: String,
        message: Option<String>,
        body: Option<serde_json::Value>,
    },
    Event {
        seq: i64,
        event: String,
        body: Option<serde_json::Value>,
    },
}
```

**Example:**
```rust
use ferrotex_dap::ProtocolMessage;
use serde_json::json;

let msg = ProtocolMessage::Request {
    seq: 1,
    command: "initialize".to_string(),
    arguments: Some(json!({"adapterID": "ferrotex"})),
};
```

#### `DebugAdapter` (Trait)

Core trait for Debug Adapter implementations.

```rust
pub trait DebugAdapter {
    fn initialize(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;
    fn launch(&mut self, args: serde_json::Value) -> Result<()>;
    fn continue_execution(&mut self) -> Result<()>;
    fn next(&mut self) -> Result<()>;
    fn step_in(&mut self) -> Result<()>;
    fn scopes(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;
    fn variables(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;
    fn disconnect(&mut self) -> Result<()>;
    fn evaluate(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;
    fn attach(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;
}
```

#### `DebugSession<A: DebugAdapter>`

Generic session handler managing the DAP protocol loop.

```rust
pub struct DebugSession<A: DebugAdapter> {
    adapter: A,
    seq: i64,
}

impl<A: DebugAdapter> DebugSession<A> {
    pub fn new(adapter: A) -> Self;
    pub fn run_loop(&mut self) -> Result<()>;
}
```

#### `DebugDriver`

Driver trait for debug session implementations.

```rust
pub trait DebugDriver {
    type Event;
    type Command;
    fn spawn(self) -> (Sender<Self::Command>, Receiver<Self::Event>);
}
```

### Error Types

Errors are returned as `anyhow::Result` with descriptive messages.

---

## ferrotex-build

Content-addressable build system using a Directed Acyclic Graph (DAG).

### Primary Entry Points

#### `Lockfile::new() -> Self`

Creates a new lockfile for reproducible builds.

```rust
use ferrotex_build::Lockfile;
use std::path::Path;

let mut lock = Lockfile::new();
lock.entries.insert("main.tex".to_string(), "abc123hash".to_string());
lock.save(Path::new("ferrotex.lock"))?;
```

### Key Types

#### `Lockfile`

Captures workspace state with SHA-256 content fingerprints.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub version: String,
    pub entries: HashMap<String, String>, // path -> sha256 hash
}

impl Lockfile {
    pub fn new() -> Self;
    pub fn save(&self, path: &Path) -> anyhow::Result<()>;
    pub fn load(path: &Path) -> anyhow::Result<Self>;
}
```

#### `Artifact` (Trait)

Represents an input or output of the build process.

```rust
pub trait Artifact {
    fn id(&self) -> ArtifactId;
    fn fingerprint(&self) -> String;
    fn path(&self) -> Option<PathBuf>;
}
```

#### `ArtifactId`

Unique identifier for artifacts.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub String);
```

#### `FileArtifact`

Concrete artifact implementation for files.

```rust
use ferrotex_build::FileArtifact;
use std::path::PathBuf;

let artifact = FileArtifact::new(PathBuf::from("main.tex"));
let id = artifact.id();
let hash = artifact.fingerprint();
```

#### `Transform` (Trait)

Build step that converts input artifacts to output artifacts.

```rust
pub trait Transform {
    fn description(&self) -> String;
    fn inputs(&self) -> HashSet<ArtifactId>;
    fn outputs(&self) -> HashSet<ArtifactId>;
    fn execute(&self) -> Result<(), String>;
}
```

#### `BuildGraph`

DAG of all transforms and artifacts.

```rust
pub struct BuildGraph {
    artifacts: HashMap<ArtifactId, Box<dyn Artifact>>,
    transforms: Vec<Box<dyn Transform>>,
}

impl BuildGraph {
    pub fn new() -> Self;
    pub fn add_artifact(&mut self, artifact: Box<dyn Artifact>);
    pub fn add_transform(&mut self, transform: Box<dyn Transform>);
    pub fn validate(&self) -> Result<(), String>;  // Detects cycles
}
```

#### `Compiler`

Configuration for executing external TeX engines.

```rust
pub struct Compiler {
    pub engine: String,       // e.g., "pdflatex", "xelatex"
    pub output_dir: PathBuf,
    pub extra_args: Vec<String>,
}

impl Compiler {
    pub fn new(engine: &str, output_dir: PathBuf) -> Self;
    pub fn with_args(self, args: Vec<String>) -> Self;
}
```

#### `ShellTransform`

Executes shell commands as build steps.

```rust
use ferrotex_build::{ShellTransform, ArtifactId};
use std::collections::HashSet;

let mut inputs = HashSet::new();
inputs.insert(ArtifactId("in.tex".to_string()));
let mut outputs = HashSet::new();
outputs.insert(ArtifactId("out.pdf".to_string()));

let transform = ShellTransform::new(
    "Compile LaTeX",
    inputs,
    outputs,
    "pdflatex",
    vec!["-interaction=nonstopmode".to_string()],
);
```

#### `PdfLatexTransform`

Convenience wrapper for pdflatex compilation.

```rust
use ferrotex_build::PdfLatexTransform;

let transform = PdfLatexTransform::new(
    ArtifactId("input.tex".to_string()),
    ArtifactId("output.pdf".to_string()),
    PathBuf::from("input.tex"),
    PathBuf::from("build"),
);
```

### Error Types

All build operations return `FerroTeXResult<T>`, where `FerroTeXError` provides unified error handling with source locations and rich context.

- **`FerroTeXError`**: Unified error type (re-exported from `error` module)
- **`FerroTeXResult<T>`**: Alias for `Result<T, FerroTeXError>`
- **`SourceLocation`**: 1-indexed line and column
- **`SourceSpan`**: Range between two `SourceLocation`s

### Modules


## ferrotex-package

LaTeX package indexing and metadata management.

### Primary Entry Points

#### `PackageIndex::new() -> Self`

Creates a new empty package index.

```rust
use ferrotex_package::PackageIndex;

let mut index = PackageIndex::new();
```

### Key Types

#### `PackageIndex`

Persistent index of LaTeX packages and their metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageIndex {
    pub packages: HashMap<String, PackageMetadata>,
}

impl PackageIndex {
    pub fn new() -> Self;
    pub fn insert(&mut self, name: String, metadata: PackageMetadata);
    pub fn get(&self, name: &str) -> Option<&PackageMetadata>;
    pub fn cache_path() -> Option<std::path::PathBuf>;
    pub fn save_to_cache(&self) -> std::io::Result<()>;
    pub fn load_from_cache() -> Option<Self>;
    pub fn save_to_path(&self, path: &std::path::Path) -> std::io::Result<()>;
    pub fn load_from_path(path: &std::path::Path) -> Option<Self>;
}
```

**Example:**
```rust
use ferrotex_package::PackageIndex;

// Load from cache
if let Some(index) = PackageIndex::load_from_cache() {
    if let Some(meta) = index.get("amsmath") {
        println!("Commands: {:?}", meta.commands);
    }
}
```

#### `PackageMetadata`

Metadata for a single LaTeX package.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageMetadata {
    pub commands: Vec<String>,       // e.g., ["\text", "\dfrac"]
    pub environments: Vec<String>,   // e.g., ["align", "gather"]
}
```

#### `PackageScanner`

Scans TeX distribution to populate the package index.

```rust
use ferrotex_package::scanner::PackageScanner;

let scanner = PackageScanner::new();
let index = scanner.scan();
```

**Located in:** `ferrotex_package::scanner::PackageScanner`

### Error Types

- **`std::io::Error`**: File I/O errors during cache operations
- **`serde_json::Error`**: JSON serialization errors (mapped to io::Error)

---

## ferrotex-log

Streaming parser for TeX engine log files with structured event output.

### Primary Entry Points

#### `LogParser::new() -> Self`

Creates a new log parser with an empty file stack.

```rust
use ferrotex_log::LogParser;

let mut parser = LogParser::new();
```

#### `LogParser::update(&mut self, chunk: &str) -> Vec<LogEvent>`

Processes a chunk of log output and returns any events discovered.

#### `LogParser::parse(&self, content: &str) -> Vec<LogEvent>`

Convenience method for one-shot parsing of complete log content.

### Key Types

#### `LogEvent`

Base event with span, confidence, and payload.

```rust
pub struct LogEvent {
    pub payload: EventPayload,
    pub span: SourceSpan,
    pub confidence: f32,
}
```

#### `EventPayload`

Discriminated union of event types.

```rust
pub enum EventPayload {
    FileEnter { path: String },
    FileExit,
    ErrorStart { message: String },
    Warning { message: String },
    ErrorLineRef { line: usize, excerpt: String },
}
```

---

## ferrotex-math-semantics

Formal semantics and shape analysis for LaTeX math environments.

### Primary Entry Points

Direct type usage for shape analysis.

```rust
use ferrotex_math_semantics::{Shape, Dimension};

let mat = Shape::Matrix {
    rows: Dimension::Finite(2),
    cols: Dimension::Finite(3),
};

assert!(mat.is_compatible_add(&mat));
```

### Key Types

#### `Shape`

Represents the dimensionality of mathematical objects.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Scalar,
    Vector(Dimension),
    Matrix { rows: Dimension, cols: Dimension },
    Tensor(Vec<Dimension>),
    Unknown,
    Invalid(String),
}

impl Shape {
    pub fn is_compatible_add(&self, other: &Shape) -> bool;
    pub fn is_compatible_mul(&self, other: &Shape) -> bool;
}
```

**Example:**
```rust
use ferrotex_math_semantics::{Shape, Dimension};

let vec1 = Shape::Vector(Dimension::Finite(3));
let vec2 = Shape::Vector(Dimension::Finite(3));
assert!(vec1.is_compatible_add(&vec2));

let mat = Shape::Matrix {
    rows: Dimension::Finite(3),
    cols: Dimension::Symbolic("n".to_string()),
};
```

#### `Dimension`

Represents a single dimension size (concrete or symbolic).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    Finite(usize),           // Known size (e.g., 3)
    Symbolic(String),        // Symbolic size (e.g., "n")
    Unknown,                 // Unknown size
}
```

### Modules

- **`analysis`**: Shape inference algorithms
- **`delimiters`**: Math delimiter analysis

### Error Types

Uses `thiserror` for structured error handling. See module-level documentation for specific error types.

---

## Version Information

All core crates are versioned together:

**Current Version:** `0.22.0`

---

## Feature Flags

### `ferrotex-dap`

- **`jxoesneon-tectonic-engine`**: Enables integration with the Tectonic TeX engine for live debugging

### Other Crates

Currently, other crates do not define feature flags. All functionality is available by default.

---

## Additional Resources

- [FerroTeX Repository](https://github.com/jxoesneon/FerroTeX)
- [rowan Documentation](https://docs.rs/rowan) (syntax tree library)
- [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/)
