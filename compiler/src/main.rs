use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod lexer;
mod parser;
mod ast;
mod interpreter;
mod error;

use error::VeyraError;

#[derive(Parser)]
#[command(name = "veyc")]
#[command(about = "The Veyra programming language compiler")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Input file to compile
    input: Option<PathBuf>,
    
    /// Output file (defaults to input name with .exe extension)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Veyra source file
    Compile {
        /// Input file to compile
        input: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run a Veyra source file directly (interpret)
    Run {
        /// Input file to run
        input: PathBuf,
    },
    /// Check syntax without compiling
    Check {
        /// Input file to check
        input: PathBuf,
    },
    /// Show lexer tokens for debugging
    Lex {
        /// Input file to tokenize
        input: PathBuf,
    },
    /// Show parser AST for debugging
    Parse {
        /// Input file to parse
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Some(Commands::Compile { input, output }) => {
            compile_file(&input, output.as_ref())
        }
        Some(Commands::Run { input }) => {
            run_file(&input)
        }
        Some(Commands::Check { input }) => {
            check_file(&input)
        }
        Some(Commands::Lex { input }) => {
            lex_file(&input)
        }
        Some(Commands::Parse { input }) => {
            parse_file(&input)
        }
        None => {
            if let Some(input) = cli.input {
                if cli.output.is_some() {
                    compile_file(&input, cli.output.as_ref())
                } else {
                    run_file(&input)
                }
            } else {
                eprintln!("No input file specified. Use --help for usage information.");
                std::process::exit(1);
            }
        }
    };
    
    if let Err(e) = result {
        // Print error using Display format, not Debug
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn compile_file(input: &PathBuf, output: Option<&PathBuf>) -> Result<(), VeyraError> {
    println!("Compiling: {}", input.display());
    
    // Read source file
    let source = std::fs::read_to_string(input)
        .map_err(|e| VeyraError::IoError(format!("Failed to read file '{}': {}", input.display(), e)))?;
    
    // Tokenize
    let tokens = lexer::tokenize(&source)?;
    
    // Parse
    let ast = parser::parse(tokens)?;
    
    // For now, just print that we would compile
    let output_name = output
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            input.with_extension("exe").to_string_lossy().to_string()
        });
    
    println!("Would compile to: {}", output_name);
    println!("AST: {:#?}", ast);
    
    Ok(())
}

fn run_file(input: &PathBuf) -> Result<(), VeyraError> {
    println!("Running: {}", input.display());
    
    // Read source file
    let source = std::fs::read_to_string(input)
        .map_err(|e| VeyraError::IoError(format!("Failed to read file '{}': {}", input.display(), e)))?;
    
    // Tokenize
    let tokens = lexer::tokenize(&source)?;
    
    // Parse
    let ast = parser::parse(tokens)?;
    
    // Interpret
    interpreter::interpret(&ast)?;
    
    Ok(())
}

fn check_file(input: &PathBuf) -> Result<(), VeyraError> {
    println!("Checking: {}", input.display());
    
    // Read source file
    let source = std::fs::read_to_string(input)
        .map_err(|e| VeyraError::IoError(format!("Failed to read file '{}': {}", input.display(), e)))?;
    
    // Tokenize
    let tokens = lexer::tokenize(&source)?;
    
    // Parse
    let _ast = parser::parse(tokens)?;
    
    println!("✓ Syntax is valid");
    Ok(())
}

fn lex_file(input: &PathBuf) -> Result<(), VeyraError> {
    println!("Tokenizing: {}", input.display());
    
    // Read source file
    let source = std::fs::read_to_string(input)
        .map_err(|e| VeyraError::IoError(format!("Failed to read file '{}': {}", input.display(), e)))?;
    
    // Tokenize
    let tokens = lexer::tokenize(&source)?;
    
    // Print tokens
    for (i, token) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, token);
    }
    
    Ok(())
}

fn parse_file(input: &PathBuf) -> Result<(), VeyraError> {
    println!("Parsing: {}", input.display());
    
    // Read source file
    let source = std::fs::read_to_string(input)
        .map_err(|e| VeyraError::IoError(format!("Failed to read file '{}': {}", input.display(), e)))?;
    
    // Tokenize
    let tokens = lexer::tokenize(&source)?;
    
    // Parse
    let ast = parser::parse(tokens)?;
    
    // Print AST
    println!("{:#?}", ast);
    
    Ok(())
}
