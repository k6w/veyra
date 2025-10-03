use anyhow::Result;
use clap::Parser;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;

// Import from the main compiler
use veyra_compiler::{
    lexer::Lexer,
    parser::Parser as VeyraParser,
    interpreter::{Interpreter, Value},
};

#[derive(Parser)]
#[command(name = "veyra-repl")]
#[command(about = "Interactive REPL for the Veyra programming language")]
#[command(version = "0.1.0")]
struct Cli {
    /// Show version information
    #[arg(short, long)]
    version: bool,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Load and execute a startup file
    #[arg(short, long)]
    startup: Option<PathBuf>,
}

struct ReplState {
    interpreter: Interpreter,
    history: Vec<String>,
    verbose: bool,
}

impl ReplState {
    fn new(verbose: bool) -> Self {
        Self {
            interpreter: Interpreter::new(),
            history: Vec::new(),
            verbose,
        }
    }
    
    fn execute(&mut self, input: &str) -> Result<Option<Value>> {
        // Skip empty input
        if input.trim().is_empty() {
            return Ok(None);
        }
        
        // Add to history
        self.history.push(input.to_string());
        
        // Tokenize
        let mut lexer = Lexer::new(input);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                eprintln!("{}: {}", "Syntax Error".red().bold(), e);
                return Ok(None);
            }
        };
        
        if self.verbose {
            println!("{}: {:?}", "Tokens".blue(), tokens);
        }
        
        // Parse
        let mut parser = VeyraParser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("{}: {}", "Parse Error".red().bold(), e);
                return Ok(None);
            }
        };
        
        if self.verbose {
            println!("{}: {:#?}", "AST".blue(), ast);
        }
        
        // Execute
        match self.interpreter.interpret(&ast) {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                eprintln!("{}: {}", "Runtime Error".red().bold(), e);
                Ok(None)
            }
        }
    }
    
    fn load_startup_file(&mut self, path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        println!("{} {}", "Loading startup file:".green(), path.display());
        
        match self.execute(&content) {
            Ok(_) => println!("{}", "Startup file loaded successfully".green()),
            Err(e) => eprintln!("{}: {}", "Error loading startup file".red(), e),
        }
        
        Ok(())
    }
}

fn print_welcome() {
    println!("{}", "=== Veyra Interactive REPL ===".cyan().bold());
    println!("{}", "Veyra Programming Language v1.0".cyan());
    println!("{}", "Type 'help' for help, 'exit' or Ctrl+C to quit".yellow());
    println!();
}

fn print_help() {
    println!("{}", "=== Veyra REPL Commands ===".cyan().bold());
    println!("  {}    - Show this help message", "help".green());
    println!("  {}    - Exit the REPL", "exit".green());
    println!("  {}   - Clear the screen", "clear".green());
    println!("  {} - Show command history", "history".green());
    println!("  {}     - Show current variables", "vars".green());
    println!("  {}  - Show REPL information", "info".green());
    println!();
    println!("{}", "Examples:".yellow().bold());
    println!("  let x = 42");
    println!("  print(\"Hello, World!\")");
    println!("  fn fibonacci(n) {{ if n <= 1 {{ return n }} return fibonacci(n-1) + fibonacci(n-2) }}");
    println!("  fibonacci(10)");
    println!();
}

fn print_info(state: &ReplState) {
    println!("{}", "=== REPL Information ===".cyan().bold());
    println!("History entries: {}", state.history.len());
    println!("Verbose mode: {}", if state.verbose { "ON" } else { "OFF" });
    // TODO: Add more interpreter state information
    println!();
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.version {
        println!("Veyra REPL v0.1.0");
        return Ok(());
    }
    
    let mut state = ReplState::new(cli.verbose);
    
    // Load startup file if provided
    if let Some(startup_path) = &cli.startup {
        state.load_startup_file(startup_path)?;
    }
    
    print_welcome();
    
    let mut rl = DefaultEditor::new()?;
    
    // Set up history file
    let history_file = dirs::home_dir()
        .map(|mut path| {
            path.push(".veyra_history");
            path
        });
    
    if let Some(ref history_path) = history_file {
        let _ = rl.load_history(history_path);
    }
    
    loop {
        let prompt = format!("{} ", "veyra>".green().bold());
        
        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();
                
                // Handle special REPL commands
                match input {
                    "help" => {
                        print_help();
                        continue;
                    }
                    "exit" | "quit" => {
                        println!("{}", "Goodbye!".yellow());
                        break;
                    }
                    "clear" => {
                        clear_screen();
                        continue;
                    }
                    "history" => {
                        println!("{}", "=== Command History ===".cyan().bold());
                        for (i, cmd) in state.history.iter().enumerate() {
                            println!("{:3}: {}", i + 1, cmd);
                        }
                        println!();
                        continue;
                    }
                    "vars" => {
                        println!("{}", "=== Current Variables ===".cyan().bold());
                        // TODO: Implement variable listing from interpreter state
                        println!("(Variable listing not yet implemented)");
                        println!();
                        continue;
                    }
                    "info" => {
                        print_info(&state);
                        continue;
                    }
                    _ => {}
                }
                
                // Add to readline history
                rl.add_history_entry(&line)?;
                
                // Execute the input
                match state.execute(input) {
                    Ok(Some(value)) => {
                        println!("{} {}", "=>".blue().bold(), format_value(&value));
                    }
                    Ok(None) => {
                        // No output (e.g., variable assignment, control flow)
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".yellow());
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "^D".yellow());
                break;
            }
            Err(err) => {
                eprintln!("{}: {}", "Error".red().bold(), err);
                break;
            }
        }
    }
    
    // Save history
    if let Some(history_path) = history_file {
        let _ = rl.save_history(&history_path);
    }
    
    Ok(())
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Char(c) => format!("'{}'", c),
        Value::Boolean(b) => b.to_string(),
        Value::None => "None".to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Dictionary(map) => {
            let mut pairs: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{}\": {}", k, format_value(v)))
                .collect();
            pairs.sort();
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Set(set) => {
            let mut elements: Vec<String> = set.iter()
                .map(|s| format!("\"{}\"", s))
                .collect();
            elements.sort();
            format!("{{{}}}", elements.join(", "))
        }
        Value::Tuple(tuple) => {
            let items: Vec<String> = tuple.iter().map(format_value).collect();
            format!("({})", items.join(", "))
        }
        Value::Reference(r) => {
            // Access the inner value through the Rc<RefCell<Value>>
            match r.value.try_borrow() {
                Ok(val) => format!("&{}", format_value(&*val)),
                Err(_) => "&<borrowed>".to_string(),
            }
        }
    }
}