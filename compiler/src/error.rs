use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum VeyraError {
    #[error("Lexer error at line {line}, column {column}: {message}")]
    LexError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Parser error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Type Error: {message}")]
    TypeError { message: String },

    #[error("Runtime Error: {message}")]
    RuntimeError { message: String },

    #[error("IO Error: {0}")]
    IoError(String),

    #[error("Compiler Error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, VeyraError>;

impl VeyraError {
    pub fn lex_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        VeyraError::LexError {
            line,
            column,
            message: message.into(),
        }
    }

    pub fn parse_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        VeyraError::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn type_error(message: impl Into<String>) -> Self {
        VeyraError::TypeError {
            message: message.into(),
        }
    }

    pub fn runtime_error(message: impl Into<String>) -> Self {
        VeyraError::RuntimeError {
            message: message.into(),
        }
    }
}
