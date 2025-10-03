# Veyra Example Programs

This directory contains example programs demonstrating various features of the Veyra programming language.

## 🎓 Getting Started

### Basic Examples

Start with these if you're new to Veyra:

- **[hello_world.vey](hello_world.vey)** - Classic Hello World program
- **[variables.vey](variables.vey)** - Variable declarations and types
- **[arithmetic.vey](arithmetic.vey)** - Basic arithmetic operations
- **[functions.vey](functions.vey)** - Function definitions and calls
- **[loops.vey](loops.vey)** - While and for loops
- **[arrays.vey](arrays.vey)** - Working with arrays

## 📚 Intermediate Examples

Once you're comfortable with the basics:

- **[math_demo.vey](math_demo.vey)** - Mathematical operations and functions
- **[text_processing.vey](text_processing.vey)** - String manipulation
- **[stdlib_simple.vey](stdlib_simple.vey)** - Using the standard library
- **[pattern_matching_exceptions.vey](pattern_matching_exceptions.vey)** - Error handling

## 🚀 Advanced Examples

For experienced developers:

- **[advanced_calculator.vey](advanced_calculator.vey)** - Complex calculator with error handling
- **[advanced_data_structures.vey](advanced_data_structures.vey)** - Custom data structures
- **[advanced_features.vey](advanced_features.vey)** - Advanced language features
- **[oop_features.vey](oop_features.vey)** - Object-oriented programming patterns
- **[async_concurrency.vey](async_concurrency.vey)** - Async/await and concurrency
- **[concurrency.vey](concurrency.vey)** - Parallel execution
- **[ownership.vey](ownership.vey)** - Memory management patterns
- **[performance_optimization.vey](performance_optimization.vey)** - Performance techniques

## 🔬 Real-World Applications

- **[data_analysis.vey](data_analysis.vey)** - Data processing and analysis
- **[simple_analysis.vey](simple_analysis.vey)** - Statistical analysis

## 🎯 Running Examples

### Using the Compiler

```bash
# From the project root
./compiler/target/release/veyra examples/hello_world.vey

# Or navigate to examples directory
cd examples
../compiler/target/release/veyra hello_world.vey
```

### Using the REPL

```bash
# Start the REPL
./tools/target/release/veyra-repl

# Load and run an example
>>> import "examples/hello_world.vey"
```

## 📖 What You'll Learn

### Basic Concepts
- Variable declarations
- Data types (integers, floats, strings, booleans)
- Arithmetic and logical operators
- Control flow (if/elif/else)
- Loops (while, for-in)

### Functions
- Function definitions
- Parameters and return values
- Recursion
- Higher-order functions
- Closures

### Data Structures
- Arrays and array operations
- Dictionaries/maps
- Nested structures
- Iteration

### Advanced Features
- Error handling (try-catch-finally)
- Module system and imports
- Object-oriented patterns
- Async/await
- Concurrency and parallelism

## 🛠️ Modifying Examples

Feel free to modify these examples to experiment with the language:

1. Copy an example to a new file
2. Make your changes
3. Run it with the compiler
4. See what happens!

```bash
# Copy an example
cp examples/hello_world.vey my_experiment.vey

# Edit it
# ... make changes ...

# Run it
./compiler/target/release/veyra my_experiment.vey
```

## 💡 Tips

- Start with simple examples and work your way up
- Read the comments in each file for explanations
- Try modifying the examples to see how things work
- Check the [Language Specification](../spec/LANGUAGE_SPEC.md) for details
- Use the REPL for quick experiments

## 🐛 Found an Issue?

If you find a problem with an example:
1. Check the [Language Specification](../spec/LANGUAGE_SPEC.md)
2. Try running it in the REPL for debugging
3. Report issues on [GitHub](https://github.com/k6w/veyra/issues)

## 🤝 Contributing Examples

Want to add your own example? Great!

1. Create a new `.vey` file in this directory
2. Add clear comments explaining what it does
3. Test it thoroughly
4. Update this README with your example
5. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

---

Happy learning! 🚀
