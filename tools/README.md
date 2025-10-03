# Veyra Development Tools

This directory contains all the development tools for the Veyra programming language.

## Tools Overview

### 🔧 Core Development Tools

- **`repl/`** - Interactive REPL (Read-Eval-Print Loop)
- **`formatter/`** - Code formatter (`veyra-fmt`)
- **`linter/`** - Static analysis and linting (`veyra-lint`)
- **`package_manager/`** - Package manager (`veyra-pkg`)
- **`debugger/`** - Interactive debugger (`veyra-dbg`)

### 🎨 IDE Integration

- **`lsp/`** - Language Server Protocol implementation
- **`vscode_extension/`** - Visual Studio Code extension

## Building All Tools

From the tools directory:

```bash
# Build all tools
cargo build --release

# Build specific tool
cargo build --release -p veyra-repl
cargo build --release -p veyra-fmt
cargo build --release -p veyra-lint
cargo build --release -p veyra-pkg
cargo build --release -p veyra-lsp
cargo build --release -p veyra-dbg
```

## Tool Usage

### REPL (Interactive Shell)
```bash
# Start interactive REPL
veyra-repl

# Load startup file
veyra-repl --startup init.vey

# Verbose mode
veyra-repl --verbose
```

### Code Formatter
```bash
# Format files in place
veyra-fmt --write *.vey

# Check if files need formatting
veyra-fmt --check src/

# Show formatting diff
veyra-fmt --diff main.vey
```

### Linter
```bash
# Lint files with warnings
veyra-lint --warnings src/

# Output as JSON
veyra-lint --format json *.vey

# Treat warnings as errors
veyra-lint --warnings-as-errors
```

### Package Manager
```bash
# Create new project
veyra-pkg init my-project

# Install dependencies
veyra-pkg install

# Build project
veyra-pkg build --release

# Run project
veyra-pkg run

# Run tests
veyra-pkg test
```

### Language Server
```bash
# Start language server (typically used by editors)
veyra-lsp
```

### Debugger
```bash
# Debug a Veyra file
veyra-dbg program.vey

# Start debugging immediately
veyra-dbg --run program.vey
```

## IDE Integration

### VS Code Extension

The VS Code extension provides:
- Syntax highlighting
- Code completion
- Error diagnostics
- Integrated debugging
- Formatting and linting
- Snippets and templates

To install:
1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Veyra Language Support"
4. Install the extension

Or build from source:
```bash
cd vscode_extension
npm install
npm run compile
code --install-extension .
```

## Development

### Adding New Tools

1. Create a new directory under `tools/`
2. Add a `Cargo.toml` with the tool configuration
3. Add the tool to the workspace `Cargo.toml`
4. Implement the tool in `src/main.rs`

### Dependencies

All tools can reference the main compiler:
```toml
[dependencies.veyra-compiler]
path = "../../compiler"
```

Common dependencies are defined in the workspace `Cargo.toml`.

## Features by Tool

### ✅ Implemented Features

| Tool | Features |
|------|----------|
| **REPL** | Interactive shell, history, variable inspection, startup files |
| **Formatter** | Code formatting, diff view, in-place editing, configuration |
| **Linter** | Static analysis, multiple rule types, JSON output, warnings |
| **Package Manager** | Project creation, dependency management, build system, testing |
| **Language Server** | LSP protocol, completions, diagnostics, symbols, hover |
| **Debugger** | Breakpoints, stepping, variable inspection, call stack |
| **VS Code Extension** | Syntax highlighting, commands, snippets, LSP integration |

### 🔄 Future Enhancements

- Performance profiler
- Code coverage analysis
- Documentation generator
- Benchmark runner
- Package registry server
- Web-based IDE
- Vim/Emacs plugins

## Installation

### From Source
```bash
# Build and install all tools
cd tools
cargo install --path repl
cargo install --path formatter
cargo install --path linter
cargo install --path package_manager
cargo install --path lsp
cargo install --path debugger
```

### Binary Releases
Download pre-built binaries from the [releases page](https://github.com/k6w/veyra/releases).

## Configuration

Each tool supports configuration files:
- **Formatter**: `.veyra-fmt.toml`
- **Linter**: `.veyra-lint.json`
- **Package Manager**: `veyra.toml`
- **Language Server**: VS Code settings

## Contributing

See the main project [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on contributing to the Veyra toolchain.