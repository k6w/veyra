# Veyra Language Specification (Complete)
## Version 1.0

### Table of Contents
1. [Introduction](#introduction)
2. [Lexical Structure](#lexical-structure)
3. [Type System](#type-system)
4. [Syntax and Grammar](#syntax-and-grammar)
5. [Object-Oriented Programming](#object-oriented-programming)
6. [Concurrency and Async Programming](#concurrency-and-async-programming)
7. [Pattern Matching](#pattern-matching)
8. [Memory Management](#memory-management)
9. [Standard Library](#standard-library)
10. [Runtime System](#runtime-system)
11. [Interoperability](#interoperability)

### Introduction

Veyra is a modern, statically-typed programming language designed for performance, safety, and developer productivity. It combines the best features of systems programming languages with high-level abstractions for concurrent and asynchronous programming.

#### Design Goals
- **Memory Safety**: Compile-time memory safety without garbage collection overhead
- **Performance**: Zero-cost abstractions and optional JIT compilation
- **Concurrency**: Built-in async/await and actor model support
- **Expressiveness**: Powerful type system with generics and pattern matching
- **Interoperability**: Seamless integration with existing ecosystems

### Lexical Structure

#### Keywords
```
fn class enum trait impl where async await
let mut const static pub private protected
if else match while for loop break continue return
true false null self Self super
try catch finally throw
import export module use as
type alias union struct
atomic unsafe extern
yield spawn select timeout
```

#### Operators
```
Arithmetic: + - * / % ** (power)
Comparison: == != < <= > >=
Logical: && || ! 
Bitwise: & | ^ ~ << >>
Assignment: = += -= *= /= %= &= |= ^= <<= >>=
Ownership: move copy clone
Async: await async
Pattern: => | @ .. ...
Access: . :: -> ?. (safe navigation)
Range: .. ..= (inclusive)
```

#### Literals
```veyra
// Numbers
42          // i64
42i32       // i32
42u64       // u64
3.14        // f64
3.14f32     // f32
0x2A        // hex
0o52        // octal
0b101010    // binary

// Strings
"hello"         // string literal
'c'             // character literal
r"raw string"   // raw string
"""
multiline
string
"""

// Arrays and collections
[1, 2, 3]                    // array
{"key": "value"}             // map
#{1, 2, 3}                   // set
```

### Type System

#### Primitive Types
```veyra
// Integer types
i8, i16, i32, i64, i128     // signed integers
u8, u16, u32, u64, u128     // unsigned integers
isize, usize                // pointer-sized integers

// Floating point
f32, f64                    // IEEE 754 floating point

// Other primitives
bool                        // boolean
char                        // Unicode character
str                         // string slice
String                      // owned string
```

#### Composite Types
```veyra
// Arrays
array[T]                    // dynamic array
[T; N]                      // fixed-size array

// Tuples
(T1, T2, ...)              // tuple types

// References
&T                          // immutable reference
&mut T                      // mutable reference

// Options and Results
Option[T]                   // nullable type
Result[T, E]                // error handling
```

#### Generic Types
```veyra
// Generic functions
fn map[T, R](items: array[T], transform: fn(T) -> R) -> array[R] {
    let result = array::new();
    for item in items {
        result.push(transform(item));
    }
    return result;
}

// Generic classes
class Vector[T] {
    private data: array[T];
    
    fn new() -> Vector[T] {
        return Vector { data: array::new() };
    }
    
    fn push(item: T) {
        self.data.push(item);
    }
    
    fn get(index: i64) -> Option[T] {
        return self.data.get(index);
    }
}

// Bounded generics
trait Comparable {
    fn compare(self, other: Self) -> i32;
}

fn sort[T](items: array[T]) where T: Comparable {
    // Sort implementation using T.compare
}
```

#### Traits
```veyra
// Trait definition
trait Display {
    fn to_string(self) -> String;
}

trait Debug {
    fn debug_string(self) -> String;
}

// Trait implementation
impl Display for i32 {
    fn to_string(self) -> String {
        return int_to_string(self);
    }
}

// Generic trait implementation
impl[T] Display for Vector[T] where T: Display {
    fn to_string(self) -> String {
        let result = "[";
        for i in 0..self.len() {
            if i > 0 {
                result += ", ";
            }
            result += self.get(i).unwrap().to_string();
        }
        result += "]";
        return result;
    }
}

// Trait objects
trait Drawable {
    fn draw(self);
}

fn render_all(objects: array[Box[dyn Drawable]]) {
    for obj in objects {
        obj.draw();
    }
}
```

### Object-Oriented Programming

#### Classes and Inheritance
```veyra
// Base class
class Animal {
    protected name: String;
    protected age: i32;
    
    fn new(name: String, age: i32) -> Animal {
        return Animal { name, age };
    }
    
    fn get_name() -> String {
        return self.name;
    }
    
    // Virtual method
    virtual fn make_sound() -> String {
        return "Some generic animal sound";
    }
    
    // Abstract method
    abstract fn get_species() -> String;
}

// Derived class
class Dog extends Animal {
    private breed: String;
    
    fn new(name: String, age: i32, breed: String) -> Dog {
        let dog = Dog { 
            name, 
            age, 
            breed 
        };
        return dog;
    }
    
    // Override virtual method
    override fn make_sound() -> String {
        return "Woof!";
    }
    
    // Implement abstract method
    fn get_species() -> String {
        return "Canis lupus";
    }
    
    fn get_breed() -> String {
        return self.breed;
    }
}

// Multiple inheritance through traits
trait Flyable {
    fn fly(self);
    fn get_max_altitude() -> f64;
}

trait Swimmable {
    fn swim(self);
    fn get_max_depth() -> f64;
}

class Duck extends Animal implements Flyable, Swimmable {
    fn new(name: String, age: i32) -> Duck {
        return Duck { name, age };
    }
    
    fn make_sound() -> String {
        return "Quack!";
    }
    
    fn get_species() -> String {
        return "Anas platyrhynchos";
    }
    
    fn fly(self) {
        println("{} is flying!", self.name);
    }
    
    fn get_max_altitude() -> f64 {
        return 1000.0;
    }
    
    fn swim(self) {
        println("{} is swimming!", self.name);
    }
    
    fn get_max_depth() -> f64 {
        return 10.0;
    }
}
```

#### Access Control
```veyra
class BankAccount {
    private balance: f64;           // Only accessible within class
    protected account_id: String;   // Accessible in subclasses
    public owner: String;           // Publicly accessible
    
    fn new(owner: String, initial_balance: f64) -> BankAccount {
        return BankAccount {
            balance: initial_balance,
            account_id: generate_account_id(),
            owner
        };
    }
    
    // Public method
    pub fn get_balance() -> f64 {
        return self.balance;
    }
    
    // Private method
    private fn validate_transaction(amount: f64) -> bool {
        return amount > 0.0 && amount <= self.balance;
    }
    
    // Protected method
    protected fn update_balance(amount: f64) {
        self.balance += amount;
    }
}
```

### Concurrency and Async Programming

#### Async/Await
```veyra
// Async function
async fn fetch_data(url: String) -> Result[String, HttpError] {
    let response = await http::get(url)?;
    let body = await response.text()?;
    return Ok(body);
}

// Using async functions
fn main() {
    let future = async {
        let data = await fetch_data("https://api.example.com/data")?;
        let processed = process_data(data);
        return processed;
    };
    
    let result = runtime::block_on(future);
    match result {
        Ok(data) => println("Processed data: {}", data),
        Err(error) => println("Error: {}", error)
    }
}

// Parallel execution
async fn fetch_multiple_urls(urls: array[String]) -> array[Result[String, HttpError]] {
    let futures = urls.map(|url| fetch_data(url));
    return await Promise::all(futures);
}

// Async streams
async fn process_stream[T](stream: AsyncStream[T], processor: fn(T) -> T) -> AsyncStream[T] {
    return stream.map(processor).buffer(100);
}
```

#### Channels and Message Passing
```veyra
// Channel creation
let (sender, receiver) = channel::new[String](100); // buffered channel

// Sending messages
async fn producer(sender: ChannelSender[String]) {
    for i in 0..10 {
        await sender.send(format!("Message {}", i))?;
        await sleep(Duration::from_millis(100));
    }
    sender.close();
}

// Receiving messages
async fn consumer(receiver: ChannelReceiver[String]) {
    while let Some(message) = await receiver.receive() {
        println("Received: {}", message);
    }
}

// Select statement for multiple channels
async fn multiplexer(
    ch1: ChannelReceiver[String],
    ch2: ChannelReceiver[i32],
    output: ChannelSender[String]
) {
    loop {
        select {
            msg = ch1.receive() => {
                if let Some(s) = msg {
                    await output.send(format!("String: {}", s))?;
                } else {
                    break;
                }
            },
            num = ch2.receive() => {
                if let Some(n) = num {
                    await output.send(format!("Number: {}", n))?;
                } else {
                    break;
                }
            },
            timeout(Duration::from_secs(1)) => {
                await output.send("Timeout!".to_string())?;
            }
        }
    }
}
```

#### Actor Model
```veyra
// Actor definition
enum BankActorMessage {
    Deposit(f64, ChannelSender[bool]),
    Withdraw(f64, ChannelSender[bool]),
    GetBalance(ChannelSender[f64]),
    Close
}

struct BankActor {
    balance: f64;
}

impl Actor[BankActor, BankActorMessage] for BankActor {
    fn new(initial_balance: f64) -> BankActor {
        return BankActor { balance: initial_balance };
    }
    
    async fn handle(mut self, message: BankActorMessage) -> BankActor {
        match message {
            BankActorMessage::Deposit(amount, reply) => {
                self.balance += amount;
                let _ = reply.send(true);
            },
            BankActorMessage::Withdraw(amount, reply) => {
                if self.balance >= amount {
                    self.balance -= amount;
                    let _ = reply.send(true);
                } else {
                    let _ = reply.send(false);
                }
            },
            BankActorMessage::GetBalance(reply) => {
                let _ = reply.send(self.balance);
            },
            BankActorMessage::Close => {
                // Actor will stop
            }
        }
        return self;
    }
}

// Using actors
async fn banking_example() {
    let bank_actor = BankActor::spawn(1000.0);
    
    // Deposit money
    let (reply_sender, reply_receiver) = channel::new(1);
    bank_actor.send(BankActorMessage::Deposit(500.0, reply_sender)).await?;
    let success = reply_receiver.receive().await.unwrap();
    
    // Get balance
    let (balance_sender, balance_receiver) = channel::new(1);
    bank_actor.send(BankActorMessage::GetBalance(balance_sender)).await?;
    let balance = balance_receiver.receive().await.unwrap();
    
    println("Current balance: {}", balance);
}
```

### Pattern Matching

#### Basic Pattern Matching
```veyra
enum Color {
    Red,
    Green,
    Blue,
    RGB(u8, u8, u8),
    HSL { hue: f32, saturation: f32, lightness: f32 }
}

fn describe_color(color: Color) -> String {
    match color {
        Color::Red => "Pure red",
        Color::Green => "Pure green", 
        Color::Blue => "Pure blue",
        Color::RGB(r, g, b) => format!("RGB({}, {}, {})", r, g, b),
        Color::HSL { hue, saturation, lightness } => {
            format!("HSL({}°, {}%, {}%)", hue, saturation * 100.0, lightness * 100.0)
        }
    }
}

// Guard clauses
fn categorize_number(n: i32) -> String {
    match n {
        x if x < 0 => "Negative",
        0 => "Zero",
        1..=10 => "Small positive",
        11..=100 => "Medium positive", 
        x if x > 100 => "Large positive",
        _ => "Unknown"
    }
}

// Destructuring
struct Point { x: f64, y: f64 }

fn analyze_point(point: Point) -> String {
    match point {
        Point { x: 0.0, y: 0.0 } => "Origin",
        Point { x, y: 0.0 } => format!("On X-axis at x={}", x),
        Point { x: 0.0, y } => format!("On Y-axis at y={}", y),
        Point { x, y } if x == y => format!("On diagonal at ({}, {})", x, y),
        Point { x, y } => format!("Point at ({}, {})", x, y)
    }
}
```

#### Advanced Pattern Matching
```veyra
// Nested patterns
enum Expression {
    Number(f64),
    Variable(String),
    Add(Box[Expression], Box[Expression]),
    Multiply(Box[Expression], Box[Expression])
}

fn evaluate(expr: Expression, vars: HashMap[String, f64]) -> Option[f64] {
    match expr {
        Expression::Number(n) => Some(n),
        Expression::Variable(name) => vars.get(&name).copied(),
        Expression::Add(left, right) => {
            match (evaluate(*left, vars), evaluate(*right, vars)) {
                (Some(a), Some(b)) => Some(a + b),
                _ => None
            }
        },
        Expression::Multiply(left, right) => {
            match (evaluate(*left, vars), evaluate(*right, vars)) {
                (Some(a), Some(b)) => Some(a * b),
                _ => None
            }
        }
    }
}

// Pattern matching with async
async fn handle_request(request: HttpRequest) -> HttpResponse {
    match request {
        HttpRequest { 
            method: HttpMethod::GET, 
            path: "/users" 
        } => {
            let users = await database.get_all_users();
            HttpResponse::json(users)
        },
        HttpRequest { 
            method: HttpMethod::POST, 
            path: "/users",
            body: Some(user_data) 
        } => {
            let user = await database.create_user(user_data);
            HttpResponse::created(user)
        },
        HttpRequest { 
            method: HttpMethod::GET, 
            path: path @ "/users/{id}" 
        } => {
            let id = extract_id(path);
            let user = await database.get_user(id);
            match user {
                Some(u) => HttpResponse::json(u),
                None => HttpResponse::not_found()
            }
        },
        _ => HttpResponse::method_not_allowed()
    }
}
```

### Memory Management

#### Ownership and Borrowing
```veyra
// Ownership transfer
fn take_ownership(s: String) -> String {
    println("I own: {}", s);
    return s; // ownership transferred back
}

// Borrowing
fn borrow_immutable(s: &String) {
    println("Borrowed: {}", s);
    // s cannot be modified
}

fn borrow_mutable(s: &mut String) {
    s.push_str(" (modified)");
    println("Modified: {}", s);
}

// Lifetime annotations
fn longest[a](x: &a str, y: &a str) -> &a str {
    if x.len() > y.len() {
        return x;
    } else {
        return y;
    }
}

// RAII and destructors
class FileHandle {
    private file: File;
    
    fn new(path: String) -> Result[FileHandle, IoError] {
        let file = File::open(path)?;
        return Ok(FileHandle { file });
    }
    
    fn write(data: &str) -> Result[(), IoError] {
        return self.file.write(data);
    }
    
    // Destructor called automatically
    fn drop() {
        self.file.close();
        println("File closed automatically");
    }
}
```

#### Smart Pointers
```veyra
// Reference counted pointer
let shared_data = Rc::new("shared string");
let clone1 = shared_data.clone();
let clone2 = shared_data.clone();

// Atomic reference counting for thread safety
let thread_safe_data = Arc::new(Mutex::new(42));
let data_clone = thread_safe_data.clone();

spawn(move || {
    let mut value = data_clone.lock().unwrap();
    *value += 1;
});

// Weak references to break cycles
class Node {
    value: i32;
    children: Vector[Rc[Node]];
    parent: Weak[Node]; // Weak reference to prevent cycles
}
```

### Runtime System

#### JIT Compilation
```veyra
// Hot function that will be JIT compiled
@jit_threshold(100)
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// Manual JIT compilation
fn compile_hot_path() {
    runtime::jit_compile("hot_computation_function");
}

// Performance hints
@inline
fn small_function() -> i32 {
    return 42;
}

@noinline
fn large_function() {
    // Complex implementation
}

@optimize(speed)
fn performance_critical() {
    // Will be optimized for speed
}

@optimize(size)
fn size_critical() {
    // Will be optimized for code size
}
```

#### Garbage Collection Control
```veyra
// Manual GC control
fn memory_intensive_operation() {
    let large_data = allocate_large_buffer();
    
    // Process data
    process_data(large_data);
    
    // Suggest garbage collection
    runtime::gc_suggest();
    
    // Force garbage collection (use sparingly)
    runtime::gc_force();
}

// GC-free regions
@nogc
fn realtime_function() {
    // This function cannot allocate on the GC heap
    // Only stack allocation and pre-allocated pools allowed
    let stack_array = [1, 2, 3, 4, 5]; // Stack allocated
}
```

### Standard Library

#### Collections
```veyra
// Vector (dynamic array)
let mut vec = Vector::new();
vec.push(1);
vec.push(2);
vec.push(3);

// HashMap
let mut map = HashMap::new();
map.insert("key1", "value1");
map.insert("key2", "value2");

// HashSet
let mut set = HashSet::new();
set.insert(1);
set.insert(2);

// BTreeMap (ordered map)
let mut btree = BTreeMap::new();
btree.insert(3, "three");
btree.insert(1, "one");
btree.insert(2, "two");

// LinkedList
let mut list = LinkedList::new();
list.push_back(1);
list.push_front(0);
```

#### Functional Programming
```veyra
// Iterator chains
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let result = numbers
    .iter()
    .filter(|&x| x % 2 == 0)      // Even numbers
    .map(|&x| x * x)               // Square them
    .fold(0, |acc, x| acc + x);    // Sum them up

// Lazy evaluation
let lazy_result = numbers
    .iter()
    .lazy()
    .map(expensive_computation)
    .take(5)
    .collect();

// Parallel processing
let parallel_result = numbers
    .par_iter()
    .map(|&x| expensive_computation(x))
    .collect();
```

### Interoperability

#### C FFI
```veyra
// External C function
extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn printf(format: *const i8, ...) -> i32;
}

// Calling C functions
unsafe fn allocate_buffer(size: usize) -> *mut u8 {
    return malloc(size);
}

// Exporting Veyra functions to C
@export("add_numbers")
extern "C" fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

#### JavaScript Interop (WebAssembly)
```veyra
// Import JavaScript function
@wasm_import("console", "log")
extern fn js_log(message: &str);

// Export to JavaScript
@wasm_export
fn fibonacci_wasm(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci_wasm(n - 1) + fibonacci_wasm(n - 2);
}

// JavaScript Promise integration
@wasm_async
async fn fetch_data_wasm(url: &str) -> Result[String, JsError] {
    let response = await js_fetch(url)?;
    let text = await js_response_text(response)?;
    return Ok(text);
}
```

### Error Handling

#### Result Type
```veyra
enum Result[T, E] {
    Ok(T),
    Err(E)
}

// Error propagation with ?
fn read_file_content(path: &str) -> Result[String, IoError] {
    let file = File::open(path)?;
    let content = file.read_to_string()?;
    return Ok(content);
}

// Custom error types
enum MyError {
    InvalidInput(String),
    NetworkError(String),
    DatabaseError(String)
}

impl Display for MyError {
    fn to_string(self) -> String {
        match self {
            MyError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            MyError::NetworkError(msg) => format!("Network error: {}", msg),
            MyError::DatabaseError(msg) => format!("Database error: {}", msg)
        }
    }
}
```

#### Try-Catch (Alternative Error Handling)
```veyra
// Traditional exception handling
fn risky_operation() throws NetworkError, DatabaseError {
    let data = fetch_from_network()?;  // Can throw NetworkError
    save_to_database(data)?;           // Can throw DatabaseError
}

fn main() {
    try {
        risky_operation();
        println("Operation successful");
    } catch NetworkError(e) {
        println("Network failed: {}", e);
    } catch DatabaseError(e) {
        println("Database failed: {}", e);
    } finally {
        cleanup_resources();
    }
}
```

This completes the comprehensive Veyra Language Specification, covering all major features from basic syntax to advanced concurrent programming, memory management, and interoperability.