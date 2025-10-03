# Veyra Quick Start Guide

Get up and running with Veyra in minutes!

## 📦 Installation

### Prerequisites

Make sure you have:
- **Rust 1.70+** - [Install here](https://rustup.rs/)
- **Git** - For cloning the repository

### Build from Source

```bash
# Clone the repository
git clone https://github.com/k6w/veyra.git
cd veyra

# Build the compiler
cd compiler
cargo build --release

# Build the toolchain
cd ../tools
cargo build --release

# Verify installation
cd ..
./compiler/target/release/veyra --version
```

## 🎯 Your First Program

### Hello, World!

Create `hello.vey`:

```veyra
println("Hello, World!")
```

Run it:

```bash
./compiler/target/release/veyra hello.vey
```

### Interactive REPL

Launch the REPL for quick experiments:

```bash
./tools/target/release/veyra-repl
```

Try these commands:

```veyra
>>> let name = "Veyra"
>>> println("Hello, " + name + "!")
Hello, Veyra!

>>> let square = fn(x) { return x * x }
>>> square(5)
25

>>> let numbers = [1, 2, 3, 4, 5]
>>> for n in numbers { println(n * 2) }
2
4
6
8
10
```

## 📖 Language Basics

### Variables

```veyra
let name = "Alice"          # String
let age = 25                # Integer
let height = 5.8            # Float
let is_student = true       # Boolean
let items = [1, 2, 3]       # Array
let person = {"name": "Bob", "age": 30}  # Dictionary
```

### Functions

```veyra
# Simple function
fn greet(name) {
    println("Hello, " + name + "!")
}

greet("World")

# Function with return value
fn add(a, b) {
    return a + b
}

let result = add(5, 3)
println(result)  # 8

# Recursive function
fn factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

println(factorial(5))  # 120
```

### Control Flow

```veyra
# If-elif-else
let score = 85

if score >= 90 {
    println("Grade: A")
} elif score >= 80 {
    println("Grade: B")
} elif score >= 70 {
    println("Grade: C")
} else {
    println("Grade: F")
}

# While loop
let count = 0
while count < 5 {
    println(count)
    count = count + 1
}

# For loop
let fruits = ["apple", "banana", "orange"]
for fruit in fruits {
    println("I like " + fruit)
}
```

### Collections

```veyra
# Arrays
let numbers = [1, 2, 3, 4, 5]
println(numbers[0])        # 1
println(len(numbers))      # 5

# Add elements
numbers = numbers + [6]
println(numbers)           # [1, 2, 3, 4, 5, 6]

# Dictionaries
let person = {
    "name": "Alice",
    "age": 30,
    "city": "NYC"
}

println(person["name"])    # Alice
person["age"] = 31         # Update value
```

### Error Handling

```veyra
fn safe_divide(a, b) {
    try {
        if b == 0 {
            throw("Division by zero!")
        }
        return a / b
    } catch error {
        println("Error: " + error)
        return null
    } finally {
        println("Division operation completed")
    }
}

let result = safe_divide(10, 2)
println(result)  # 5
```

## 🛠️ Using the Tools

### Linter

Check your code for issues:

```bash
./tools/target/release/veyra-lint myprogram.vey
```

### Debugger

Debug your program:

```bash
./tools/target/release/veyra-dbg myprogram.vey
```

Debugger commands:
- `break <line>` - Set breakpoint
- `continue` - Continue execution
- `step` - Step to next line
- `print <var>` - Print variable value
- `quit` - Exit debugger

### VS Code Integration

1. Open VS Code
2. Go to Extensions
3. Install from `tools/vscode_extension`
4. Enjoy syntax highlighting and IntelliSense!

## 📚 Next Steps

### Learn More

- **[Language Specification](spec/LANGUAGE_SPEC.md)** - Complete language reference
- **[Examples](examples/)** - More code examples
- **[Design Philosophy](docs/DESIGN_PHILOSOPHY.md)** - Language design principles

### Example Programs

Check out these examples:

```bash
# Hello World
./compiler/target/release/veyra examples/hello_world.vey

# Functions
./compiler/target/release/veyra examples/functions.vey

# Loops
./compiler/target/release/veyra examples/loops.vey

# Math Demo
./compiler/target/release/veyra examples/math_demo.vey
```

### Standard Library

Veyra includes a rich standard library:

```veyra
# Math operations
import math
println(math.sqrt(16))      # 4
println(math.pow(2, 8))     # 256
println(math.sin(math.pi))  # ~0

# String operations
import string
let text = "Hello, World!"
println(string.upper(text))          # HELLO, WORLD!
println(string.split(text, ","))     # ["Hello", " World!"]

# Collections
import collections
let items = [3, 1, 4, 1, 5, 9]
println(collections.sort(items))     # [1, 1, 3, 4, 5, 9]
println(collections.max(items))      # 9
```

## 🐛 Troubleshooting

### Common Issues

**Problem**: `cargo: command not found`
- **Solution**: Install Rust from https://rustup.rs/

**Problem**: Build fails with linking errors
- **Solution**: Make sure you have the Visual Studio Build Tools installed (Windows) or build-essential (Linux)

**Problem**: REPL doesn't show output
- **Solution**: Make sure you're using `println()` to display values

### Getting Help

- 📖 [Documentation](spec/LANGUAGE_SPEC.md)
- 💬 [GitHub Discussions](https://github.com/k6w/veyra/discussions)
- 🐛 [Report Issues](https://github.com/k6w/veyra/issues)

## 🎓 Tutorial: Building a Calculator

Let's build a simple calculator to learn Veyra:

```veyra
# calculator.vey

fn add(a, b) { return a + b }
fn subtract(a, b) { return a - b }
fn multiply(a, b) { return a * b }
fn divide(a, b) {
    if b == 0 {
        println("Error: Division by zero")
        return null
    }
    return a / b
}

fn calculate(op, a, b) {
    if op == "+" {
        return add(a, b)
    } elif op == "-" {
        return subtract(a, b)
    } elif op == "*" {
        return multiply(a, b)
    } elif op == "/" {
        return divide(a, b)
    } else {
        println("Unknown operation: " + op)
        return null
    }
}

# Test the calculator
println("5 + 3 = " + str(calculate("+", 5, 3)))
println("10 - 4 = " + str(calculate("-", 10, 4)))
println("6 * 7 = " + str(calculate("*", 6, 7)))
println("20 / 4 = " + str(calculate("/", 20, 4)))
```

Run it:

```bash
./compiler/target/release/veyra calculator.vey
```

Output:
```
5 + 3 = 8
10 - 4 = 6
6 * 7 = 42
20 / 4 = 5
```

## 🚀 Ready to Code!

You're now ready to start coding in Veyra! Here are some ideas:

1. **Calculator** - Build a more advanced calculator
2. **To-Do List** - Create a task manager
3. **Data Analysis** - Process and analyze data
4. **Web Scraper** - Extract data from websites
5. **Game** - Build a text-based game

Happy coding! 🎉
