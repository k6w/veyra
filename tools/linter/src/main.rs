use anyhow::{anyhow, Result};
use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Import from the main compiler
use veyra_compiler::{
    lexer::Lexer,
    parser::Parser as VeyraParser,
    ast::*,
};

#[derive(Parser)]
#[command(name = "veyra-lint")]
#[command(about = "Linter and static analysis for the Veyra programming language")]
#[command(version = "0.1.0")]
struct Cli {
    /// Files or directories to lint
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
    
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
    
    /// Recursively lint directories
    #[arg(short, long)]
    recursive: bool,
    
    /// Show warnings
    #[arg(short, long)]
    warnings: bool,
    
    /// Treat warnings as errors
    #[arg(long)]
    warnings_as_errors: bool,
    
    /// Enable specific lint rules (comma-separated)
    #[arg(long)]
    enable: Option<String>,
    
    /// Disable specific lint rules (comma-separated)
    #[arg(long)]
    disable: Option<String>,
    
    /// Configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LintLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
struct LintRule {
    _name: &'static str,
    level: LintLevel,
    enabled: bool,
    _description: &'static str,
}

#[derive(Debug, Clone)]
struct LintIssue {
    rule: &'static str,
    level: LintLevel,
    message: String,
    file: PathBuf,
    line: usize,
    column: usize,
    suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LintConfig {
    #[serde(default)]
    rules: HashMap<String, String>, // rule_name -> "error" | "warning" | "info" | "off"
    
    #[serde(default)]
    warnings_as_errors: bool,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            warnings_as_errors: false,
        }
    }
}

struct Linter {
    config: LintConfig,
    rules: HashMap<&'static str, LintRule>,
    issues: Vec<LintIssue>,
    current_file: PathBuf,
}

impl Linter {
    fn new(config: LintConfig) -> Self {
        let mut rules = HashMap::new();
        
        // Define built-in lint rules
        rules.insert("unused-variable", LintRule {
            _name: "unused-variable",
            level: LintLevel::Warning,
            enabled: true,
            _description: "Variable is declared but never used",
        });
        
        rules.insert("unused-function", LintRule {
            _name: "unused-function",
            level: LintLevel::Warning,
            enabled: true,
            _description: "Function is defined but never called",
        });
        
        rules.insert("undefined-variable", LintRule {
            _name: "undefined-variable",
            level: LintLevel::Error,
            enabled: true,
            _description: "Variable is used but never defined",
        });
        
        rules.insert("unreachable-code", LintRule {
            _name: "unreachable-code",
            level: LintLevel::Warning,
            enabled: true,
            _description: "Code after return statement is unreachable",
        });
        
        rules.insert("missing-return", LintRule {
            _name: "missing-return",
            level: LintLevel::Warning,
            enabled: true,
            _description: "Function may not return a value on all paths",
        });
        
        rules.insert("infinite-loop", LintRule {
            _name: "infinite-loop",
            level: LintLevel::Info,
            enabled: true,
            _description: "Loop condition is always true",
        });
        
        rules.insert("shadow-variable", LintRule {
            _name: "shadow-variable",
            level: LintLevel::Warning,
            enabled: true,
            _description: "Variable shadows another variable in outer scope",
        });
        
        rules.insert("empty-block", LintRule {
            _name: "empty-block",
            level: LintLevel::Info,
            enabled: true,
            _description: "Block contains no statements",
        });
        
        rules.insert("complex-expression", LintRule {
            _name: "complex-expression",
            level: LintLevel::Info,
            enabled: false,
            _description: "Expression is too complex and may be hard to read",
        });
        
        // Apply config overrides
        for (rule_name, level_str) in &config.rules {
            if let Some(rule) = rules.get_mut(rule_name.as_str()) {
                match level_str.as_str() {
                    "error" => {
                        rule.level = LintLevel::Error;
                        rule.enabled = true;
                    }
                    "warning" => {
                        rule.level = LintLevel::Warning;
                        rule.enabled = true;
                    }
                    "info" => {
                        rule.level = LintLevel::Info;
                        rule.enabled = true;
                    }
                    "off" => {
                        rule.enabled = false;
                    }
                    _ => {} // Invalid level, keep default
                }
            }
        }
        
        Self {
            config,
            rules,
            issues: Vec::new(),
            current_file: PathBuf::new(),
        }
    }
    
    fn lint_file(&mut self, path: &Path) -> Result<()> {
        self.current_file = path.to_path_buf();
        let content = fs::read_to_string(path)?;
        
        // Tokenize
        let mut lexer = Lexer::new(&content);
        let tokens = lexer.tokenize()
            .map_err(|e| anyhow!("Syntax error in {}: {}", path.display(), e))?;
        
        // Parse
        let mut parser = VeyraParser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| anyhow!("Parse error in {}: {}", path.display(), e))?;
        
        // Run lint checks
        self.check_unused_variables(&ast);
        self.check_unused_functions(&ast);
        self.check_undefined_variables(&ast);
        self.check_unreachable_code(&ast);
        self.check_missing_returns(&ast);
        self.check_empty_blocks(&ast);
        self.check_variable_shadowing(&ast);
        
        Ok(())
    }
    
    fn add_issue(&mut self, rule: &'static str, message: String, line: usize, column: usize, suggestion: Option<String>) {
        if let Some(rule_config) = self.rules.get(rule) {
            if rule_config.enabled {
                let level = if self.config.warnings_as_errors && rule_config.level == LintLevel::Warning {
                    LintLevel::Error
                } else {
                    rule_config.level.clone()
                };
                
                self.issues.push(LintIssue {
                    rule,
                    level,
                    message,
                    file: self.current_file.clone(),
                    line,
                    column,
                    suggestion,
                });
            }
        }
    }
    
    fn check_unused_variables(&mut self, program: &Program) {
        let mut analyzer = VariableAnalyzer::new();
        analyzer.analyze_program(program);
        
        for var in analyzer.unused_variables() {
            self.add_issue(
                "unused-variable",
                format!("Variable '{}' is declared but never used", var),
                1, // TODO: Track actual line numbers in AST
                1,
                Some(format!("Consider removing the variable or prefixing with '_'")),
            );
        }
    }
    
    fn check_unused_functions(&mut self, program: &Program) {
        let mut analyzer = FunctionAnalyzer::new();
        analyzer.analyze_program(program);
        
        for func in analyzer.unused_functions() {
            self.add_issue(
                "unused-function",
                format!("Function '{}' is defined but never called", func),
                1,
                1,
                Some(format!("Consider removing the function or making it public")),
            );
        }
    }
    
    fn check_undefined_variables(&mut self, program: &Program) {
        let mut analyzer = VariableAnalyzer::new();
        analyzer.analyze_program(program);
        
        for var in analyzer.undefined_variables() {
            self.add_issue(
                "undefined-variable",
                format!("Variable '{}' is used but never defined", var),
                1,
                1,
                None,
            );
        }
    }
    
    fn check_unreachable_code(&mut self, program: &Program) {
        let mut analyzer = ReachabilityAnalyzer::new();
        analyzer.analyze_program(program);
        
        if analyzer.has_unreachable_code() {
            self.add_issue(
                "unreachable-code",
                "Code after return statement is unreachable".to_string(),
                1,
                1,
                Some("Remove unreachable code".to_string()),
            );
        }
    }
    
    fn check_missing_returns(&mut self, program: &Program) {
        let mut analyzer = ReturnAnalyzer::new();
        analyzer.analyze_program(program);
        
        for func in analyzer.functions_missing_returns() {
            self.add_issue(
                "missing-return",
                format!("Function '{}' may not return a value on all paths", func),
                1,
                1,
                Some("Add explicit return statements".to_string()),
            );
        }
    }
    
    fn check_empty_blocks(&mut self, program: &Program) {
        let mut analyzer = BlockAnalyzer::new();
        analyzer.analyze_program(program);
        
        if analyzer.has_empty_blocks() {
            self.add_issue(
                "empty-block",
                "Block contains no statements".to_string(),
                1,
                1,
                Some("Add statements or remove the block".to_string()),
            );
        }
    }
    
    fn check_variable_shadowing(&mut self, program: &Program) {
        let mut analyzer = ShadowAnalyzer::new();
        analyzer.analyze_program(program);
        
        for var in analyzer.shadowed_variables() {
            self.add_issue(
                "shadow-variable",
                format!("Variable '{}' shadows another variable in outer scope", var),
                1,
                1,
                Some("Use a different variable name".to_string()),
            );
        }
    }
    
    fn get_issues(&self) -> &[LintIssue] {
        &self.issues
    }
    
    fn clear_issues(&mut self) {
        self.issues.clear();
    }
}

// Simplified analyzers (these would need full implementation)
struct VariableAnalyzer {
    declared: HashSet<String>,
    used: HashSet<String>,
}

impl VariableAnalyzer {
    fn new() -> Self {
        Self {
            declared: HashSet::new(),
            used: HashSet::new(),
        }
    }
    
    fn analyze_program(&mut self, _program: &Program) {
        // TODO: Implement proper variable tracking
        // For now, return empty sets
    }
    
    fn unused_variables(&self) -> Vec<String> {
        self.declared.difference(&self.used).cloned().collect()
    }
    
    fn undefined_variables(&self) -> Vec<String> {
        self.used.difference(&self.declared).cloned().collect()
    }
}

struct FunctionAnalyzer {
    defined: HashSet<String>,
    called: HashSet<String>,
}

impl FunctionAnalyzer {
    fn new() -> Self {
        Self {
            defined: HashSet::new(),
            called: HashSet::new(),
        }
    }
    
    fn analyze_program(&mut self, _program: &Program) {
        // TODO: Implement proper function tracking
    }
    
    fn unused_functions(&self) -> Vec<String> {
        self.defined.difference(&self.called).cloned().collect()
    }
}

struct ReachabilityAnalyzer {
    has_unreachable: bool,
}

impl ReachabilityAnalyzer {
    fn new() -> Self {
        Self { has_unreachable: false }
    }
    
    fn analyze_program(&mut self, _program: &Program) {
        // TODO: Implement proper reachability analysis
    }
    
    fn has_unreachable_code(&self) -> bool {
        self.has_unreachable
    }
}

struct ReturnAnalyzer {
    missing_returns: Vec<String>,
}

impl ReturnAnalyzer {
    fn new() -> Self {
        Self { missing_returns: Vec::new() }
    }
    
    fn analyze_program(&mut self, _program: &Program) {
        // TODO: Implement proper return analysis
    }
    
    fn functions_missing_returns(&self) -> &[String] {
        &self.missing_returns
    }
}

struct BlockAnalyzer {
    has_empty: bool,
}

impl BlockAnalyzer {
    fn new() -> Self {
        Self { has_empty: false }
    }
    
    fn analyze_program(&mut self, _program: &Program) {
        // TODO: Implement proper block analysis
    }
    
    fn has_empty_blocks(&self) -> bool {
        self.has_empty
    }
}

struct ShadowAnalyzer {
    shadowed: Vec<String>,
}

impl ShadowAnalyzer {
    fn new() -> Self {
        Self { shadowed: Vec::new() }
    }
    
    fn analyze_program(&mut self, _program: &Program) {
        // TODO: Implement proper shadow analysis
    }
    
    fn shadowed_variables(&self) -> &[String] {
        &self.shadowed
    }
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

fn load_config(config_path: Option<&Path>) -> Result<LintConfig> {
    if let Some(path) = config_path {
        let content = fs::read_to_string(path)?;
        let config: LintConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        // Look for default config files
        for default_path in &[".veyra-lint.json", "veyra-lint.json"] {
            if Path::new(default_path).exists() {
                let content = fs::read_to_string(default_path)?;
                let config: LintConfig = serde_json::from_str(&content)?;
                return Ok(config);
            }
        }
        Ok(LintConfig::default())
    }
}

fn print_issues_text(issues: &[LintIssue], show_warnings: bool) {
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    
    for issue in issues {
        match issue.level {
            LintLevel::Error => {
                error_count += 1;
                println!(
                    "{}: {}:{}:{} {} [{}]",
                    "error".red().bold(),
                    issue.file.display(),
                    issue.line,
                    issue.column,
                    issue.message,
                    issue.rule
                );
            }
            LintLevel::Warning if show_warnings => {
                warning_count += 1;
                println!(
                    "{}: {}:{}:{} {} [{}]",
                    "warning".yellow().bold(),
                    issue.file.display(),
                    issue.line,
                    issue.column,
                    issue.message,
                    issue.rule
                );
            }
            LintLevel::Info if show_warnings => {
                info_count += 1;
                println!(
                    "{}: {}:{}:{} {} [{}]",
                    "info".blue().bold(),
                    issue.file.display(),
                    issue.line,
                    issue.column,
                    issue.message,
                    issue.rule
                );
            }
            _ => {}
        }
        
        if let Some(suggestion) = &issue.suggestion {
            println!("  {}: {}", "help".green().bold(), suggestion);
        }
    }
    
    println!();
    if error_count > 0 {
        println!("{} {}", "Found".bold(), format!("{} error(s)", error_count).red().bold());
    }
    if warning_count > 0 && show_warnings {
        println!("{} {}", "Found".bold(), format!("{} warning(s)", warning_count).yellow().bold());
    }
    if info_count > 0 && show_warnings {
        println!("{} {}", "Found".bold(), format!("{} info(s)", info_count).blue().bold());
    }
}

fn print_issues_json(issues: &[LintIssue]) -> Result<()> {
    #[derive(Serialize)]
    struct JsonIssue<'a> {
        rule: &'a str,
        level: &'a str,
        message: &'a String,
        file: &'a Path,
        line: usize,
        column: usize,
        suggestion: &'a Option<String>,
    }
    
    let json_issues: Vec<JsonIssue> = issues
        .iter()
        .map(|issue| JsonIssue {
            rule: issue.rule,
            level: match issue.level {
                LintLevel::Error => "error",
                LintLevel::Warning => "warning",
                LintLevel::Info => "info",
            },
            message: &issue.message,
            file: &issue.file,
            line: issue.line,
            column: issue.column,
            suggestion: &issue.suggestion,
        })
        .collect();
    
    println!("{}", serde_json::to_string_pretty(&json_issues)?);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let config = load_config(cli.config.as_deref())?;
    let mut linter = Linter::new(config);
    
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
    
    let mut has_errors = false;
    
    for file in files {
        if cli.verbose {
            println!("Linting: {}", file.display());
        }
        
        linter.clear_issues();
        
        if let Err(e) = linter.lint_file(&file) {
            eprintln!("Error linting {}: {}", file.display(), e);
            continue;
        }
        
        let issues = linter.get_issues();
        
        // Check for errors
        if issues.iter().any(|issue| issue.level == LintLevel::Error) {
            has_errors = true;
        }
        
        match cli.format.as_str() {
            "json" => print_issues_json(issues)?,
            _ => print_issues_text(issues, cli.warnings),
        }
    }
    
    if has_errors {
        std::process::exit(1);
    }
    
    Ok(())
}