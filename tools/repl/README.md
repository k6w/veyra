# Veyra REPL v0.2.0

A modern, feature-rich interactive REPL (Read-Eval-Print Loop) for the Veyra programming language.

## Features

### 🎨 Modern CLI Design
- Beautiful ASCII art banner
- Rich color themes (Default, Monokai, Dracula, and more)
- Intuitive command syntax with `:` prefix
- Spinners and progress indicators for long operations
- Clean, professional output formatting

### ⚡ Advanced Editing
- **Syntax Highlighting** - Real-time code coloring
- **Auto-completion** - Tab completion for keywords, variables, and functions
- **Smart Bracket Matching** - Automatic multiline mode for unclosed brackets
- **History Navigation** - Use arrow keys to browse command history
- **VI/Emacs Modes** - Choose your preferred editing mode

### 🔧 Powerful Features
- **Execution Timing** - See how fast your code runs
- **Persistent History** - Commands saved between sessions
- **Session Management** - Save and load REPL sessions
- **File Loading** - Execute Veyra files directly
- **Variable Tracking** - View all defined variables and functions
- **Configurable** - Customize every aspect of the REPL

### 📦 Configuration
Settings are stored in:
- Config: `~/.config/veyra/repl-config.toml` (Linux/macOS) or `%APPDATA%\veyra\repl-config.toml` (Windows)
- History: `~/.local/share/veyra/repl-history.txt` (Linux/macOS) or `%APPDATA%\veyra\repl-history.txt` (Windows)

## Installation

From the project root:

```bash
cd tools/repl
cargo build --release
```

The binary will be at `target/release/veyra-repl` (or `veyra-repl.exe` on Windows).

## Usage

### Basic Usage

```bash
# Start the REPL
veyra-repl

# Execute code and exit
veyra-repl -e "print(42 + 58)"

# Load a startup script
veyra-repl --startup ~/.veyra-startup.vey

# Use a different theme
veyra-repl --theme monokai
```

### Command Line Options

```
Options:
  -s, --startup <FILE>     Load and execute a startup file
  -v, --verbose            Enable verbose output
      --no-highlight       Disable syntax highlighting
      --no-completion      Disable auto-completion
      --vi-mode            Enable VI mode
      --theme <THEME>      Color theme (default, monokai, dracula)
      --no-tips            Don't show tips on startup
  -e, --execute <CODE>     Execute code and exit
  -h, --help               Print help
  -V, --version            Print version
```

## REPL Commands

All REPL commands start with `:` to distinguish them from Veyra code.

### Essential Commands

| Command | Description |
|---------|-------------|
| `:help` or `:h` | Show help message with all commands |
| `:exit` or `:quit` or `:q` | Exit the REPL |
| `:clear` or `:cls` | Clear the screen |

### Information Commands

| Command | Description |
|---------|-------------|
| `:info` | Show REPL and system information |
| `:history` | Display command history |
| `:vars` or `:variables` | List all defined variables |
| `:funcs` or `:functions` | List all defined functions |
| `:type <expr>` | Show the type of an expression |

### Configuration Commands

| Command | Description |
|---------|-------------|
| `:config` | Show current configuration |
| `:config set <key> <value>` | Change a configuration setting |
| `:config save` | Save configuration to file |
| `:theme [name]` | Change color theme |
| `:time` | Toggle execution timing display |
| `:verbose` | Toggle verbose output |
| `:multiline` | Toggle multiline mode |

### Session Commands

| Command | Description |
|---------|-------------|
| `:save <file>` | Save session history to file |
| `:load <file>` | Load and execute a Veyra file |
| `:reset` | Reset the REPL state (clear all variables/functions) |

### Help Commands

| Command | Description |
|---------|-------------|
| `:tips` | Show helpful tips for using the REPL |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Tab` | Auto-complete keywords, variables, functions |
| `↑` / `↓` | Navigate command history |
| `Ctrl+C` | Cancel current input (or show exit message) |
| `Ctrl+D` | Exit the REPL |
| `Ctrl+L` | Clear screen |
| `Ctrl+A` | Move to beginning of line |
| `Ctrl+E` | Move to end of line |
| `Ctrl+U` | Clear line before cursor |
| `Ctrl+K` | Clear line after cursor |

## Configuration Settings

You can customize these settings using `:config set <key> <value>`:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `syntax_highlighting` | bool | true | Enable syntax highlighting |
| `auto_completion` | bool | true | Enable auto-completion |
| `multiline_mode` | bool | true | Enable multiline editing |
| `show_timing` | bool | true | Show execution time |
| `max_history` | number | 10000 | Maximum history entries |
| `vi_mode` | bool | false | Use VI editing mode |
| `show_tips` | bool | true | Show tips on startup |
| `auto_save_history` | bool | true | Automatically save history |

Example:
```
:config set show_timing false
:config set vi_mode true
```

## Examples

### Basic Calculations
```veyra
veyra> 2 + 2
⇒ 4

veyra> let x = 42
veyra> x * 2
⇒ 84
```

### Multiline Input
```veyra
veyra> fn fibonacci(n) {
...   if n <= 1 {
...     return n
...   }
...   return fibonacci(n-1) + fibonacci(n-2)
... }
veyra> fibonacci(10)
⇒ 55 (2.345ms)
```

### Working with Data Structures
```veyra
veyra> let numbers = [1, 2, 3, 4, 5]
veyra> numbers
⇒ [1, 2, 3, 4, 5]

veyra> let person = {"name": "Alice", "age": 30}
veyra> person
⇒ {"age": 30, "name": "Alice"}
```

### Using REPL Commands
```veyra
veyra> let x = 10
veyra> let y = 20
veyra> :vars
▸ Defined Variables
──────────────────────────────────────────────────────────────────────
│ Variable │ Type │
├──────────┼──────┤
│ x        │ int  │
│ y        │ int  │

veyra> :type x + y
ℹ Type: int

veyra> :save my-session.vey
✓ Session saved to my-session.vey
```

## Tips and Tricks

1. **Incomplete Input**: If you have unclosed brackets, the REPL automatically enters multiline mode.

2. **Quick Exit**: Press `Ctrl+D` for a fast exit, or type `:q` for short.

3. **Clear Screen**: Use `Ctrl+L` or `:clear` to clean up your terminal.

4. **History Search**: Start typing and use `↑` to find previous commands starting with that text.

5. **Persistent Config**: All your settings are saved automatically and restored on next launch.

6. **Startup Scripts**: Create `~/.veyra-startup.vey` with common functions and load it with `--startup`.

7. **Execute Mode**: Use `-e` flag for one-off executions: `veyra-repl -e "print('Hello')"`

## Architecture

The new REPL is built with a modular architecture:

- **`main.rs`** - Entry point and main loop
- **`config.rs`** - Configuration management
- **`state.rs`** - REPL state and execution
- **`ui.rs`** - User interface and styling
- **`helper.rs`** - Completion, highlighting, and validation
- **`commands.rs`** - REPL command handlers

## Comparison with Old REPL

### Old REPL (v0.1.0)
- Basic colored output
- Simple command handling
- Limited configuration
- No auto-completion
- No syntax highlighting
- Basic history support

### New REPL (v0.2.0)
✨ Modern CLI design with themes
✨ Advanced auto-completion
✨ Real-time syntax highlighting
✨ Smart multiline editing
✨ Execution timing
✨ Comprehensive configuration
✨ Session management
✨ Better error messages
✨ Table formatting
✨ Progress indicators
✨ Extensive help system

## Dependencies

The new REPL uses modern, well-maintained crates:

- **`rustyline`** - Advanced line editing
- **`clap`** - Modern CLI argument parsing
- **`console`** - Cross-platform terminal features
- **`indicatif`** - Progress bars and spinners
- **`owo-colors`** - Fast, modern terminal colors
- **`nu-ansi-term`** - Advanced ANSI styling
- **`crossterm`** - Cross-platform terminal manipulation

## Contributing

Contributions are welcome! Some areas for improvement:

- [ ] Add more color themes
- [ ] Implement debugger integration
- [ ] Add code snippets/templates
- [ ] Support for plugins
- [ ] Network REPL mode (remote execution)
- [ ] Better error recovery
- [ ] Code formatting on paste
- [ ] Export session to different formats

## License

Licensed under MIT OR Apache-2.0, same as the Veyra project.
