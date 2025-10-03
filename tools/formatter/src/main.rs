use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Import from the main compiler
use veyra_compiler::{
    ast::*,
    lexer::Lexer,
    parser::Parser as VeyraParser,
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

#[derive(Clone)]
struct FormatterConfig {
    indent_size: usize,
    #[allow(dead_code)]
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

        for (i, item) in program.items.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.format_item(item);
        }

        // Ensure file ends with newline
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.output.clone()
    }

    fn format_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.format_function(func),
            Item::Struct(s) => self.format_struct(s),
            Item::Impl(i) => self.format_impl(i),
            Item::Import(import) => self.format_import(import),
            Item::Actor(actor) => self.format_actor(actor),
            Item::Statement(stmt) => self.format_statement(stmt),
        }
    }

    fn format_function(&mut self, func: &Function) {
        self.write_indent();
        if func.is_async {
            self.output.push_str("async ");
        }
        self.output.push_str("fn ");
        self.output.push_str(&func.name);
        self.output.push('(');
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&param.name);
            if let Some(t) = &param.param_type {
                self.output.push_str(": ");
                self.format_type(t);
            }
        }
        self.output.push(')');
        if let Some(ret_type) = &func.return_type {
            self.output.push_str(" -> ");
            self.format_type(ret_type);
        }
        self.output.push_str(" {");
        self.format_block_content(&func.body);
    }

    fn format_struct(&mut self, s: &Struct) {
        self.write_indent();
        self.output.push_str("struct ");
        self.output.push_str(&s.name);
        self.output.push_str(" {");
        if s.fields.is_empty() {
            self.output.push('\n');
            self.write_indent();
            self.output.push('}');
        } else {
            self.output.push('\n');
            self.current_indent += 1;
            for field in &s.fields {
                self.write_indent();
                self.output.push_str(&field.name);
                self.output.push_str(": ");
                self.format_type(&field.field_type);
                self.output.push_str(",\n");
            }
            self.current_indent -= 1;
            self.write_indent();
            self.output.push('}');
        }
    }

    fn format_impl(&mut self, i: &Impl) {
        self.write_indent();
        self.output.push_str("impl ");
        self.output.push_str(&i.target);
        self.output.push_str(" {");
        if i.methods.is_empty() {
            self.output.push('\n');
            self.write_indent();
            self.output.push('}');
        } else {
            self.output.push('\n');
            self.current_indent += 1;
            for (idx, method) in i.methods.iter().enumerate() {
                if idx > 0 {
                    self.output.push('\n');
                }
                self.format_function(method);
                self.output.push('\n');
            }
            self.current_indent -= 1;
            self.write_indent();
            self.output.push('}');
        }
    }

    fn format_import(&mut self, import: &Import) {
        self.write_indent();
        self.output.push_str("import ");
        self.output.push_str(&import.path.join("::"));
        if let Some(alias) = &import.alias {
            self.output.push_str(" as ");
            self.output.push_str(alias);
        }
    }

    fn format_actor(&mut self, actor: &Actor) {
        self.write_indent();
        self.output.push_str("actor ");
        self.output.push_str(&actor.name);
        self.output.push_str(" {");
        self.output.push('\n');
        self.current_indent += 1;
        for field in &actor.fields {
            self.write_indent();
            self.output.push_str(&field.name);
            self.output.push_str(": ");
            self.format_type(&field.field_type);
            self.output.push_str(",\n");
        }
        for (idx, method) in actor.methods.iter().enumerate() {
            if idx > 0 || !actor.fields.is_empty() {
                self.output.push('\n');
            }
            self.format_function(method);
            self.output.push('\n');
        }
        self.current_indent -= 1;
        self.write_indent();
        self.output.push('}');
    }

    fn format_type(&mut self, t: &Type) {
        match t {
            Type::Primitive(p) => {
                let name = match p {
                    PrimitiveType::Int => "int",
                    PrimitiveType::I32 => "i32",
                    PrimitiveType::I64 => "i64",
                    PrimitiveType::U32 => "u32",
                    PrimitiveType::U64 => "u64",
                    PrimitiveType::F32 => "f32",
                    PrimitiveType::F64 => "f64",
                    PrimitiveType::Bool => "bool",
                    PrimitiveType::Char => "char",
                    PrimitiveType::String => "string",
                };
                self.output.push_str(name);
            }
            Type::Array { element_type, size } => {
                self.output.push('[');
                self.format_type(element_type);
                if let Some(s) = size {
                    self.output.push_str("; ");
                    self.output.push_str(&s.to_string());
                }
                self.output.push(']');
            }
            Type::Optional(inner) => {
                self.format_type(inner);
                self.output.push('?');
            }
            Type::Reference { target, mutable } => {
                self.output.push('&');
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.format_type(target);
            }
            Type::Function {
                parameters,
                return_type,
            } => {
                self.output.push_str("fn(");
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type(param);
                }
                self.output.push_str(") -> ");
                self.format_type(return_type);
            }
            Type::Custom(name) => self.output.push_str(name),
        }
    }

    fn format_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                self.write_indent();
                self.output.push_str("let ");
                if var_decl.mutable {
                    self.output.push_str("mut ");
                }
                self.output.push_str(&var_decl.name);
                if let Some(t) = &var_decl.var_type {
                    self.output.push_str(": ");
                    self.format_type(t);
                }
                self.output.push_str(" = ");
                self.format_expression(&var_decl.initializer);
            }
            Statement::Assignment(assign) => {
                self.write_indent();
                self.format_expression(&assign.target);
                self.output.push(' ');
                match assign.operator {
                    AssignmentOperator::Assign => self.output.push('='),
                    AssignmentOperator::AddAssign => self.output.push_str("+="),
                    AssignmentOperator::SubAssign => self.output.push_str("-="),
                    AssignmentOperator::MulAssign => self.output.push_str("*="),
                    AssignmentOperator::DivAssign => self.output.push_str("/="),
                    AssignmentOperator::ModAssign => self.output.push_str("%="),
                    AssignmentOperator::BitwiseAndAssign => self.output.push_str("&="),
                    AssignmentOperator::BitwiseOrAssign => self.output.push_str("|="),
                    AssignmentOperator::BitwiseXorAssign => self.output.push_str("^="),
                    AssignmentOperator::LeftShiftAssign => self.output.push_str("<<="),
                    AssignmentOperator::RightShiftAssign => self.output.push_str(">>="),
                }
                self.output.push(' ');
                self.format_expression(&assign.value);
            }
            Statement::Expression(expr_stmt) => {
                self.write_indent();
                self.format_expression(&expr_stmt.expression);
            }
            Statement::If(if_stmt) => {
                self.write_indent();
                self.output.push_str("if ");
                self.format_expression(&if_stmt.condition);
                self.output.push_str(" {");
                self.format_block_content(&if_stmt.then_branch);

                for (elif_cond, elif_body) in &if_stmt.elif_branches {
                    self.output.push_str(" elif ");
                    self.format_expression(elif_cond);
                    self.output.push_str(" {");
                    self.format_block_content(elif_body);
                }

                if let Some(else_body) = &if_stmt.else_branch {
                    self.output.push_str(" else {");
                    self.format_block_content(else_body);
                }
            }
            Statement::While(while_stmt) => {
                self.write_indent();
                self.output.push_str("while ");
                self.format_expression(&while_stmt.condition);
                self.output.push_str(" {");
                self.format_block_content(&while_stmt.body);
            }
            Statement::For(for_stmt) => {
                self.write_indent();
                self.output.push_str("for ");
                self.output.push_str(&for_stmt.variable);
                self.output.push_str(" in ");
                self.format_expression(&for_stmt.iterable);
                self.output.push_str(" {");
                self.format_block_content(&for_stmt.body);
            }
            Statement::Match(match_stmt) => {
                self.write_indent();
                self.output.push_str("match ");
                self.format_expression(&match_stmt.expression);
                self.output.push_str(" {");
                self.output.push('\n');
                self.current_indent += 1;
                for arm in &match_stmt.arms {
                    self.write_indent();
                    self.format_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    // Format the arm body inline if it's simple
                    self.format_statement(&arm.body);
                    self.output.push(',');
                    self.output.push('\n');
                }
                self.current_indent -= 1;
                self.write_indent();
                self.output.push('}');
            }
            Statement::Return(ret_stmt) => {
                self.write_indent();
                self.output.push_str("return");
                if let Some(e) = &ret_stmt.value {
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
            Statement::Block(block) => {
                self.write_indent();
                self.output.push('{');
                self.format_block_content(block);
            }
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(name) => self.output.push_str(name),
            Pattern::Literal(lit) => self.format_literal(lit),
            Pattern::Wildcard => self.output.push('_'),
        }
    }

    fn format_block_content(&mut self, block: &Block) {
        if block.statements.is_empty() {
            self.output.push('\n');
            self.write_indent();
            self.output.push('}');
            return;
        }

        self.output.push('\n');
        self.current_indent += 1;

        for (i, stmt) in block.statements.iter().enumerate() {
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

    fn format_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Integer(n) => self.output.push_str(&n.to_string()),
            Literal::Float(f) => self.output.push_str(&f.to_string()),
            Literal::String(s) => {
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            Literal::Char(c) => {
                self.output.push('\'');
                self.output.push(*c);
                self.output.push('\'');
            }
            Literal::Boolean(b) => self.output.push_str(if *b { "true" } else { "false" }),
            Literal::None => self.output.push_str("None"),
        }
    }

    fn format_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(lit) => self.format_literal(lit),
            Expression::Identifier(name) => {
                self.output.push_str(name);
            }
            Expression::Binary(bin_expr) => {
                self.format_expression(&bin_expr.left);
                self.output.push(' ');
                let op_str = match bin_expr.operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Power => "**",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::And => "and",
                    BinaryOperator::Or => "or",
                    BinaryOperator::BitwiseAnd => "&",
                    BinaryOperator::BitwiseOr => "|",
                    BinaryOperator::BitwiseXor => "^",
                    BinaryOperator::LeftShift => "<<",
                    BinaryOperator::RightShift => ">>",
                };
                self.output.push_str(op_str);
                self.output.push(' ');
                self.format_expression(&bin_expr.right);
            }
            Expression::Unary(unary_expr) => {
                let op_str = match unary_expr.operator {
                    UnaryOperator::Minus => "-",
                    UnaryOperator::Not => "not ",
                    UnaryOperator::BitwiseNot => "~",
                    UnaryOperator::Reference => "&",
                    UnaryOperator::MutableReference => "&mut ",
                    UnaryOperator::Dereference => "*",
                };
                self.output.push_str(op_str);
                self.format_expression(&unary_expr.operand);
            }
            Expression::Call(call_expr) => {
                self.format_expression(&call_expr.callee);
                self.output.push('(');
                for (i, arg) in call_expr.arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(arg);
                }
                self.output.push(')');
            }
            Expression::Index(index_expr) => {
                self.format_expression(&index_expr.object);
                self.output.push('[');
                self.format_expression(&index_expr.index);
                self.output.push(']');
            }
            Expression::FieldAccess(field_expr) => {
                self.format_expression(&field_expr.object);
                self.output.push('.');
                self.output.push_str(&field_expr.field);
            }
            Expression::MethodCall(method_expr) => {
                self.format_expression(&method_expr.object);
                self.output.push('.');
                self.output.push_str(&method_expr.method);
                self.output.push('(');
                for (i, arg) in method_expr.arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(arg);
                }
                self.output.push(')');
            }
            Expression::ModuleAccess(mod_expr) => {
                self.output.push_str(&mod_expr.module);
                self.output.push_str("::");
                self.output.push_str(&mod_expr.item);
            }
            Expression::Array(array_expr) => {
                self.output.push('[');
                for (i, elem) in array_expr.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(elem);
                }
                self.output.push(']');
            }
            Expression::Dictionary(dict_expr) => {
                self.output.push('{');
                for (i, (key, value)) in dict_expr.pairs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(key);
                    self.output.push_str(": ");
                    self.format_expression(value);
                }
                self.output.push('}');
            }
            Expression::Set(set_expr) => {
                self.output.push('{');
                for (i, elem) in set_expr.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(elem);
                }
                self.output.push('}');
            }
            Expression::Tuple(tuple_expr) => {
                self.output.push('(');
                for (i, elem) in tuple_expr.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expression(elem);
                }
                if tuple_expr.elements.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Expression::StructInit(struct_expr) => {
                self.output.push_str(&struct_expr.struct_name);
                self.output.push_str(" { ");
                for (i, (field_name, field_value)) in struct_expr.fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(field_name);
                    self.output.push_str(": ");
                    self.format_expression(field_value);
                }
                self.output.push_str(" }");
            }
            Expression::If(if_expr) => {
                self.output.push_str("if ");
                self.format_expression(&if_expr.condition);
                self.output.push_str(" then ");
                self.format_expression(&if_expr.then_expr);
                self.output.push_str(" else ");
                self.format_expression(&if_expr.else_expr);
            }
            Expression::Match(match_expr) => {
                self.output.push_str("match ");
                self.format_expression(&match_expr.expression);
                self.output.push_str(" { ");
                for (i, arm) in match_expr.arms.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    self.format_expression(&arm.expression);
                }
                self.output.push_str(" }");
            }
            Expression::Range(range_expr) => {
                self.format_expression(&range_expr.start);
                if range_expr.inclusive {
                    self.output.push_str("..=");
                } else {
                    self.output.push_str("..");
                }
                self.format_expression(&range_expr.end);
            }
            Expression::Await(await_expr) => {
                self.output.push_str("await ");
                self.format_expression(&await_expr.expression);
            }
            Expression::Spawn(spawn_expr) => {
                self.output.push_str("spawn ");
                self.format_expression(&spawn_expr.expression);
            }
            Expression::Cast(cast_expr) => {
                self.format_expression(&cast_expr.expression);
                self.output.push_str(" as ");
                self.format_type(&cast_expr.target_type);
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
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow!("Syntax error in {}: {}", path.display(), e))?;

    // Parse
    let mut parser = VeyraParser::new(tokens);
    let ast = parser
        .parse()
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
