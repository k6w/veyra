# Veyra Language Specification v1.0

**Status**: Production Ready  
**Last Updated**: September 28, 2025

## Table of Contents
1. [Overview](#overview)
2. [Lexical Structure](#lexical-structure)
3. [Data Types](#data-types)
4. [Variables and Assignment](#variables-and-assignment)
5. [Operators](#operators)
6. [Expressions](#expressions)
7. [Control Flow](#control-flow)
8. [Functions](#functions)
9. [Arrays](#arrays)
10. [Module System](#module-system)
11. [Built-in Functions](#built-in-functions)
12. [Error Handling](#error-handling)

## Overview

Veyra is a dynamically-typed, interpreted programming language designed for clarity, simplicity, and rapid development. It features:

- **Dynamic typing** with runtime type checking
- **Interpreted execution** for fast development cycles
- **Rich operator set** including compound assignments and power operator
- **Complete control flow** with if/elif/else, while, and for-in loops
- **Function definitions** with parameter passing and return values
- **Recursion support** for algorithmic programming
- **Module system** with importable standard libraries
- **Array operations** with built-in manipulation functions
- **Error handling** with comprehensive runtime error reporting

## Lexical Structure

### Comments
```veyra
# Single line comment - everything after # is ignored
```

### Keywords
Reserved words in Veyra:
```
and, break, continue, elif, else, false, fn, for, if, import, in, let, None, not, or, pub, return, true, while
```

### Identifiers
- Start with letter or underscore
- Followed by letters, digits, or underscores
- Case-sensitive
- Cannot be keywords

Valid identifiers: `name`, `_private`, `calculate_total`, `item2`, `PI`, `userName`

### Literals

#### Integer Literals
```veyra
42      # Positive integer
-17     # Negative integer
0       # Zero
123456  # Large integer
```

#### Float Literals
```veyra
3.14159   # Standard float
2.718     # Float with decimal
0.5       # Less than one
-9.81     # Negative float
```

#### String Literals
```veyra
"Hello, World!"       # Double-quoted string
"Veyra Programming"   # String with spaces
"Line 1\nLine 2"      # String with escape sequences
""                    # Empty string
```

#### Boolean Literals
```veyra
true    # Boolean true
false   # Boolean false
```

#### None Literal
```veyra
None    # Represents null/empty value
```

### Operators and Punctuation
```
# Arithmetic
+  -  *  /  %  **

# Comparison  
==  !=  <  <=  >  >=

# Logical
and  or  not

# Assignment
=  +=  -=  *=  /=  %=

# Unary
-  not

# Grouping and Delimiters
()  []  {}  ,  :
```

## Data Types

### Primitive Types

#### Integer (`int`)
64-bit signed integers supporting full arithmetic operations.
```veyra
let age = 25
let temperature = -10
let large_number = 1000000
```

#### Float (`float`) 
64-bit floating-point numbers supporting mathematical operations.
```veyra
let pi = 3.14159
let rate = 0.05
let measurement = -2.718
```

#### String (`string`)
UTF-8 encoded text strings with concatenation support.
```veyra
let name = "Alice"
let greeting = "Hello, " + name + "!"
let multiword = "Veyra Programming Language"
```

#### Boolean (`bool`)
Logical true/false values.
```veyra
let is_ready = true
let is_finished = false
let condition_result = (5 > 3)  # true
```

#### None (`None`)
Represents absence of value or null state.
```veyra
let empty_value = None
let uninitialized = None
```

### Composite Types

#### Arrays
Ordered collections supporting mixed types.
```veyra
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]
let mixed = [42, "hello", true, 3.14, None]
let empty_array = []
```

## Variables and Assignment

### Variable Declaration
```veyra
let variable_name = expression
```

### Assignment
```veyra
variable_name = new_value
```

### Examples
```veyra
# Basic assignment
let count = 10
let message = "Hello"
let is_active = true

# Reassignment
count = 20
message = "Updated message"

# Expression assignment
let sum = 5 + 3
let full_name = first_name + " " + last_name
```

## Operators

### Arithmetic Operators
| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Addition | `5 + 3` | `8` |
| `-` | Subtraction | `5 - 3` | `2` |
| `*` | Multiplication | `5 * 3` | `15` |
| `/` | Division | `15 / 3` | `5` |
| `%` | Modulo | `15 % 4` | `3` |
| `**` | Power/Exponentiation | `2 ** 3` | `8` |

### Comparison Operators
| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `==` | Equal | `5 == 5` | `true` |
| `!=` | Not equal | `5 != 3` | `true` |
| `<` | Less than | `3 < 5` | `true` |
| `<=` | Less than or equal | `5 <= 5` | `true` |
| `>` | Greater than | `5 > 3` | `true` |
| `>=` | Greater than or equal | `5 >= 5` | `true` |

### Logical Operators
| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `and` | Logical AND | `true and false` | `false` |
| `or` | Logical OR | `true or false` | `true` |
| `not` | Logical NOT | `not true` | `false` |

### Assignment Operators
| Operator | Description | Example | Equivalent |
|----------|-------------|---------|------------|
| `=` | Assignment | `x = 5` | `x = 5` |
| `+=` | Add and assign | `x += 3` | `x = x + 3` |
| `-=` | Subtract and assign | `x -= 3` | `x = x - 3` |
| `*=` | Multiply and assign | `x *= 3` | `x = x * 3` |
| `/=` | Divide and assign | `x /= 3` | `x = x / 3` |
| `%=` | Modulo and assign | `x %= 3` | `x = x % 3` |

### Unary Operators
| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `-` | Arithmetic negation | `-5` | `-5` |
| `not` | Logical negation | `not true` | `false` |

### Operator Precedence (Highest to Lowest)
1. **Unary**: `not`, `-` (unary minus)
2. **Power**: `**`
3. **Multiplicative**: `*`, `/`, `%`
4. **Additive**: `+`, `-`
5. **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
6. **Logical AND**: `and`
7. **Logical OR**: `or`
8. **Assignment**: `=`, `+=`, `-=`, `*=`, `/=`, `%=`

## Types

### Primitive Types
```veyra
int     # 64-bit signed integer
i32     # 32-bit signed integer  
i64     # 64-bit signed integer
u32     # 32-bit unsigned integer
u64     # 64-bit unsigned integer
f32     # 32-bit floating point
f64     # 64-bit floating point
bool    # Boolean
char    # Unicode character
string  # UTF-8 string
```

### Collection Types
```veyra
[T]              # Array of T
[T; N]           # Fixed-size array of N elements
HashMap<K, V>    # Hash map
Set<T>           # Set
```

### Optional Types
```veyra
T?               # Optional T (Some(T) or None)
```

### Function Types
```veyra
fn(int, string) -> bool     # Function type
```

### Reference Types
```veyra
&T               # Immutable reference to T
&mut T           # Mutable reference to T
```

## Expressions

### Literals
```veyra
42
3.14
"hello"
true
```

### Variables
```veyra
x               # Variable reference
self            # Self reference in methods
```

### Function Calls
```veyra
add(5, 3)
person.greet()
Math.sqrt(16)
```

### Array/Index Access
```veyra
arr[0]
matrix[i][j]
```

### Field Access
```veyra
person.name
person.address.street
```

### Method Calls
```veyra
string.length()
list.push(item)
```

### Binary Operations
```veyra
a + b
x == y
flag and condition
```

### Unary Operations
```veyra
-x
not flag
```

### Range Expressions
```veyra
0..10           # Exclusive range
0..=10          # Inclusive range
```

### Match Expressions
```veyra
match value
    1 -> "one"
    2 -> "two"
    _ -> "other"
```

### If Expressions
```veyra
if condition then "yes" else "no"
```

## Statements

### Expression Statements
```veyra
print("Hello")
x + y
```

### Variable Declarations
```veyra
let x = 42                  # Immutable
let mut y = 0              # Mutable
let name: string = "Alice"  # With type annotation
```

### Assignment
```veyra
x = 42
arr[0] = value
person.name = "Bob"
```

### If Statements
```veyra
if condition
    do_something()
elif other_condition
    do_other()
else
    default_action()
```

### While Loops
```veyra
while condition
    process()
```

### For Loops
```veyra
for item in collection
    process(item)

for i in 0..10
    process(i)
```

### Match Statements
```veyra
match expression
    pattern1 -> statement1
    pattern2 -> statement2
    _ -> default_statement
```

### Return Statements
```veyra
return
return value
return Ok(result)
```

### Break and Continue
```veyra
break
continue
```

## Functions

### Function Declaration
```veyra
fn name(param1: Type1, param2: Type2) -> ReturnType
    # function body
    return value
```

### Examples
```veyra
# Simple function
fn add(a: int, b: int) -> int
    return a + b

# Function with type inference
fn multiply(x, y)
    return x * y

# Function with default parameters
fn greet(name: string, greeting: string = "Hello")
    print("{greeting}, {name}!")

# Async function
async fn fetch_data(url: string) -> Result<string, Error>
    response = await http.get(url)
    return response.text()
```

### Method Syntax
```veyra
impl Person
    fn greet(self) -> string
        return "Hello, I'm {self.name}"
    
    fn birthday(mut self)
        self.age += 1
```

## Structs and Implementations

### Struct Definition
```veyra
struct Person
    name: string
    age: int
    email: string?
```

### Struct Construction
```veyra
person = Person {
    name: "Alice",
    age: 30,
    email: Some("alice@example.com")
}

# Shorthand when variable names match fields
name = "Bob"
age = 25
person = Person { name, age, email: None }
```

### Implementation Blocks
```veyra
impl Person
    fn new(name: string, age: int) -> Person
        return Person { name, age, email: None }
    
    fn greet(self) -> string
        return "Hello, I'm {self.name}"
```

## Ownership and Borrowing

### Ownership Rules
1. Each value has a single owner
2. When the owner goes out of scope, the value is dropped
3. Ownership can be transferred (moved)
4. Values can be borrowed immutably or mutably

### Examples
```veyra
# Ownership transfer
let s1 = "hello"
let s2 = s1     # s1 is moved to s2, s1 is no longer valid

# Borrowing
fn process(s: &string)  # Immutable borrow
    print(s)

fn modify(s: &mut string)  # Mutable borrow
    s.push("!")

let mut text = "hello"
process(&text)      # Borrow immutably
modify(&mut text)   # Borrow mutably
```

## Concurrency

### Async/Await
```veyra
async fn fetch_data() -> string
    response = await http.get("https://api.example.com")
    return response.text()

# Using async functions
async fn main()
    data = await fetch_data()
    print(data)
```

### Actors
```veyra
actor Counter
    state: int = 0
    
    fn increment() -> int
        state += 1
        return state
    
    fn get() -> int
        return state

# Using actors
counter = Counter.spawn()
count = await counter.increment()
```

### Channels
```veyra
channel = Channel<string>.new()

# Sender
spawn fn()
    channel.send("Hello")
    channel.send("World")
    channel.close()

# Receiver
for message in channel
    print(message)
```

## Module System

### Importing
```veyra
import std.collections.HashMap
import std.io.{File, BufReader}
import std.net as network
```

### Exporting
```veyra
# Public by default
fn public_function()
    # ...

# Private with underscore prefix
fn _private_function()
    # ...

# Explicit visibility (future feature)
pub fn explicitly_public()
    # ...
```

---

*This specification is a living document and will evolve as the language develops.*