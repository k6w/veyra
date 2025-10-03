# 🎨 Veyra Language Support for Visual Studio Code

<div align="center">

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![VS Code](https://img.shields.io/badge/VS%20Code-1.75%2B-007ACC.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

**Complete, modern language support for the Veyra programming language**

Rich syntax highlighting • Intelligent IntelliSense • Real-time diagnostics • Seamless debugging

[Getting Started](#getting-started) • [Features](#features) • [Configuration](#configuration) • [Shortcuts](#keyboard-shortcuts)

</div>

---

## ✨ Features at a Glance

### 🎯 **Smart Code Execution**
- **One-Click Run** - Execute your Veyra files instantly with the ▶️ button in the editor toolbar
- **Terminal Integration** - Beautifully formatted terminal output with emoji indicators
- **Progress Indicators** - Visual feedback for long-running operations
- **Status Bar** - Real-time display of file status, errors, and warnings

### 🎨 **Beautiful Syntax Highlighting**
- **300+ built-in functions** with semantic coloring
- **Advanced number formats**: binary (`0b1010`), octal (`0o755`), hex (`0xFF`), scientific (`1.5e10`)
- **String escape sequences**: `\n`, `\r`, `\t`, `\x00`, `\u{1F600}`, `\"`
- **Keywords and operators**: Full support for all language constructs
- **Function declarations**: Distinct highlighting for definitions vs calls
- **Constants**: `PI`, `E`, `PHI`, `TAU`, `true`, `false`, `None`

### ⚡ **Real-Time Error Detection**
- **Live validation** as you type (500ms intelligent debounce)
- **Precise error positioning**: Errors highlight exactly where they occur
- **Smart token highlighting**: Underlines the problematic code
- **Compiler integration**: Uses actual Veyra compiler for 100% accuracy
- **Error counts in status bar**: Quick overview of issues

### 🧠 **Advanced IntelliSense**
- **Context-aware auto-completion** for all standard library functions
- **Signature help**: See function parameters and types as you type
- **Hover documentation**: Detailed information on hover for all stdlib functions
- **Go to definition**: Jump directly to function definitions
- **Symbol navigation**: Quickly find functions and variables across your workspace
- **Parameter hints**: Smart suggestions while writing function calls

### 🚀 **Quick Actions Menu**
Click the Veyra status bar or use the command palette to access:
- ▶️ **Run Veyra File** - Execute the current file
- 🔨 **Build Project** - Build your entire Veyra project
- ✨ **Format File** - Auto-format with veyra-fmt
- 🔍 **Lint File** - Check code quality with veyra-lint
- 📦 **New Project** - Create a new Veyra project with scaffolding
- 📋 **Show Output** - View the Veyra output channel
- 📚 **Documentation** - Quick access to docs

### 📚 **Comprehensive Standard Library Coverage**

#### Core Functions (14)
`is_int`, `is_float`, `is_string`, `is_bool`, `is_array`, `is_none`, `type_of`, `to_int`, `to_float`, `to_string`, `to_bool`, `len`, `print`, `deep_copy`

#### Math Functions (30+)
`abs`, `sign`, `min`, `max`, `clamp`, `pow`, `sqrt`, `cbrt`, `exp`, `ln`, `log10`, `log2`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `factorial`, `binomial`, `permutation`, `floor`, `ceil`, `round`, `mean`, `variance`, `std_dev`

#### String Functions (16)
`string_length`, `string_concat`, `string_substring`, `string_index_of`, `string_last_index_of`, `string_starts_with`, `string_ends_with`, `string_contains`, `string_to_upper`, `string_to_lower`, `string_trim`, `string_replace`, `string_split`, `string_join`, `string_repeat`, `string_reverse`

#### Array Functions (19)
`array_push`, `array_pop`, `array_unshift`, `array_shift`, `array_insert`, `array_remove`, `array_contains`, `array_slice`, `array_concat`, `array_reverse`, `array_sort`, `array_map`, `array_filter`, `array_reduce`, `array_find`, `array_unique`, `array_flatten`, `array_min`, `array_max`, `array_sum`

#### I/O Functions (10)
`read_file`, `write_file`, `append_file`, `file_exists`, `file_size`, `list_directory`, `create_directory`, `delete_file`, `copy_file`, `move_file`

#### Network Functions (10)
`http_get`, `http_post`, `http_put`, `http_delete`, `json_encode`, `json_decode`, `url_encode`, `url_decode`, `base64_encode`, `base64_decode`

#### DateTime Functions (9)
`now`, `current_time`, `timestamp_to_struct`, `format_datetime`, `parse_datetime`, `add_days`, `add_hours`, `diff_days`, `is_leap_year`

---

## 🚀 Getting Started

### � Installation

#### **From VSIX Package**
1. Download the latest `veyra-lang-x.x.x.vsix` file
2. Open VS Code
3. Go to **Extensions** (`Ctrl+Shift+X`)
4. Click the **"..."** menu → **Install from VSIX**
5. Select the downloaded file
6. Reload VS Code

#### **From Marketplace** *(coming soon)*
Search for "Veyra" in the VS Code marketplace and click **Install**.

### ⚡ Quick Start

1. **Create a new file** with `.vey` extension
2. **Start typing** - IntelliSense will guide you
3. **Click the ▶️ button** in the editor toolbar to run
4. Or use **`Ctrl+F5`** to execute instantly

### 📝 Example

```veyra
// Your first Veyra program
fn greet(name: string) {
    print("Hello, " + name + "!")
}

greet("World")

// Use stdlib functions
let numbers = [1, 2, 3, 4, 5]
let doubled = array_map(numbers, fn(x) { x * 2 })
print("Doubled: " + to_string(doubled))
```

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+F5` | Run current Veyra file |
| `Shift+Alt+F` | Format current file |
| `F12` | Go to definition |
| `Ctrl+Space` | Trigger IntelliSense |
| `Ctrl+Shift+Space` | Show signature help |

---

## 🎮 Commands

Access via **Command Palette** (`Ctrl+Shift+P`):

| Command | Description |
|---------|-------------|
| `Veyra: Run File` | Execute the current Veyra file |
| `Veyra: Build Project` | Build entire Veyra project |
| `Veyra: Format File` | Format code with veyra-fmt |
| `Veyra: Lint File` | Check code quality |
| `Veyra: New Project` | Create new project with scaffolding |
| `Veyra: Quick Actions` | Open quick actions menu |
| `Veyra: Show Output` | View Veyra output channel |
| `Veyra: Open Documentation` | Access docs |

---

## ⚙️ Configuration

Customize your Veyra experience in **Settings** (`Ctrl+,`):

```json
{
  // Compiler Settings
  "veyra.compilerPath": "",              // Auto-detected if empty
  "veyra.enableLanguageServer": true,    // Enable LSP features
  "veyra.languageServerPath": "",        // Auto-detected if empty
  
  // Editor Behavior
  "veyra.formatOnSave": true,            // Auto-format on save
  "veyra.lintOnSave": true,              // Auto-lint on save
  "veyra.build.beforeRun": false,        // Build before running
  
  // Diagnostics
  "veyra.diagnostics.enable": true,      // Enable real-time diagnostics
  "veyra.diagnostics.debounceMs": 500,   // Validation debounce (100-5000ms)
  
  // IntelliSense
  "veyra.intellisense.enable": true,     // Enable IntelliSense
  "veyra.intellisense.includeStdlib": true, // Include stdlib in completions
  
  // Advanced
  "veyra.trace.server": "off"            // LSP trace level: off|messages|verbose
}
```

---

## 📋 Requirements

- **Veyra Compiler**: Auto-detected from workspace or system PATH
  - If not found, configure path in settings: `veyra.compilerPath`
- **VS Code**: Version 1.75.0 or higher
- **Operating Systems**: Windows, macOS, Linux

---

## 🎨 UI Features

### **Status Bar**
- 📄 Current file name and status
- ⚠️ Error and warning counts (click to view problems)
- 🔄 Running indicator during execution

### **Output Channel**
- Beautifully formatted logs with emojis
- Execution history
- Compiler messages
- Access via **View → Output → Veyra**

### **Context Menus**
- Right-click in editor for quick actions
- Run, format, and lint from context menu
- Available only for `.vey` files

---

## 🐛 Troubleshooting

### **Compiler Not Found**
If you see "Veyra compiler not found":
1. Install the Veyra toolchain
2. Or set `veyra.compilerPath` in settings
3. Make sure `veyra` is in your system PATH

### **IntelliSense Not Working**
1. Check `veyra.intellisense.enable` is `true`
2. Reload VS Code: **Developer → Reload Window**
3. Check Output panel for errors

### **Formatting Issues**
1. Ensure `veyra-fmt` is installed
2. Check terminal output for errors
3. Try manual format: **Right-click → Format Document**

---

## 🛠️ Development

### **Building from Source**
```bash
cd tools/vscode_extension
npm install
npm run compile
```

### **Packaging**
```bash
npm install -g @vscode/vsce
vsce package
```

---

## 📄 License

MIT License - see LICENSE file for details

---

## 🙏 Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## 📧 Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/k6w/veyra/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/k6w/veyra/discussions)
- 📚 **Documentation**: See `docs/` folder in the workspace

---

<div align="center">

**Made with ❤️ by the Veyra Team**

⭐ Star us on [GitHub](https://github.com/k6w/veyra) ⭐

</div>

## Usage

1. Open any `.vey` file
2. Start typing - IntelliSense will provide suggestions
3. Errors appear automatically in the Problems panel
4. Press `Ctrl+F5` to run your code
5. Hover over functions for documentation

## Examples

### Auto-completion
```veyra
let numbers = [1, 2, 3, 4, 5]
array_  // <-- IntelliSense shows 30+ array functions
```

### Error Detection
```veyra
fn hello() {
    print("Hello, World!")
    // Missing closing brace - error shown immediately
```

### Hover Documentation
```veyra
let result = sqrt(16)  // Hover over 'sqrt' to see documentation
```

## Known Issues

- Type annotations (`:i64`, `:f64`) are not yet supported by the parser
- Dictionary literals are not yet implemented in the parser
- Some advanced example files may show errors due to unimplemented language features

These are **compiler limitations**, not extension bugs. The extension accurately reports what the compiler detects.

## Contributing

Contributions are welcome! Please visit our [GitHub repository](https://github.com/k6w/veyra) to:
- Report bugs
- Suggest features
- Submit pull requests

## Release Notes

### 0.1.0 (2025-10-03)

#### ✨ Features
- Complete syntax highlighting with 300+ functions
- Real-time error detection with accurate positioning
- IntelliSense with auto-completion, hover, and signature help
- Integrated compiler support
- Symbol navigation and go-to-definition
- Code snippets for common patterns

#### 🐛 Bug Fixes
- Fixed error positioning (was showing all errors at line 1, column 1)
- Fixed diagnostic collection conflicts
- Improved error message parsing
- Enhanced token highlighting

#### 🔧 Technical Improvements
- Completely rewritten TextMate grammar
- New diagnostic provider with debounced validation
- Proper error format handling
- Token-aware error highlighting
- Comprehensive stdlib integration

## Support

- **Documentation**: [Veyra Docs](https://github.com/k6w/veyra/tree/main/docs)
- **Issues**: [GitHub Issues](https://github.com/k6w/veyra/issues)
- **Discussions**: [GitHub Discussions](https://github.com/k6w/veyra/discussions)

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Enjoy coding in Veyra! 🚀**
