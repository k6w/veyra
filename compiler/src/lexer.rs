use crate::error::{Result, VeyraError};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),

    // Identifiers and Keywords
    Identifier,

    // Keywords
    And,
    As,
    Async,
    Await,
    Actor,
    Break,
    Continue,
    Elif,
    Else,
    False,
    Fn,
    For,
    If,
    Impl,
    Import,
    In,
    Let,
    Loop,
    Match,
    Mut,
    None,
    Not,
    Or,
    Pub,
    Return,
    Some,
    Spawn,
    Struct,
    True,
    Unsafe,
    While,

    // Operators
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %
    StarStar, // **

    Equal,        // =
    PlusEqual,    // +=
    MinusEqual,   // -=
    StarEqual,    // *=
    SlashEqual,   // /=
    PercentEqual, // %=

    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    Question,    // ?
    QuestionDot, // ?.
    DotDot,      // ..
    DotDotEqual, // ..=
    LeftArrow,   // <-

    // Bitwise operators
    Ampersand,          // &
    Pipe,               // |
    Caret,              // ^
    Tilde,              // ~
    AmpersandAmpersand, // &&
    PipePipe,           // ||
    LeftShift,          // <<
    RightShift,         // >>
    AmpersandEqual,     // &=
    PipeEqual,          // |=
    CaretEqual,         // ^=
    LeftShiftEqual,     // <<=
    RightShiftEqual,    // >>=

    // Punctuation
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Semicolon,    // ;
    Colon,        // :
    DoubleColon,  // ::
    Dot,          // .
    Arrow,        // ->
    Dollar,       // $

    // Special
    Newline,
    Indent,
    Dedent,
    Eof,

    // Comments (usually filtered out)
    Comment,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    #[allow(dead_code)]
    indent_stack: Vec<usize>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0], // Start with 0 indentation
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            if token.kind != TokenKind::Comment {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    #[allow(dead_code)]
    fn handle_line_start(&mut self, tokens: &mut Vec<Token>) -> Result<()> {
        if self.is_at_end() {
            return Ok(());
        }

        let indent_level = self.count_indentation();
        let current_indent = *self.indent_stack.last().unwrap();

        if indent_level > current_indent {
            self.indent_stack.push(indent_level);
            tokens.push(Token {
                kind: TokenKind::Indent,
                lexeme: String::new(),
                line: self.line,
                column: 1,
            });
        } else if indent_level < current_indent {
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                tokens.push(Token {
                    kind: TokenKind::Dedent,
                    lexeme: String::new(),
                    line: self.line,
                    column: 1,
                });
            }

            if self.indent_stack.last() != Some(&indent_level) {
                return Err(VeyraError::lex_error(
                    self.line,
                    1,
                    "Indentation does not match any outer indentation level",
                ));
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn count_indentation(&mut self) -> usize {
        let mut count = 0;
        while self.position < self.input.len() {
            match self.input[self.position] {
                ' ' => {
                    count += 1;
                    self.advance();
                }
                '\t' => {
                    // Return error for tab indentation
                    return 0; // For now, just return 0 - in a real implementation we'd handle this better
                }
                '#' => {
                    // Skip comment lines for indentation
                    self.skip_line();
                    if !self.is_at_end() {
                        self.line += 1;
                        self.column = 1;
                        return self.count_indentation();
                    }
                    return 0;
                }
                '\n' | '\r' => {
                    // Empty line, skip
                    self.skip_line();
                    if !self.is_at_end() {
                        self.line += 1;
                        self.column = 1;
                        return self.count_indentation();
                    }
                    return 0;
                }
                _ => break,
            }
        }
        count
    }

    fn next_token(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_column = self.column;

        let c = self.advance();

        let kind = match c {
            // Single-character tokens
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => {
                if self.match_char(':') {
                    TokenKind::DoubleColon
                } else {
                    TokenKind::Colon
                }
            }
            '%' => {
                if self.match_char('=') {
                    TokenKind::PercentEqual
                } else {
                    TokenKind::Percent
                }
            }

            // Operators that might be compound
            '+' => {
                if self.match_char('=') {
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.match_char('=') {
                    TokenKind::MinusEqual
                } else if self.match_char('>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.match_char('=') {
                    TokenKind::StarEqual
                } else if self.match_char('*') {
                    TokenKind::StarStar
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.match_char('=') {
                    TokenKind::SlashEqual
                } else {
                    TokenKind::Slash
                }
            }
            '=' => {
                if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    return self.error("Unexpected character '!'");
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenKind::LessEqual
                } else if self.match_char('-') {
                    TokenKind::LeftArrow
                } else if self.match_char('<') {
                    if self.match_char('=') {
                        TokenKind::LeftShiftEqual
                    } else {
                        TokenKind::LeftShift
                    }
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else if self.match_char('>') {
                    if self.match_char('=') {
                        TokenKind::RightShiftEqual
                    } else {
                        TokenKind::RightShift
                    }
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                if self.match_char('&') {
                    TokenKind::AmpersandAmpersand
                } else if self.match_char('=') {
                    TokenKind::AmpersandEqual
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.match_char('|') {
                    TokenKind::PipePipe
                } else if self.match_char('=') {
                    TokenKind::PipeEqual
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => {
                if self.match_char('=') {
                    TokenKind::CaretEqual
                } else {
                    TokenKind::Caret
                }
            }
            '~' => TokenKind::Tilde,
            '?' => {
                if self.match_char('.') {
                    TokenKind::QuestionDot
                } else {
                    TokenKind::Question
                }
            }
            '.' => {
                if self.match_char('.') {
                    if self.match_char('=') {
                        TokenKind::DotDotEqual
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }

            // Comments
            '#' => {
                if self.match_char('[') && self.match_char('[') {
                    // Block comment
                    self.skip_block_comment()?;
                } else {
                    // Line comment
                    self.skip_line_comment();
                }
                TokenKind::Comment
            }

            // Newlines
            '\n' => {
                self.line += 1;
                self.column = 1;
                TokenKind::Newline
            }
            '\r' => {
                if self.match_char('\n') {
                    self.line += 1;
                    self.column = 1;
                }
                TokenKind::Newline
            }

            // String literals
            '"' => {
                return self.string_literal();
            }

            // Character literals
            '\'' => {
                return self.char_literal();
            }

            // Numbers
            c if c.is_ascii_digit() => {
                return self.number_literal(c);
            }

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => {
                return self.identifier_or_keyword(c);
            }

            _ => {
                return self.error(&format!("Unexpected character '{}'", c));
            }
        };

        let lexeme = match &kind {
            TokenKind::LeftParen => "(".to_string(),
            TokenKind::RightParen => ")".to_string(),
            TokenKind::LeftBracket => "[".to_string(),
            TokenKind::RightBracket => "]".to_string(),
            TokenKind::LeftBrace => "{".to_string(),
            TokenKind::RightBrace => "}".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Semicolon => ";".to_string(),
            TokenKind::Colon => ":".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Percent => "%".to_string(),
            TokenKind::StarStar => "**".to_string(),
            TokenKind::Equal => "=".to_string(),
            TokenKind::PlusEqual => "+=".to_string(),
            TokenKind::MinusEqual => "-=".to_string(),
            TokenKind::StarEqual => "*=".to_string(),
            TokenKind::SlashEqual => "/=".to_string(),
            TokenKind::PercentEqual => "%=".to_string(),
            TokenKind::EqualEqual => "==".to_string(),
            TokenKind::BangEqual => "!=".to_string(),
            TokenKind::Less => "<".to_string(),
            TokenKind::LessEqual => "<=".to_string(),
            TokenKind::Greater => ">".to_string(),
            TokenKind::GreaterEqual => ">=".to_string(),
            TokenKind::Question => "?".to_string(),
            TokenKind::QuestionDot => "?.".to_string(),
            TokenKind::DotDot => "..".to_string(),
            TokenKind::DotDotEqual => "..=".to_string(),
            TokenKind::LeftArrow => "<-".to_string(),
            TokenKind::Arrow => "->".to_string(),
            TokenKind::Dollar => "$".to_string(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::Newline => "\\n".to_string(),
            TokenKind::Comment => "#".to_string(),
            _ => String::new(),
        };

        Ok(Token {
            kind,
            lexeme,
            line: start_line,
            column: start_column,
        })
    }

    fn string_literal(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_column = self.column - 1; // Include opening quote
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let c = self.advance();
            if c == '\\' {
                // Handle escape sequences
                if self.is_at_end() {
                    return self.error("Unterminated string literal");
                }
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    '0' => value.push('\0'),
                    'x' => {
                        // Hex escape: \xHH
                        if self.position + 1 >= self.input.len() {
                            return self.error("Incomplete hex escape sequence");
                        }
                        let hex_chars: String = self.input[self.position..self.position + 2]
                            .iter()
                            .collect();
                        if let Ok(byte) = u8::from_str_radix(&hex_chars, 16) {
                            value.push(byte as char);
                            self.position += 2;
                            self.column += 2;
                        } else {
                            return self.error("Invalid hex escape sequence");
                        }
                    }
                    'u' => {
                        // Unicode escape: \u{HHHHHH}
                        if self.peek() != '{' {
                            return self.error("Expected '{' after \\u");
                        }
                        self.advance(); // consume '{'

                        let mut hex_digits = String::new();
                        while !self.is_at_end() && self.peek() != '}' {
                            let digit = self.advance();
                            if digit.is_ascii_hexdigit() {
                                hex_digits.push(digit);
                            } else {
                                return self.error("Invalid character in unicode escape");
                            }
                        }

                        if self.is_at_end() || self.peek() != '}' {
                            return self.error("Unterminated unicode escape");
                        }
                        self.advance(); // consume '}'

                        if hex_digits.is_empty() || hex_digits.len() > 6 {
                            return self.error("Invalid unicode escape length");
                        }

                        if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
                            if let Some(ch) = char::from_u32(code_point) {
                                value.push(ch);
                            } else {
                                return self.error("Invalid unicode code point");
                            }
                        } else {
                            return self.error("Invalid unicode escape sequence");
                        }
                    }
                    _ => {
                        return self.error(&format!("Unknown escape sequence '\\{}'", escaped));
                    }
                }
            } else {
                if c == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                value.push(c);
            }
        }

        if self.is_at_end() {
            return self.error("Unterminated string literal");
        }

        // Consume closing quote
        self.advance();

        Ok(Token {
            kind: TokenKind::String(value.clone()),
            lexeme: format!("\"{}\"", value),
            line: start_line,
            column: start_column + 1,
        })
    }

    fn char_literal(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_column = self.column - 1; // Include opening quote

        if self.is_at_end() {
            return self.error("Unterminated character literal");
        }

        let c = self.advance();
        let value = if c == '\\' {
            // Handle escape sequences
            if self.is_at_end() {
                return self.error("Unterminated character literal");
            }
            let escaped = self.advance();
            match escaped {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                '0' => '\0',
                'x' => {
                    // Hex escape: \xHH
                    if self.position + 1 >= self.input.len() {
                        return self.error("Incomplete hex escape sequence");
                    }
                    let hex_chars: String = self.input[self.position..self.position + 2]
                        .iter()
                        .collect();
                    if let Ok(byte) = u8::from_str_radix(&hex_chars, 16) {
                        self.position += 2;
                        self.column += 2;
                        byte as char
                    } else {
                        return self.error("Invalid hex escape sequence");
                    }
                }
                _ => {
                    return self.error(&format!("Unknown escape sequence '\\{}'", escaped));
                }
            }
        } else {
            c
        };

        if self.is_at_end() || self.peek() != '\'' {
            return self.error("Unterminated character literal");
        }

        // Consume closing quote
        self.advance();

        Ok(Token {
            kind: TokenKind::Char(value),
            lexeme: if value == '\'' {
                "'\\''".to_string()
            } else {
                format!("'{}'", value)
            },
            line: start_line,
            column: start_column + 1,
        })
    }

    fn number_literal(&mut self, first_digit: char) -> Result<Token> {
        let start_line = self.line;
        let start_column = self.column - 1;
        let mut value = String::new();
        value.push(first_digit);

        // Handle different number bases
        if first_digit == '0' && !self.is_at_end() {
            match self.peek() {
                'b' => return self.binary_literal(start_line, start_column),
                'o' => return self.octal_literal(start_line, start_column),
                'x' => return self.hex_literal(start_line, start_column),
                _ => {}
            }
        }

        // Decimal number
        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
            let c = self.advance();
            if c != '_' {
                value.push(c);
            }
        }

        // Check for decimal point
        if !self.is_at_end()
            && self.peek() == '.'
            && self.position + 1 < self.input.len()
            && self.input[self.position + 1].is_ascii_digit()
        {
            value.push(self.advance()); // consume '.'

            while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
                let c = self.advance();
                if c != '_' {
                    value.push(c);
                }
            }

            // Check for exponent
            if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
                value.push(self.advance());
                if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                    value.push(self.advance());
                }
                while !self.is_at_end() && self.peek().is_ascii_digit() {
                    value.push(self.advance());
                }
            }

            let float_val = value.parse::<f64>().map_err(|_| {
                VeyraError::lex_error(start_line, start_column + 1, "Invalid float literal")
            })?;

            return Ok(Token {
                kind: TokenKind::Float(float_val),
                lexeme: self.lexeme_from_range(start_column, self.column - 1),
                line: start_line,
                column: start_column + 1,
            });
        }

        let int_val = value.parse::<i64>().map_err(|_| {
            VeyraError::lex_error(start_line, start_column + 1, "Invalid integer literal")
        })?;

        Ok(Token {
            kind: TokenKind::Integer(int_val),
            lexeme: self.lexeme_from_range(start_column, self.column - 1),
            line: start_line,
            column: start_column + 1,
        })
    }

    fn binary_literal(&mut self, start_line: usize, start_column: usize) -> Result<Token> {
        self.advance(); // consume 'b'
        let mut value = String::new();

        while !self.is_at_end() && (self.peek() == '0' || self.peek() == '1' || self.peek() == '_')
        {
            let c = self.advance();
            if c != '_' {
                value.push(c);
            }
        }

        if value.is_empty() {
            return self.error("Invalid binary literal");
        }

        let int_val = i64::from_str_radix(&value, 2).map_err(|_| {
            VeyraError::lex_error(start_line, start_column + 1, "Invalid binary literal")
        })?;

        Ok(Token {
            kind: TokenKind::Integer(int_val),
            lexeme: self.lexeme_from_range(start_column, self.column - 1),
            line: start_line,
            column: start_column + 1,
        })
    }

    fn octal_literal(&mut self, start_line: usize, start_column: usize) -> Result<Token> {
        self.advance(); // consume 'o'
        let mut value = String::new();

        while !self.is_at_end()
            && (self.peek().is_ascii_digit() && self.peek() <= '7' || self.peek() == '_')
        {
            let c = self.advance();
            if c != '_' {
                value.push(c);
            }
        }

        if value.is_empty() {
            return self.error("Invalid octal literal");
        }

        let int_val = i64::from_str_radix(&value, 8).map_err(|_| {
            VeyraError::lex_error(start_line, start_column + 1, "Invalid octal literal")
        })?;

        Ok(Token {
            kind: TokenKind::Integer(int_val),
            lexeme: self.lexeme_from_range(start_column, self.column - 1),
            line: start_line,
            column: start_column + 1,
        })
    }

    fn hex_literal(&mut self, start_line: usize, start_column: usize) -> Result<Token> {
        self.advance(); // consume 'x'
        let mut value = String::new();

        while !self.is_at_end() && (self.peek().is_ascii_hexdigit() || self.peek() == '_') {
            let c = self.advance();
            if c != '_' {
                value.push(c);
            }
        }

        if value.is_empty() {
            return self.error("Invalid hexadecimal literal");
        }

        let int_val = i64::from_str_radix(&value, 16).map_err(|_| {
            VeyraError::lex_error(start_line, start_column + 1, "Invalid hexadecimal literal")
        })?;

        Ok(Token {
            kind: TokenKind::Integer(int_val),
            lexeme: self.lexeme_from_range(start_column, self.column - 1),
            line: start_line,
            column: start_column + 1,
        })
    }

    fn identifier_or_keyword(&mut self, first_char: char) -> Result<Token> {
        let start_line = self.line;
        let start_column = self.column - 1;
        let mut value = String::new();
        value.push(first_char);

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            value.push(self.advance());
        }

        let kind = match value.as_str() {
            "and" => TokenKind::And,
            "as" => TokenKind::As,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "actor" => TokenKind::Actor,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "fn" => TokenKind::Fn,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "impl" => TokenKind::Impl,
            "import" => TokenKind::Import,
            "in" => TokenKind::In,
            "let" => TokenKind::Let,
            "loop" => TokenKind::Loop,
            "match" => TokenKind::Match,
            "mut" => TokenKind::Mut,
            "None" => TokenKind::None,
            "not" => TokenKind::Not,
            "or" => TokenKind::Or,
            "pub" => TokenKind::Pub,
            "return" => TokenKind::Return,
            "Some" => TokenKind::Some,
            "spawn" => TokenKind::Spawn,
            "struct" => TokenKind::Struct,
            "true" => TokenKind::True,
            "unsafe" => TokenKind::Unsafe,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };

        Ok(Token {
            kind,
            lexeme: value,
            line: start_line,
            column: start_column + 1,
        })
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            if self.peek() == '#'
                && self.position + 1 < self.input.len()
                && self.input[self.position + 1] == '['
                && self.position + 2 < self.input.len()
                && self.input[self.position + 2] == '['
            {
                self.advance(); // #
                self.advance(); // [
                self.advance(); // [
                depth += 1;
            } else if self.peek() == ']'
                && self.position + 1 < self.input.len()
                && self.input[self.position + 1] == ']'
                && self.position + 2 < self.input.len()
                && self.input[self.position + 2] == '#'
            {
                self.advance(); // ]
                self.advance(); // ]
                self.advance(); // #
                depth -= 1;
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                self.advance();
            }
        }

        if depth > 0 {
            return self.error("Unterminated block comment");
        }

        Ok(())
    }

    fn _skip_cpp_block_comment(&mut self) -> Result<()> {
        while !self.is_at_end() {
            if self.peek() == '*'
                && self.position + 1 < self.input.len()
                && self.input[self.position + 1] == '/'
            {
                self.advance(); // *
                self.advance(); // /
                return Ok(());
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                self.advance();
            }
        }

        return self.error("Unterminated block comment");
    }

    #[allow(dead_code)]
    fn skip_line(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' && self.peek() != '\r' {
            self.advance();
        }
        if !self.is_at_end() {
            if self.peek() == '\r' {
                self.advance();
                if !self.is_at_end() && self.peek() == '\n' {
                    self.advance();
                }
            } else {
                self.advance(); // \n
            }
        }
    }

    #[allow(dead_code)]
    fn is_at_line_start(&self) -> bool {
        self.column == 1
            || (self.position > 0
                && (self.input[self.position - 1] == '\n' || self.input[self.position - 1] == '\r'))
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        let c = self.input[self.position];
        self.position += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.input[self.position] != expected {
            false
        } else {
            self.position += 1;
            self.column += 1;
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn lexeme_from_range(&self, start: usize, end: usize) -> String {
        if start >= self.input.len() || start >= end {
            return String::new();
        }
        let actual_end = end.min(self.input.len());
        self.input[start..actual_end].iter().collect()
    }

    fn error<T>(&self, message: &str) -> Result<T> {
        Err(VeyraError::lex_error(self.line, self.column, message))
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    lexer.tokenize()
}
