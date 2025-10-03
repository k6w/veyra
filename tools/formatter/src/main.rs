use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Import from the main compiler
use veyra_compiler::{
    lexer::{Lexer, Token, TokenType},
    parser::Parser as VeyraParser,
    ast::*,
};

#[derive(Parser)]
#[command(name = "veyra-fmt")]
#[command(about = "Code formatter for the Veyra programming language")]
#[command(version = "0.1.0")]
struct Cli {
    /// Files or directories to format
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
    
    /// Format files in place
    #[arg(short, long)]
    write: bool,
    
    /// Check if files are already formatted (exit code 1 if not)
    #[arg(short, long)]
    check: bool,
    
    /// Show diff of formatting changes
    #[arg(short, long)]
    diff: bool,
    
    /// Recursively format directories
    #[arg(short, long)]
    recursive: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Indentation size (default: 4 spaces)
    #[arg(long, default_value = "4")]
    indent: usize,
    
    /// Maximum line length (default: 100)
    #[arg(long, default_value = "100")]
    max_line_length: usize,
}

struct FormatterConfig {
    indent_size: usize,
    max_line_length: usize,
    use_spaces: bool, // vs tabs
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 100,
            use_spaces: true,
        }
    }
}

struct Formatter {
    config: FormatterConfig,
    current_indent: usize,
    output: String,
}

impl Formatter {
    fn new(config: FormatterConfig) -> Self {
        Self {
            config,
            current_indent: 0,
            output: String::new(),
        }
    }
    
    fn format_program(&mut self, program: &Program) -> String {
        self.output.clear();
        self.current_indent = 0;
        
        for (i, stmt) in program.statements.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.format_statement(stmt);
        }
        
        // Ensure file ends with newline
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        
        self.output.clone()
    }
    
    fn format_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value } => {
                self.write_indent();
                self.output.push_str("let ");
                self.output.push_str(name);
                self.output.push_str(" = ");
                self.format_expression(value);
            }
            Statement::Assignment { name, value } => {
                self.write_indent();
                self.output.push_str(name);
                self.output.push_str(" = ");
                self.format_expression(value);
            }
            Statement::CompoundAssignment { name, operator, value } => {
                self.write_indent();
                self.output.push_str(name);
                self.output.push(' ');
                self.output.push_str(operator);
                self.output.push_str(" = ");
                self.format_expression(value);
            }
            Statement::Expression(expr) => {
                self.write_indent();
                self.format_expression(expr);
            }
            Statement::If { condition, then_branch, elif_branches, else_branch } => {
                self.write_indent();
                self.output.push_str("if ");
                self.format_expression(condition);
                self.output.push_str(" {");
                self.format_block(then_branch);
                
                for (elif_cond, elif_body) in elif_branches {
                    self.output.push_str(" elif ");
                    self.format_expression(elif_cond);
                    self.output.push_str(" {");
                    self.format_block(elif_body);
                }
                
                if let Some(else_body) = else_branch {
                    self.output.push_str(" else {");
                    self.format_block(else_body);
                }
            }
            Statement::While { condition, body } => {
                self.write_indent();
                self.output.push_str("while ");
                self.format_expression(condition);
                self.output.push_str(" {");
                self.format_block(body);
            }
            Statement::For { variable, iterable, body } => {
                self.write_indent();
                self.output.push_str("for ");
                self.output.push_str(variable);
                self.output.push_str(" in ");
                self.format_expression(iterable);
                self.output.push_str(" {");
                self.format_block(body);
            }
            Statement::Function { name, params, body } => {
                self.write_indent();
                self.output.push_str("fn ");
                self.output.push_str(name);
                self.output.push('(');
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(param);
                }
                self.output.push_str(") {");
                self.format_block(body);
            }
            Statement::Return(expr) => {
                self.write_indent();
                self.output.push_str("return");
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.format_expression(e);
                }
            }
            Statement::Break => {
                self.write_indent();
                self.output.push_str("break");
            }
            Statement::Continue => {
                self.write_indent();
                self.output.push_str("continue");
            }
            Statement::Import { module } => {
                self.write_indent();
                self.output.push_str("import ");
                self.output.push_str(module);
            }
        }
    }
    
    fn format_block(&mut self, statements: &[Statement]) {
        if statements.is_empty() {
            self.output.push_str("\n");
            self.write_indent();
            self.output.push('}');
            return;
        }
        
        self.output.push('\n');
        self.current_indent += 1;
        
        for (i, stmt) in statements.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.format_statement(stmt);
        }
        
        self.output.push('\n');
        self.current_indent -= 1;
        self.write_indent();
        self.output.push('}');
    }
    
    fn format_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(lit) => {
                match lit {
                    Literal::Int(n) => self.output.push_str(&n.to_string()),
                    Literal::Float(f) => self.output.push_str(&f.to_string()),
                    Literal::String(s) => {
                        self.output.push('"');
                        self.output.push_str(s);
                        self.output.push('"');
                    }
                    Literal::Bool(b) => self.output.push_str(&b.to_string()),
                    Literal::None => self.output.push_str("None"),
                }
            }
            Expression::Identifier(name) => {
                self.output.push_str(name);
            }
            Expression::Binary { left, operator, right } => {
                self.format_expression(left);
                self.output.push(' ');
                self.output.push_str(operator);
                self.output.push(' ');
                self.format_expression(right);
            }
            Expression::Unary { operator, operand } => {
                self.output.push_str(operator);
                self.format_expression(operand);
            }
            Expression::Call { name, args } => {
                self.output.push_str(name);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(arg);
                }
                self.output.push(')');
            }
            Expression::Array(elements) => {
                self.output.push('[');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(elem);
                }
                self.output.push(']');
            }
            Expression::Index { array, index } => {
                self.format_expression(array);
                self.output.push('[');
                self.format_expression(index);
                self.output.push(']');
            }
        }
    }
    
    fn write_indent(&mut self) {
        if self.config.use_spaces {
            for _ in 0..(self.current_indent * self.config.indent_size) {
                self.output.push(' ');
            }
        } else {
            for _ in 0..self.current_indent {
                self.output.push('\t');
            }
        }
    }
}

fn format_file(path: &Path, config: &FormatterConfig) -> Result<String> {
    let content = fs::read_to_string(path)?;
    
    // Tokenize
    let mut lexer = Lexer::new(&content);
    let tokens = lexer.tokenize()
        .map_err(|e| anyhow!("Syntax error in {}: {}", path.display(), e))?;
    
    // Parse
    let mut parser = VeyraParser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| anyhow!("Parse error in {}: {}", path.display(), e))?;
    
    // Format
    let mut formatter = Formatter::new(config.clone());
    Ok(formatter.format_program(&ast))
}

fn collect_veyra_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for path in paths {
        if path.is_file() {
            if path.extension().and_then(|s| s.to_str()) == Some("vey") {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            if recursive {
                for entry in WalkDir::new(path) {
                    let entry = entry?;
                    if entry.file_type().is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "vey" {
                                files.push(entry.path().to_path_buf());
                            }
                        }
                    }
                }
            } else {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    if entry.file_type()?.is_file() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("vey") {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }
    
    Ok(files)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let config = FormatterConfig {
        indent_size: cli.indent,
        max_line_length: cli.max_line_length,
        use_spaces: true,
    };
    
    // If no paths specified, use current directory
    let paths = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths
    };
    
    let files = collect_veyra_files(&paths, cli.recursive)?;
    
    if files.is_empty() {
        println!("No .vey files found");
        return Ok(());
    }
    
    let mut needs_formatting = false;
    
    for file in files {
        if cli.verbose {
            println!("Processing: {}", file.display());
        }
        
        let original_content = fs::read_to_string(&file)?;
        let formatted_content = match format_file(&file, &config) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error formatting {}: {}", file.display(), e);
                continue;
            }
        };
        
        if original_content != formatted_content {
            needs_formatting = true;
            
            if cli.check {
                println!("File needs formatting: {}", file.display());
            } else if cli.diff {
                println!("--- {}", file.display());
                println!("+++ {} (formatted)", file.display());
                // Simple line-by-line diff
                let orig_lines: Vec<&str> = original_content.lines().collect();
                let fmt_lines: Vec<&str> = formatted_content.lines().collect();
                
                for (i, (orig, fmt)) in orig_lines.iter().zip(fmt_lines.iter()).enumerate() {
                    if orig != fmt {
                        println!("@@ -{} +{} @@", i + 1, i + 1);
                        println!("-{}", orig);
                        println!("+{}", fmt);
                    }
                }
            } else if cli.write {
                fs::write(&file, &formatted_content)?;
                println!("Formatted: {}", file.display());
            } else {
                print!("{}", formatted_content);
            }
        } else if cli.verbose {
            println!("Already formatted: {}", file.display());
        }
    }
    
    if cli.check && needs_formatting {
        std::process::exit(1);
    }
    
    Ok(())
}