# Contributing to Veyra

Thank you for your interest in contributing to Veyra! We welcome contributions from everyone, whether you're fixing a bug, adding a feature, or improving documentation.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Project Structure](#project-structure)

## 🤝 Code of Conduct

By participating in this project, you agree to:
- Be respectful and inclusive
- Accept constructive criticism
- Focus on what's best for the community
- Show empathy towards other community members

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Git** - For version control
- **A GitHub account** - For submitting contributions

### First Time Setup

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/veyra.git
   cd veyra
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/k6w/veyra.git
   ```
4. **Build the project**:
   ```bash
   cd compiler
   cargo build
   cd ../tools
   cargo build
   ```
5. **Run tests**:
   ```bash
   cargo test
   ```

## 💻 Development Setup

### Building Components

```bash
# Build compiler
cd compiler
cargo build --release

# Build runtime
cd runtime
cargo build --release

# Build tools (REPL, LSP, debugger, etc.)
cd tools
cargo build --release

# Build specific tool
cargo build -p veyra-repl --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific package
cargo test -p veyra-compiler
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all
```

## 🛠️ How to Contribute

### Types of Contributions

- **🐛 Bug Fixes** - Fix issues and improve stability
- **✨ New Features** - Add new language features or tools
- **📝 Documentation** - Improve docs, examples, or comments
- **🧪 Tests** - Add or improve test coverage
- **🎨 Code Quality** - Refactoring and optimization
- **🔧 Tools** - Improve developer tools (REPL, LSP, etc.)

### Finding Issues to Work On

1. Check the [Issues](https://github.com/k6w/veyra/issues) page
2. Look for issues labeled:
   - `good first issue` - Great for newcomers
   - `help wanted` - Need community help
   - `bug` - Bug fixes needed
   - `enhancement` - Feature requests

### Before Starting Work

1. **Check existing issues** - Make sure your idea isn't already being worked on
2. **Create an issue** - Discuss your idea before implementing large changes
3. **Get feedback** - Wait for maintainer approval on significant features
4. **Claim the issue** - Comment on the issue to let others know you're working on it

## 📝 Coding Standards

### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` to catch common mistakes
- Write idiomatic Rust code

### Naming Conventions

```rust
// Modules: snake_case
mod parser;
mod lexer;

// Types: PascalCase
struct Token;
enum TokenKind;

// Functions: snake_case
fn parse_expression();
fn tokenize();

// Constants: SCREAMING_SNAKE_CASE
const MAX_DEPTH: usize = 100;

// Variables: snake_case
let token_kind = TokenKind::Identifier;
```

### Documentation

```rust
/// Brief description of the function
///
/// # Arguments
///
/// * `input` - Description of input parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = function_name(input);
/// ```
pub fn function_name(input: &str) -> Result<String, Error> {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for operations that can fail
- Provide meaningful error messages
- Use the `?` operator for error propagation
- Document error conditions

```rust
pub fn parse_file(path: &Path) -> Result<Program, ParseError> {
    let content = fs::read_to_string(path)
        .map_err(|e| ParseError::IoError(e))?;
    
    let tokens = tokenize(&content)?;
    parse_program(tokens)
}
```

## 🧪 Testing

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let input = "test input";
        let expected = "expected output";
        assert_eq!(function_under_test(input), expected);
    }

    #[test]
    fn test_error_case() {
        let input = "invalid input";
        assert!(function_under_test(input).is_err());
    }
}
```

### Test Coverage

- Write tests for new features
- Add tests for bug fixes to prevent regressions
- Include edge cases and error conditions
- Aim for high test coverage

### Running Specific Tests

```bash
# Run tests for a specific module
cargo test lexer

# Run a specific test
cargo test test_basic_functionality

# Run tests with output
cargo test -- --nocapture
```

## 🔄 Pull Request Process

### 1. Create a Branch

```bash
# Create and switch to a new branch
git checkout -b feature/my-new-feature

# Or for bug fixes
git checkout -b fix/issue-123
```

### 2. Make Your Changes

- Write clean, well-documented code
- Follow coding standards
- Add tests for new functionality
- Update documentation as needed

### 3. Commit Your Changes

```bash
# Stage your changes
git add .

# Commit with a descriptive message
git commit -m "Add feature: brief description"
```

**Commit Message Guidelines:**
- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit first line to 72 characters
- Reference issues and pull requests

Examples:
```
Add support for async/await syntax
Fix lexer bug with string escaping (#123)
Update documentation for error handling
Refactor parser for better performance
```

### 4. Keep Your Fork Updated

```bash
# Fetch upstream changes
git fetch upstream

# Merge upstream changes
git merge upstream/main
```

### 5. Push to Your Fork

```bash
git push origin feature/my-new-feature
```

### 6. Create a Pull Request

1. Go to your fork on GitHub
2. Click "Pull Request"
3. Select your branch
4. Fill out the PR template:
   - **Title**: Clear, concise description
   - **Description**: What changes were made and why
   - **Related Issues**: Link to related issues
   - **Testing**: How you tested the changes
   - **Screenshots**: If applicable

### 7. PR Review Process

- Maintainers will review your PR
- Address feedback and requested changes
- Push additional commits to your branch if needed
- Once approved, a maintainer will merge your PR

## 📁 Project Structure

```
veyra/
├── compiler/              # Core compiler
│   ├── src/
│   │   ├── lexer.rs      # Tokenization
│   │   ├── parser.rs     # AST generation
│   │   ├── ast.rs        # AST definitions
│   │   ├── interpreter.rs # Execution engine
│   │   └── error.rs      # Error types
│   └── Cargo.toml
│
├── runtime/               # Advanced runtime features
│   ├── src/
│   │   ├── garbage_collector.rs
│   │   ├── jit_compiler.rs
│   │   ├── async_runtime.rs
│   │   └── actor_system.rs
│   └── Cargo.toml
│
├── tools/                 # Developer tools
│   ├── repl/             # Interactive REPL
│   ├── lsp/              # Language Server Protocol
│   ├── debugger/         # Debugger
│   ├── linter/           # Code linter
│   └── package_manager/  # Package management
│
├── stdlib/                # Standard library
│   ├── core.vey
│   ├── math.vey
│   ├── string.vey
│   └── collections.vey
│
├── spec/                  # Language specification
│   ├── LANGUAGE_SPEC.md
│   └── GRAMMAR.ebnf
│
├── examples/              # Example programs
├── tests/                 # Test suite
└── docs/                  # Additional documentation
```

## 🎯 Areas Needing Contributions

### High Priority
- [ ] Improve error messages
- [ ] Add more standard library functions
- [ ] Expand test coverage
- [ ] Performance optimizations
- [ ] Documentation improvements

### Features
- [ ] Pattern matching enhancements
- [ ] Type inference improvements
- [ ] Additional collection types
- [ ] Async/await optimizations
- [ ] Package registry implementation

### Tools
- [ ] IDE plugins for other editors
- [ ] Code formatter improvements
- [ ] Debugger enhancements
- [ ] REPL features (autocomplete, syntax highlighting)
- [ ] Package manager features

## 💬 Getting Help

- **Questions?** Open a [Discussion](https://github.com/k6w/veyra/discussions)
- **Found a bug?** Open an [Issue](https://github.com/k6w/veyra/issues)
- **Need guidance?** Ask in the issue comments

## 🙏 Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Credited in release notes
- Recognized in project documentation

Thank you for contributing to Veyra! 🎉

---

**Questions?** Feel free to ask in [GitHub Discussions](https://github.com/k6w/veyra/discussions)
