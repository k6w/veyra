use crate::config::ReplConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;
use veyra_compiler::{
    interpreter::{Interpreter, Value},
    lexer::Lexer,
    parser::Parser as VeyraParser,
};

/// REPL execution state
pub struct ReplState {
    interpreter: Interpreter,
    history: Vec<String>,
    config: ReplConfig,
    variables: HashMap<String, String>,
    functions: Vec<String>,
    pub(crate) multiline_buffer: String,
    last_execution_time: Option<f64>,
}

impl ReplState {
    pub fn new(config: ReplConfig) -> Self {
        Self {
            interpreter: Interpreter::new(),
            history: Vec::new(),
            config,
            variables: HashMap::new(),
            functions: Vec::new(),
            multiline_buffer: String::new(),
            last_execution_time: None,
        }
    }

    /// Execute Veyra code
    pub fn execute(&mut self, input: &str) -> Result<Option<Value>> {
        if input.trim().is_empty() {
            return Ok(None);
        }

        let start = Instant::now();

        // Tokenize
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;

        // Parse
        let mut parser = VeyraParser::new(tokens);
        let ast = parser.parse()?;

        // Execute
        let result = self.interpreter.interpret(&ast)?;

        let duration = start.elapsed();
        self.last_execution_time = Some(duration.as_secs_f64() * 1000.0);

        // Add to history
        self.history.push(input.to_string());

        Ok(Some(result))
    }

    /// Get execution timing
    pub fn last_timing(&self) -> Option<f64> {
        self.last_execution_time
    }

    /// Add input to multiline buffer
    pub fn add_to_multiline(&mut self, line: &str) {
        if !self.multiline_buffer.is_empty() {
            self.multiline_buffer.push('\n');
        }
        self.multiline_buffer.push_str(line);
    }

    /// Get and clear multiline buffer
    pub fn take_multiline(&mut self) -> String {
        std::mem::take(&mut self.multiline_buffer)
    }

    /// Check if in multiline mode
    pub fn is_multiline(&self) -> bool {
        !self.multiline_buffer.is_empty()
    }

    /// Get history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Get config
    pub fn config(&self) -> &ReplConfig {
        &self.config
    }

    /// Get config mutably
    pub fn config_mut(&mut self) -> &mut ReplConfig {
        &mut self.config
    }

    /// Clear state
    pub fn reset(&mut self) {
        self.interpreter = Interpreter::new();
        self.variables.clear();
        self.functions.clear();
        self.multiline_buffer.clear();
    }

    /// Get variable names
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Get function names
    pub fn functions(&self) -> &[String] {
        &self.functions
    }

    /// Load and execute a file
    pub fn load_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.execute(&content)?;
        Ok(())
    }

    /// Save history to file
    pub fn save_history(&self, path: &std::path::Path) -> Result<()> {
        let content = self.history.join("\n");
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Format a value for display
pub fn format_value(value: &Value) -> String {
    format_value_with_depth(value, 0, 3)
}

fn format_value_with_depth(value: &Value, depth: usize, max_depth: usize) -> String {
    if depth >= max_depth {
        return "...".to_string();
    }

    match value {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e10 {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s.escape_default()),
        Value::Char(c) => format!("'{}'", c.escape_default()),
        Value::Boolean(b) => b.to_string(),
        Value::None => "None".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else if depth >= max_depth - 1 {
                format!("[... {} items]", arr.len())
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .take(10)
                    .map(|v| format_value_with_depth(v, depth + 1, max_depth))
                    .collect();

                if arr.len() > 10 {
                    format!("[{}, ... {} more]", items.join(", "), arr.len() - 10)
                } else {
                    format!("[{}]", items.join(", "))
                }
            }
        }
        Value::Dictionary(map) => {
            if map.is_empty() {
                "{}".to_string()
            } else if depth >= max_depth - 1 {
                format!("{{... {} items}}", map.len())
            } else {
                let mut pairs: Vec<String> = map
                    .iter()
                    .take(10)
                    .map(|(k, v)| {
                        format!(
                            "\"{}\": {}",
                            k.escape_default(),
                            format_value_with_depth(v, depth + 1, max_depth)
                        )
                    })
                    .collect();
                pairs.sort();

                if map.len() > 10 {
                    format!("{{{}, ... {} more}}", pairs.join(", "), map.len() - 10)
                } else {
                    format!("{{{}}}", pairs.join(", "))
                }
            }
        }
        Value::Set(set) => {
            if set.is_empty() {
                "{}".to_string()
            } else {
                let mut elements: Vec<String> = set
                    .iter()
                    .take(10)
                    .map(|s| format!("\"{}\"", s.escape_default()))
                    .collect();
                elements.sort();

                if set.len() > 10 {
                    format!("{{{}, ... {} more}}", elements.join(", "), set.len() - 10)
                } else {
                    format!("{{{}}}", elements.join(", "))
                }
            }
        }
        Value::Tuple(tuple) => {
            if tuple.is_empty() {
                "()".to_string()
            } else if depth >= max_depth - 1 {
                format!("(... {} items)", tuple.len())
            } else {
                let items: Vec<String> = tuple
                    .iter()
                    .map(|v| format_value_with_depth(v, depth + 1, max_depth))
                    .collect();

                if tuple.len() == 1 {
                    format!("({},)", items[0])
                } else {
                    format!("({})", items.join(", "))
                }
            }
        }
        Value::Reference(r) => match r.value.try_borrow() {
            Ok(val) => format!(
                "&{}{}",
                if r.mutable { "mut " } else { "" },
                format_value_with_depth(&val, depth + 1, max_depth)
            ),
            Err(_) => "&<borrowed>".to_string(),
        },
    }
}

/// Get type name of a value
pub fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "int",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Char(_) => "char",
        Value::Boolean(_) => "bool",
        Value::None => "none",
        Value::Array(_) => "array",
        Value::Dictionary(_) => "dictionary",
        Value::Set(_) => "set",
        Value::Tuple(_) => "tuple",
        Value::Reference(r) => {
            if r.mutable {
                "&mut"
            } else {
                "&"
            }
        }
    }
}
