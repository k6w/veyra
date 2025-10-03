# Veyra Programming Language

<div align="center">

![Veyra Logo](https://img.shields.io/badge/Veyra-Programming%20Language-purple?style=for-the-badge)

**A modern, safe, and performant programming language with advanced runtime features**

[![CI](https://github.com/k6w/veyra/actions/workflows/ci.yml/badge.svg)](https://github.com/k6w/veyra/actions/workflows/ci.yml)
[![Release](https://github.com/k6w/veyra/actions/workflows/release.yml/badge.svg)](https://github.com/k6w/veyra/actions/workflows/release.yml)
[![Security](https://github.com/k6w/veyra/actions/workflows/security.yml/badge.svg)](https://github.com/k6w/veyra/actions/workflows/security.yml)
[![Version](https://img.shields.io/badge/version-1.0.0-blue)](https://github.com/k6w/veyra/releases)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)

[Quick Start](#quick-start) • [Documentation](#documentation) • [Examples](#examples) • [Contributing](#contributing)

</div>

---

## 🌟 Overview

Veyra is a production-ready programming language that combines the safety of modern language design with powerful runtime features. Built with Rust, it offers:

- **Memory Safety** - Advanced garbage collection with mark-and-sweep algorithm
- **High Performance** - JIT compilation with optimization passes
- **Modern Concurrency** - Built-in async/await and actor model support
- **Developer Experience** - Comprehensive toolchain with IDE integration
- **Production Ready** - Complete runtime system with thread pools and memory management

## ✨ Key Features

### Core Language
- **Dynamic Typing** with runtime type checking
- **First-class Functions** with closures and higher-order functions
- **Pattern Matching** for elegant control flow
- **Rich Operators** including compound assignments and power operators
- **Exception Handling** with try-catch-finally blocks
- **Module System** for code organization

### Advanced Runtime
- **Garbage Collector** - Automatic memory management
- **JIT Compiler** - Dynamic compilation for performance
- **Async Runtime** - Native async/await support
- **Thread Pool** - Work-stealing scheduler for parallelism
- **Actor System** - Message-passing concurrency

### Developer Toolchain
- **REPL** - Interactive development environment
- **LSP Server** - Full IDE integration with IntelliSense
- **Debugger** - Step-through debugging with breakpoints
- **Linter** - Code quality and style checking
- **Package Manager** - Dependency management
- **VS Code Extension** - Syntax highlighting and language support

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or higher ([Install Rust](https://rustup.rs/))
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/k6w/veyra.git
cd veyra

# Build the compiler
cd compiler
cargo build --release

# Build the tools
cd ../tools
cargo build --release
```

### Your First Veyra Program

Create a file `hello.vey`:

```veyra
# Classic Hello World
fn main() {
    println("Hello, Veyra!")
}

main()
```

Run it:

```bash
./compiler/target/release/veyra hello.vey
```

### Try the REPL

```bash
./tools/target/release/veyra-repl
```

```veyra
>>> let x = 42
>>> let y = x * 2
>>> println(y)
84
>>> fn greet(name) { return "Hello, " + name + "!" }
>>> greet("World")
"Hello, World!"
```

## 📚 Documentation

- **[Language Specification](spec/LANGUAGE_SPEC.md)** - Complete language reference
- **[Grammar](spec/GRAMMAR.ebnf)** - EBNF grammar specification
- **[Quick Start Guide](QUICK_START.md)** - Get up and running quickly
- **[Design Philosophy](docs/DESIGN_PHILOSOPHY.md)** - Language design principles

## 💡 Examples

### Variables and Types

```veyra
let name = "Alice"
let age = 30
let pi = 3.14159
let active = true
let items = [1, 2, 3, 4, 5]
```

### Functions

```veyra
fn fibonacci(n) {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

println(fibonacci(10))  # Output: 55
```

### Control Flow

```veyra
fn check_grade(score) {
    if score >= 90 {
        return "A"
    } elif score >= 80 {
        return "B"
    } elif score >= 70 {
        return "C"
    } else {
        return "F"
    }
}
```

### Collections

```veyra
# Arrays
let numbers = [1, 2, 3, 4, 5]
for num in numbers {
    println(num * 2)
}

# Dictionaries
let person = {"name": "Bob", "age": 25}
println(person["name"])
```

### Error Handling

```veyra
fn safe_divide(a, b) {
    try {
        return a / b
    } catch e {
        println("Error: " + e)
        return null
    } finally {
        println("Division attempted")
    }
}
```

More examples in the [`examples/`](examples/) directory.

## 🛠️ Tools

### Language Server (LSP)

```bash
# Start the language server
./tools/target/release/veyra-lsp
```

Provides:
- Auto-completion
- Go to definition
- Hover information
- Error diagnostics
- Code formatting

### Debugger

```bash
# Debug a program
./tools/target/release/veyra-dbg program.vey
```

Features:
- Breakpoints
- Step through execution
- Variable inspection
- Call stack viewing

### Linter

```bash
# Lint your code
./tools/target/release/veyra-lint program.vey
```

Checks for:
- Code style violations
- Potential bugs
- Best practices

### Package Manager

```bash
# Install a package
./tools/target/release/veyra-pkg install package-name

# Update dependencies
./tools/target/release/veyra-pkg update
```

## 🧪 Running Tests

```bash
# Run compiler tests
cd compiler
cargo test

# Run tool tests
cd tools
cargo test

# Run language test suite
./compiler/target/release/veyra tests/comprehensive_test_suite.vey
```

## 🏗️ Project Structure

```
veyra/
├── compiler/          # Veyra compiler (lexer, parser, interpreter)
├── runtime/           # Advanced runtime (GC, JIT, async, actors)
├── tools/             # Developer tools (REPL, LSP, debugger, etc.)
│   ├── repl/         # Interactive REPL
│   ├── lsp/          # Language Server Protocol implementation
│   ├── debugger/     # Debugger
│   ├── linter/       # Code linter
│   ├── package_manager/  # Package manager
│   └── vscode_extension/ # VS Code extension
├── stdlib/            # Standard library modules
├── spec/              # Language specification
├── docs/              # Additional documentation
├── examples/          # Example programs
└── tests/             # Test suite
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Fork and clone
git clone https://github.com/k6w/veyra.git
cd veyra

# Build all components
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Inspired by modern programming language design
- Community-driven development

## 📧 Contact

- **Issues**: [GitHub Issues](https://github.com/k6w/veyra/issues)
- **Discussions**: [GitHub Discussions](https://github.com/k6w/veyra/discussions)

---

<div align="center">

**[⬆ back to top](#veyra-programming-language)**

Made with ❤️ by the Veyra community

</div>
