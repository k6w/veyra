// Library interface for the Veyra compiler
// This exposes the internal modules for use by other tools like LSP

pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;

// Re-export commonly used types
pub use ast::*;
pub use error::VeyraError;
pub use interpreter::Interpreter;
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
