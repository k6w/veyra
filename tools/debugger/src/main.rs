use anyhow::{anyhow, Result};
use clap::Parser;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Import from the main compiler
use veyra_compiler::{
    lexer::Lexer,
    parser::Parser as VeyraParser,
    interpreter::Interpreter,
    ast::*,
};

#[derive(Parser)]
#[command(name = "veyra-dbg")]
#[command(about = "Debugger for the Veyra programming language")]
#[command(version = "0.1.0")]
struct Cli {
    /// Veyra file to debug
    file: PathBuf,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Start debugging immediately
    #[arg(short, long)]
    run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Breakpoint {
    id: usize,
    line: usize,
    condition: Option<String>,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum DebugCommand {
    Run,
    Continue,
    Step,
    StepOver,
    StepOut,
    Break(usize),           // Set breakpoint at line
    Delete(usize),          // Delete breakpoint by ID
    List,                   // List source code
    Print(String),          // Print variable
    Backtrace,             // Show call stack
    Variables,             // Show all variables
    Help,
    Quit,
}

struct DebuggerState {
    _source_code: String,
    source_lines: Vec<String>,
    _ast: Program,
    _interpreter: Interpreter,
    breakpoints: HashMap<usize, Breakpoint>,
    next_breakpoint_id: usize,
    current_line: usize,
    call_stack: Vec<String>,
    is_running: bool,
    is_paused: bool,
    step_mode: StepMode,
}

#[derive(Debug, Clone, PartialEq)]
enum StepMode {
    None,
    Step,        // Step into
    StepOver,    // Step over function calls
    StepOut,     // Step out of current function
}

impl DebuggerState {
    fn new(source_code: String, ast: Program) -> Self {
        let source_lines: Vec<String> = source_code.lines().map(|s| s.to_string()).collect();
        
        Self {
            _source_code: source_code,
            source_lines,
            _ast: ast,
            _interpreter: Interpreter::new(),
            breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
            current_line: 1,
            call_stack: Vec::new(),
            is_running: false,
            is_paused: false,
            step_mode: StepMode::None,
        }
    }
    
    fn add_breakpoint(&mut self, line: usize, condition: Option<String>) -> usize {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        let breakpoint = Breakpoint {
            id,
            line,
            condition,
            enabled: true,
        };
        
        self.breakpoints.insert(id, breakpoint);
        id
    }
    
    fn delete_breakpoint(&mut self, id: usize) -> bool {
        self.breakpoints.remove(&id).is_some()
    }
    
    fn _should_break_at_line(&self, line: usize) -> bool {
        for breakpoint in self.breakpoints.values() {
            if breakpoint.enabled && breakpoint.line == line {
                // TODO: Evaluate condition if present
                return true;
            }
        }
        false
    }
    
    fn list_source(&self, around_line: Option<usize>) -> Vec<String> {
        let center_line = around_line.unwrap_or(self.current_line);
        let start = center_line.saturating_sub(5);
        let end = (center_line + 5).min(self.source_lines.len());
        
        let mut result = Vec::new();
        
        for (i, line) in self.source_lines.iter().enumerate() {
            let line_num = i + 1;
            
            if line_num >= start && line_num <= end {
                let marker = if line_num == self.current_line {
                    " -> "
                } else if self.breakpoints.values().any(|bp| bp.line == line_num && bp.enabled) {
                    " *  "
                } else {
                    "    "
                };
                
                result.push(format!("{}{:3}: {}", marker, line_num, line));
            }
        }
        
        result
    }
    
    fn get_variable_value(&self, name: &str) -> Option<String> {
        // TODO: Get variable from interpreter state
        // This would require extending the interpreter to expose variable state
        Some(format!("(variable '{}' not implemented)", name))
    }
    
    fn get_all_variables(&self) -> HashMap<String, String> {
        // TODO: Get all variables from interpreter state
        HashMap::new()
    }
}

struct Debugger {
    state: DebuggerState,
    _verbose: bool,
}

impl Debugger {
    fn new(source_code: String, ast: Program, verbose: bool) -> Self {
        let state = DebuggerState::new(source_code, ast);
        
        Self {
            state,
            _verbose: verbose,
        }
    }
    
    fn parse_command(&self, input: &str) -> Result<DebugCommand> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(anyhow!("Empty command"));
        }
        
        match parts[0] {
            "r" | "run" => Ok(DebugCommand::Run),
            "c" | "continue" => Ok(DebugCommand::Continue),
            "s" | "step" => Ok(DebugCommand::Step),
            "n" | "next" => Ok(DebugCommand::StepOver),
            "f" | "finish" => Ok(DebugCommand::StepOut),
            "b" | "break" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: break <line_number>"));
                }
                let line: usize = parts[1].parse()
                    .map_err(|_| anyhow!("Invalid line number: {}", parts[1]))?;
                Ok(DebugCommand::Break(line))
            }
            "d" | "delete" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: delete <breakpoint_id>"));
                }
                let id: usize = parts[1].parse()
                    .map_err(|_| anyhow!("Invalid breakpoint ID: {}", parts[1]))?;
                Ok(DebugCommand::Delete(id))
            }
            "l" | "list" => Ok(DebugCommand::List),
            "p" | "print" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: print <variable_name>"));
                }
                Ok(DebugCommand::Print(parts[1].to_string()))
            }
            "bt" | "backtrace" => Ok(DebugCommand::Backtrace),
            "vars" | "variables" => Ok(DebugCommand::Variables),
            "h" | "help" => Ok(DebugCommand::Help),
            "q" | "quit" => Ok(DebugCommand::Quit),
            _ => Err(anyhow!("Unknown command: {}", parts[0])),
        }
    }
    
    fn execute_command(&mut self, command: DebugCommand) -> Result<bool> {
        match command {
            DebugCommand::Run => {
                println!("{} Starting program...", "->".green().bold());
                self.state.is_running = true;
                self.state.is_paused = false;
                self.run_until_breakpoint()?;
            }
            DebugCommand::Continue => {
                if !self.state.is_running {
                    println!("{} Program not running. Use 'run' to start.", "!".yellow().bold());
                    return Ok(false);
                }
                println!("{} Continuing...", "->".green().bold());
                self.state.is_paused = false;
                self.run_until_breakpoint()?;
            }
            DebugCommand::Step => {
                self.state.step_mode = StepMode::Step;
                self.execute_step()?;
            }
            DebugCommand::StepOver => {
                self.state.step_mode = StepMode::StepOver;
                self.execute_step()?;
            }
            DebugCommand::StepOut => {
                self.state.step_mode = StepMode::StepOut;
                self.execute_step()?;
            }
            DebugCommand::Break(line) => {
                if line > self.state.source_lines.len() {
                    println!("{} Line {} is beyond end of file", "!".yellow().bold(), line);
                    return Ok(false);
                }
                let id = self.state.add_breakpoint(line, None);
                println!("{} Breakpoint {} set at line {}", "✓".green().bold(), id, line);
            }
            DebugCommand::Delete(id) => {
                if self.state.delete_breakpoint(id) {
                    println!("{} Breakpoint {} deleted", "✓".green().bold(), id);
                } else {
                    println!("{} Breakpoint {} not found", "!".yellow().bold(), id);
                }
            }
            DebugCommand::List => {
                let lines = self.state.list_source(None);
                for line in lines {
                    println!("{}", line);
                }
            }
            DebugCommand::Print(var_name) => {
                if let Some(value) = self.state.get_variable_value(&var_name) {
                    println!("{} = {}", var_name.cyan().bold(), value);
                } else {
                    println!("{} Variable '{}' not found", "!".yellow().bold(), var_name);
                }
            }
            DebugCommand::Backtrace => {
                println!("{}", "Call Stack:".bold());
                if self.state.call_stack.is_empty() {
                    println!("  (empty)");
                } else {
                    for (i, frame) in self.state.call_stack.iter().enumerate() {
                        println!("  #{}: {}", i, frame);
                    }
                }
            }
            DebugCommand::Variables => {
                let vars = self.state.get_all_variables();
                if vars.is_empty() {
                    println!("No variables in current scope");
                } else {
                    println!("{}", "Variables:".bold());
                    for (name, value) in vars {
                        println!("  {} = {}", name.cyan().bold(), value);
                    }
                }
            }
            DebugCommand::Help => {
                self.print_help();
            }
            DebugCommand::Quit => {
                println!("{} Goodbye!", "✓".green().bold());
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn run_until_breakpoint(&mut self) -> Result<()> {
        // TODO: Implement actual execution with breakpoint checking
        // This is a simplified version that just simulates execution
        
        println!("  {} Program execution (simulated)", "->".blue());
        
        // Simulate hitting a breakpoint
        if !self.state.breakpoints.is_empty() {
            let first_bp_line = self.state.breakpoints.values().next().unwrap().line;
            self.state.current_line = first_bp_line;
            self.state.is_paused = true;
            
            println!("{} Breakpoint hit at line {}", "!".red().bold(), first_bp_line);
            
            // Show current line
            let lines = self.state.list_source(Some(first_bp_line));
            for line in lines {
                println!("{}", line);
            }
        } else {
            println!("{} Program finished", "✓".green().bold());
            self.state.is_running = false;
        }
        
        Ok(())
    }
    
    fn execute_step(&mut self) -> Result<()> {
        // TODO: Implement actual single-step execution
        println!("  {} Step execution (simulated)", "->".blue());
        
        self.state.current_line += 1;
        
        if self.state.current_line > self.state.source_lines.len() {
            println!("{} Program finished", "✓".green().bold());
            self.state.is_running = false;
            return Ok(());
        }
        
        // Show current line
        let lines = self.state.list_source(Some(self.state.current_line));
        for line in lines {
            println!("{}", line);
        }
        
        Ok(())
    }
    
    fn print_help(&self) {
        println!("{}", "=== Veyra Debugger Commands ===".cyan().bold());
        println!();
        println!("{}", "Execution Control:".yellow().bold());
        println!("  {} {} - Start program execution", "r, run".green(), "".dimmed());
        println!("  {} {} - Continue execution", "c, continue".green(), "".dimmed());
        println!("  {} {} - Step into (single instruction)", "s, step".green(), "".dimmed());
        println!("  {} {} - Step over (skip function calls)", "n, next".green(), "".dimmed());
        println!("  {} {} - Step out (finish current function)", "f, finish".green(), "".dimmed());
        println!();
        println!("{}", "Breakpoints:".yellow().bold());
        println!("  {} {} - Set breakpoint at line", "b, break <line>".green(), "".dimmed());
        println!("  {} {} - Delete breakpoint by ID", "d, delete <id>".green(), "".dimmed());
        println!();
        println!("{}", "Information:".yellow().bold());
        println!("  {} {} - List source code", "l, list".green(), "".dimmed());
        println!("  {} {} - Print variable value", "p, print <var>".green(), "".dimmed());
        println!("  {} {} - Show call stack", "bt, backtrace".green(), "".dimmed());
        println!("  {} {} - Show all variables", "vars, variables".green(), "".dimmed());
        println!();
        println!("{}", "Other:".yellow().bold());
        println!("  {} {} - Show this help", "h, help".green(), "".dimmed());
        println!("  {} {} - Quit debugger", "q, quit".green(), "".dimmed());
        println!();
    }
    
    fn print_status(&self) {
        if self.state.is_running {
            if self.state.is_paused {
                println!("{} Paused at line {}", "Status:".bold(), self.state.current_line);
            } else {
                println!("{} Running", "Status:".bold());
            }
        } else {
            println!("{} Not running", "Status:".bold());
        }
        
        if !self.state.breakpoints.is_empty() {
            println!("{}", "Breakpoints:".bold());
            for bp in self.state.breakpoints.values() {
                let status = if bp.enabled { "enabled" } else { "disabled" };
                println!("  #{}: line {} ({})", bp.id, bp.line, status);
            }
        }
    }
    
    fn run_interactive(&mut self) -> Result<()> {
        println!("{}", "=== Veyra Debugger ===".cyan().bold());
        println!("Type 'help' for available commands");
        println!();
        
        self.print_status();
        println!();
        
        let mut rl = DefaultEditor::new()?;
        
        loop {
            let prompt = if self.state.is_paused {
                format!("{} ", "(veyra-dbg) [paused]".red().bold())
            } else {
                format!("{} ", "(veyra-dbg)".green().bold())
            };
            
            match rl.readline(&prompt) {
                Ok(line) => {
                    let input = line.trim();
                    
                    if input.is_empty() {
                        continue;
                    }
                    
                    rl.add_history_entry(&line)?;
                    
                    match self.parse_command(input) {
                        Ok(command) => {
                            match self.execute_command(command) {
                                Ok(should_quit) => {
                                    if should_quit {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}: {}", "Error".red().bold(), e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{}: {}", "Command Error".red().bold(), e);
                            println!("Type 'help' for available commands");
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("{}: {}", "Error".red().bold(), err);
                    break;
                }
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read and parse the source file
    let source_code = fs::read_to_string(&cli.file)
        .map_err(|e| anyhow!("Failed to read file {}: {}", cli.file.display(), e))?;
    
    // Tokenize
    let mut lexer = Lexer::new(&source_code);
    let tokens = lexer.tokenize()
        .map_err(|e| anyhow!("Syntax error: {}", e))?;
    
    // Parse
    let mut parser = VeyraParser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| anyhow!("Parse error: {}", e))?;
    
    println!("{} Loaded {}", "✓".green().bold(), cli.file.display());
    println!("  {} {} lines", "Lines:".bold(), source_code.lines().count());
    
    // Create debugger
    let mut debugger = Debugger::new(source_code, ast, cli.verbose);
    
    if cli.run {
        // Start debugging immediately
        debugger.execute_command(DebugCommand::Run)?;
    }
    
    // Enter interactive mode
    debugger.run_interactive()?;
    
    Ok(())
}