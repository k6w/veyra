use crate::ast::*;
use crate::error::{Result, VeyraError};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at top level
            if self.check(&TokenKind::Newline) {
                self.advance();
                continue;
            }

            if self.check(&TokenKind::Eof) {
                break;
            }

            items.push(self.parse_item()?);

            // Skip trailing newlines after items
            while self.check(&TokenKind::Newline) {
                self.advance();
            }
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item> {
        match self.peek().kind {
            TokenKind::Fn => Ok(Item::Function(self.parse_function(false)?)),
            TokenKind::Async => {
                self.advance(); // consume 'async'
                if self.check(&TokenKind::Fn) {
                    Ok(Item::Function(self.parse_function(true)?))
                } else {
                    Err(self.error("Expected 'fn' after 'async'"))
                }
            }
            TokenKind::Struct => Ok(Item::Struct(self.parse_struct()?)),
            TokenKind::Impl => Ok(Item::Impl(self.parse_impl()?)),
            TokenKind::Import => Ok(Item::Import(self.parse_import()?)),
            TokenKind::Actor => Ok(Item::Actor(self.parse_actor()?)),
            _ => {
                // Try to parse as a statement (for top-level expressions like print calls)
                let statement = self.parse_statement()?;
                Ok(Item::Statement(statement))
            }
        }
    }

    fn parse_function(&mut self, is_async: bool) -> Result<Function> {
        self.consume(&TokenKind::Fn, "Expected 'fn'")?;

        let name = self
            .consume_identifier("Expected function name")?
            .lexeme
            .clone();

        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                parameters.push(self.parse_parameter()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Function {
            name,
            parameters,
            return_type,
            body,
            is_async,
        })
    }

    fn parse_parameter(&mut self) -> Result<Parameter> {
        let name = self
            .consume_identifier("Expected parameter name")?
            .lexeme
            .clone();

        let param_type = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let default = if self.match_token(&TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Parameter {
            name,
            param_type,
            default,
        })
    }

    fn parse_struct(&mut self) -> Result<Struct> {
        self.consume(&TokenKind::Struct, "Expected 'struct'")?;

        let name = self
            .consume_identifier("Expected struct name")?
            .lexeme
            .clone();

        self.consume(&TokenKind::LeftBrace, "Expected '{' after struct name")?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            fields.push(self.parse_field()?);
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after struct fields")?;

        Ok(Struct { name, fields })
    }

    fn parse_field(&mut self) -> Result<Field> {
        let name = self
            .consume_identifier("Expected field name")?
            .lexeme
            .clone();
        self.consume(&TokenKind::Colon, "Expected ':' after field name")?;
        let field_type = self.parse_type()?;

        Ok(Field { name, field_type })
    }

    fn parse_impl(&mut self) -> Result<Impl> {
        self.consume(&TokenKind::Impl, "Expected 'impl'")?;

        let target = self
            .consume_identifier("Expected type name")?
            .lexeme
            .clone();

        self.consume(&TokenKind::LeftBrace, "Expected '{' after impl target")?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let is_async = self.match_token(&TokenKind::Async);
            methods.push(self.parse_function(is_async)?);
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after impl methods")?;

        Ok(Impl { target, methods })
    }

    fn parse_import(&mut self) -> Result<Import> {
        self.consume(&TokenKind::Import, "Expected 'import'")?;

        let mut path = vec![self
            .consume_identifier("Expected module name")?
            .lexeme
            .clone()];

        while self.match_token(&TokenKind::Dot) {
            if self.check(&TokenKind::LeftBrace) {
                // Selective import: import std.collections.{HashMap, Vec}
                self.advance(); // consume '{'
                let mut items = Vec::new();
                loop {
                    items.push(
                        self.consume_identifier("Expected import item")?
                            .lexeme
                            .clone(),
                    );
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.consume(&TokenKind::RightBrace, "Expected '}' after import items")?;
                return Ok(Import {
                    path,
                    alias: None,
                    items: Some(items),
                });
            } else {
                path.push(
                    self.consume_identifier("Expected module name")?
                        .lexeme
                        .clone(),
                );
            }
        }

        let alias = if self.match_token(&TokenKind::Identifier) && self.previous().lexeme == "as" {
            Some(
                self.consume_identifier("Expected alias name")?
                    .lexeme
                    .clone(),
            )
        } else {
            None
        };

        Ok(Import {
            path,
            alias,
            items: None,
        })
    }

    fn parse_actor(&mut self) -> Result<Actor> {
        self.consume(&TokenKind::Actor, "Expected 'actor'")?;

        let name = self
            .consume_identifier("Expected actor name")?
            .lexeme
            .clone();

        self.consume(&TokenKind::LeftBrace, "Expected '{' after actor name")?;
        self.skip_newlines();

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Fn) || self.check(&TokenKind::Async) {
                let is_async = self.match_token(&TokenKind::Async);
                methods.push(self.parse_function(is_async)?);
            } else {
                fields.push(self.parse_field()?);
            }
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after actor body")?;

        Ok(Actor {
            name,
            fields,
            methods,
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let base_type = match &self.peek().kind {
            TokenKind::Identifier => {
                let name = self.advance().lexeme.clone();
                match name.as_str() {
                    "int" => Type::Primitive(PrimitiveType::Int),
                    "i32" => Type::Primitive(PrimitiveType::I32),
                    "i64" => Type::Primitive(PrimitiveType::I64),
                    "u32" => Type::Primitive(PrimitiveType::U32),
                    "u64" => Type::Primitive(PrimitiveType::U64),
                    "f32" => Type::Primitive(PrimitiveType::F32),
                    "f64" => Type::Primitive(PrimitiveType::F64),
                    "bool" => Type::Primitive(PrimitiveType::Bool),
                    "char" => Type::Primitive(PrimitiveType::Char),
                    "string" => Type::Primitive(PrimitiveType::String),
                    _ => Type::Custom(name),
                }
            }
            TokenKind::LeftBracket => {
                self.advance(); // consume '['
                let element_type = Box::new(self.parse_type()?);

                let size = if self.match_token(&TokenKind::Semicolon) {
                    if let TokenKind::Integer(n) = &self.peek().kind {
                        let size = *n as usize;
                        self.advance();
                        Some(size)
                    } else {
                        return Err(self.error("Expected array size"));
                    }
                } else {
                    None
                };

                self.consume(&TokenKind::RightBracket, "Expected ']' after array type")?;
                Type::Array { element_type, size }
            }
            TokenKind::Fn => {
                self.advance(); // consume 'fn'
                self.consume(&TokenKind::LeftParen, "Expected '(' after 'fn'")?;

                let mut parameters = Vec::new();
                if !self.check(&TokenKind::RightParen) {
                    loop {
                        parameters.push(self.parse_type()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }

                self.consume(
                    &TokenKind::RightParen,
                    "Expected ')' after function parameters",
                )?;

                let return_type = if self.match_token(&TokenKind::Arrow) {
                    Box::new(self.parse_type()?)
                } else {
                    // Default to unit type (represented as empty tuple for now)
                    Box::new(Type::Custom("()".to_string()))
                };

                Type::Function {
                    parameters,
                    return_type,
                }
            }
            _ => return Err(self.error("Expected type")),
        };

        // Handle optional types (T?)
        if self.match_token(&TokenKind::Question) {
            Ok(Type::Optional(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    fn parse_block(&mut self) -> Result<Block> {
        if self.match_token(&TokenKind::LeftBrace) {
            // Brace-delimited block
            self.skip_newlines();
            let mut statements = Vec::new();

            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
                self.skip_newlines();
            }

            self.consume(&TokenKind::RightBrace, "Expected '}' after block")?;
            Ok(Block { statements })
        } else if self.match_token(&TokenKind::Indent) {
            // Indentation-delimited block
            let mut statements = Vec::new();

            while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
                self.skip_newlines();
            }

            if self.check(&TokenKind::Dedent) {
                self.advance();
            }

            Ok(Block { statements })
        } else {
            // Single statement block
            let statement = self.parse_statement()?;
            Ok(Block {
                statements: vec![statement],
            })
        }
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match &self.peek().kind {
            TokenKind::Let => self.parse_variable_declaration(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Match => self.parse_match_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => {
                self.advance();
                Ok(Statement::Break)
            }
            TokenKind::Continue => {
                self.advance();
                Ok(Statement::Continue)
            }
            TokenKind::LeftBrace | TokenKind::Indent => Ok(Statement::Block(self.parse_block()?)),
            _ => {
                // Try to parse as assignment or expression
                let expr = self.parse_expression()?;

                // Check if this is an assignment
                if let Some(op) = self.parse_assignment_operator() {
                    let value = self.parse_expression()?;
                    Ok(Statement::Assignment(Assignment {
                        target: expr,
                        operator: op,
                        value,
                    }))
                } else {
                    Ok(Statement::Expression(ExpressionStatement {
                        expression: expr,
                    }))
                }
            }
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::Let, "Expected 'let'")?;

        let mutable = self.match_token(&TokenKind::Mut);
        let name = self
            .consume_identifier("Expected variable name")?
            .lexeme
            .clone();

        let var_type = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&TokenKind::Equal, "Expected '=' after variable declaration")?;
        let initializer = self.parse_expression()?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            name,
            var_type,
            initializer,
            mutable,
        }))
    }

    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::If, "Expected 'if'")?;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let mut elif_branches = Vec::new();
        while self.match_token(&TokenKind::Elif) {
            let elif_condition = self.parse_expression()?;
            let elif_body = self.parse_block()?;
            elif_branches.push((elif_condition, elif_body));
        }

        let else_branch = if self.match_token(&TokenKind::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        }))
    }

    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::While, "Expected 'while'")?;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::While(WhileStatement { condition, body }))
    }

    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::For, "Expected 'for'")?;

        let variable = self
            .consume_identifier("Expected loop variable")?
            .lexeme
            .clone();
        self.consume(&TokenKind::In, "Expected 'in' after loop variable")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For(ForStatement {
            variable,
            iterable,
            body,
        }))
    }

    fn parse_match_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::Match, "Expected 'match'")?;

        let expression = self.parse_expression()?;
        self.consume(&TokenKind::LeftBrace, "Expected '{' after match expression")?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(&TokenKind::Arrow, "Expected '->' after match pattern")?;
            let body = self.parse_statement()?;
            arms.push(MatchArm { pattern, body });
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after match arms")?;

        Ok(Statement::Match(MatchStatement { expression, arms }))
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::Return, "Expected 'return'")?;

        let value = if self.check(&TokenKind::Newline)
            || self.check(&TokenKind::RightBrace)
            || self.check(&TokenKind::Dedent)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return(ReturnStatement { value }))
    }

    fn parse_assignment_operator(&mut self) -> Option<AssignmentOperator> {
        match &self.peek().kind {
            TokenKind::Equal => {
                self.advance();
                Some(AssignmentOperator::Assign)
            }
            TokenKind::PlusEqual => {
                self.advance();
                Some(AssignmentOperator::AddAssign)
            }
            TokenKind::MinusEqual => {
                self.advance();
                Some(AssignmentOperator::SubAssign)
            }
            TokenKind::StarEqual => {
                self.advance();
                Some(AssignmentOperator::MulAssign)
            }
            TokenKind::SlashEqual => {
                self.advance();
                Some(AssignmentOperator::DivAssign)
            }
            TokenKind::PercentEqual => {
                self.advance();
                Some(AssignmentOperator::ModAssign)
            }
            TokenKind::AmpersandEqual => {
                self.advance();
                Some(AssignmentOperator::BitwiseAndAssign)
            }
            TokenKind::PipeEqual => {
                self.advance();
                Some(AssignmentOperator::BitwiseOrAssign)
            }
            TokenKind::CaretEqual => {
                self.advance();
                Some(AssignmentOperator::BitwiseXorAssign)
            }
            TokenKind::LeftShiftEqual => {
                self.advance();
                Some(AssignmentOperator::LeftShiftAssign)
            }
            TokenKind::RightShiftEqual => {
                self.advance();
                Some(AssignmentOperator::RightShiftAssign)
            }
            _ => None,
        }
    }

    // Expression parsing using precedence climbing
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::Or) {
            let right = self.parse_logical_and()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_or()?;

        while let Some(op) = self.match_equality_operator() {
            let right = self.parse_bitwise_or()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn match_equality_operator(&mut self) -> Option<BinaryOperator> {
        match &self.peek().kind {
            TokenKind::EqualEqual => {
                self.advance();
                Some(BinaryOperator::Equal)
            }
            TokenKind::BangEqual => {
                self.advance();
                Some(BinaryOperator::NotEqual)
            }
            _ => None,
        }
    }

    fn parse_bitwise_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(&TokenKind::Pipe) {
            let right = self.parse_bitwise_xor()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::BitwiseOr,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(&TokenKind::Caret) {
            let right = self.parse_bitwise_and()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::BitwiseXor,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_shift()?;

        while self.match_token(&TokenKind::Ampersand) {
            let right = self.parse_shift()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::BitwiseAnd,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while let Some(op) = self.match_shift_operator() {
            let right = self.parse_comparison()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn match_shift_operator(&mut self) -> Option<BinaryOperator> {
        match &self.peek().kind {
            TokenKind::LeftShift => {
                self.advance();
                Some(BinaryOperator::LeftShift)
            }
            TokenKind::RightShift => {
                self.advance();
                Some(BinaryOperator::RightShift)
            }
            _ => None,
        }
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_addition()?;

        while let Some(op) = self.match_comparison_operator() {
            let right = self.parse_addition()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn match_comparison_operator(&mut self) -> Option<BinaryOperator> {
        match &self.peek().kind {
            TokenKind::Greater => {
                self.advance();
                Some(BinaryOperator::Greater)
            }
            TokenKind::GreaterEqual => {
                self.advance();
                Some(BinaryOperator::GreaterEqual)
            }
            TokenKind::Less => {
                self.advance();
                Some(BinaryOperator::Less)
            }
            TokenKind::LessEqual => {
                self.advance();
                Some(BinaryOperator::LessEqual)
            }
            _ => None,
        }
    }

    fn parse_addition(&mut self) -> Result<Expression> {
        let mut expr = self.parse_multiplication()?;

        while let Some(op) = self.match_additive_operator() {
            let right = self.parse_multiplication()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn match_additive_operator(&mut self) -> Option<BinaryOperator> {
        match &self.peek().kind {
            TokenKind::Plus => {
                self.advance();
                Some(BinaryOperator::Add)
            }
            TokenKind::Minus => {
                self.advance();
                Some(BinaryOperator::Subtract)
            }
            _ => None,
        }
    }

    fn parse_multiplication(&mut self) -> Result<Expression> {
        let mut expr = self.parse_power()?;

        while let Some(op) = self.match_multiplicative_operator() {
            let right = self.parse_power()?;
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn match_multiplicative_operator(&mut self) -> Option<BinaryOperator> {
        match &self.peek().kind {
            TokenKind::Star => {
                self.advance();
                Some(BinaryOperator::Multiply)
            }
            TokenKind::Slash => {
                self.advance();
                Some(BinaryOperator::Divide)
            }
            TokenKind::Percent => {
                self.advance();
                Some(BinaryOperator::Modulo)
            }
            _ => None,
        }
    }

    fn parse_power(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        if self.match_token(&TokenKind::StarStar) {
            let right = self.parse_power()?; // Right associative
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: BinaryOperator::Power,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        match &self.peek().kind {
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary(UnaryExpression {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(operand),
                }))
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary(UnaryExpression {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                }))
            }
            TokenKind::Tilde => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary(UnaryExpression {
                    operator: UnaryOperator::BitwiseNot,
                    operand: Box::new(operand),
                }))
            }
            TokenKind::Ampersand => {
                self.advance();
                // Check for &mut
                if self.match_token(&TokenKind::Mut) {
                    let operand = self.parse_unary()?;
                    Ok(Expression::Unary(UnaryExpression {
                        operator: UnaryOperator::MutableReference,
                        operand: Box::new(operand),
                    }))
                } else {
                    let operand = self.parse_unary()?;
                    Ok(Expression::Unary(UnaryExpression {
                        operator: UnaryOperator::Reference,
                        operand: Box::new(operand),
                    }))
                }
            }
            TokenKind::Star => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary(UnaryExpression {
                    operator: UnaryOperator::Dereference,
                    operand: Box::new(operand),
                }))
            }
            _ => self.parse_cast(),
        }
    }

    fn parse_cast(&mut self) -> Result<Expression> {
        let mut expr = self.parse_postfix()?;

        // Check for 'as' type cast
        while self.match_token(&TokenKind::As) {
            let target_type = self.parse_type()?;
            expr = Expression::Cast(CastExpression {
                expression: Box::new(expr),
                target_type,
            });
        }

        Ok(expr)
    }

    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.peek().kind {
                TokenKind::LeftParen => {
                    self.advance();
                    let mut arguments = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    expr = Expression::Call(CallExpression {
                        callee: Box::new(expr),
                        arguments,
                    });
                }
                TokenKind::LeftBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.consume(&TokenKind::RightBracket, "Expected ']' after array index")?;
                    expr = Expression::Index(IndexExpression {
                        object: Box::new(expr),
                        index: Box::new(index),
                    });
                }
                TokenKind::DoubleColon => {
                    self.advance();
                    let item = self.consume_identifier("Expected item name after '::'")?;
                    let item_name = item.lexeme.clone();

                    // For now, we only support module::item access from identifiers
                    if let Expression::Identifier(module_name) = expr {
                        expr = Expression::ModuleAccess(ModuleAccessExpression {
                            module: module_name,
                            item: item_name,
                        });
                    } else {
                        return Err(self.error("'::' can only be used after module names"));
                    }
                }
                TokenKind::Dot => {
                    self.advance();
                    let field = self.consume_identifier("Expected field name after '.'")?;
                    let field_name = field.lexeme.clone();

                    // Check if this is a method call
                    if self.check(&TokenKind::LeftParen) {
                        self.advance();
                        let mut arguments = Vec::new();
                        if !self.check(&TokenKind::RightParen) {
                            loop {
                                arguments.push(self.parse_expression()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                        }
                        self.consume(
                            &TokenKind::RightParen,
                            "Expected ')' after method arguments",
                        )?;
                        expr = Expression::MethodCall(MethodCallExpression {
                            object: Box::new(expr),
                            method: field_name,
                            arguments,
                        });
                    } else {
                        expr = Expression::FieldAccess(FieldAccessExpression {
                            object: Box::new(expr),
                            field: field_name,
                        });
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match &self.peek().kind {
            TokenKind::Integer(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(value)))
            }
            TokenKind::Float(f) => {
                let value = *f;
                self.advance();
                Ok(Expression::Literal(Literal::Float(value)))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(value)))
            }
            TokenKind::Char(c) => {
                let value = *c;
                self.advance();
                Ok(Expression::Literal(Literal::Char(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expression::Literal(Literal::None))
            }
            TokenKind::Identifier => {
                let name = self.advance().lexeme.clone();
                Ok(Expression::Identifier(name))
            }
            TokenKind::LeftParen => {
                self.advance();
                self.skip_newlines(); // Allow newlines after opening paren

                // Check if this is a tuple or grouped expression
                if self.check(&TokenKind::RightParen) {
                    // Empty tuple: ()
                    self.advance();
                    return Ok(Expression::Tuple(TupleExpression { elements: vec![] }));
                }

                let first_expr = self.parse_expression()?;
                self.skip_newlines(); // Allow newlines after first element

                if self.match_token(&TokenKind::Comma) {
                    // It's a tuple
                    self.skip_newlines(); // Allow newlines after comma
                    let mut elements = vec![first_expr];
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            elements.push(self.parse_expression()?);
                            self.skip_newlines(); // Allow newlines after element
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                            self.skip_newlines(); // Allow newlines after comma
                                                  // Allow trailing comma
                            if self.check(&TokenKind::RightParen) {
                                break;
                            }
                        }
                    }
                    self.skip_newlines(); // Allow newlines before closing paren
                    self.consume(&TokenKind::RightParen, "Expected ')' after tuple elements")?;
                    Ok(Expression::Tuple(TupleExpression { elements }))
                } else {
                    // Just a grouped expression
                    self.skip_newlines(); // Allow newlines before closing paren
                    self.consume(&TokenKind::RightParen, "Expected ')' after expression")?;
                    Ok(first_expr)
                }
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                self.skip_newlines(); // Allow newlines after opening bracket

                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        self.skip_newlines(); // Allow newlines after element

                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                        self.skip_newlines(); // Allow newlines after comma

                        // Allow trailing comma
                        if self.check(&TokenKind::RightBracket) {
                            break;
                        }
                    }
                }
                self.skip_newlines(); // Allow newlines before closing bracket
                self.consume(
                    &TokenKind::RightBracket,
                    "Expected ']' after array elements",
                )?;
                Ok(Expression::Array(ArrayExpression { elements }))
            }
            TokenKind::LeftBrace => {
                self.advance();
                self.skip_newlines(); // Allow newlines after opening brace

                // Check for empty dictionary/set
                if self.check(&TokenKind::RightBrace) {
                    self.advance();
                    // Empty dictionary by default
                    return Ok(Expression::Dictionary(DictionaryExpression {
                        pairs: vec![],
                    }));
                }

                // Parse first element to determine if it's a set or dictionary
                let first_expr = self.parse_expression()?;
                self.skip_newlines(); // Allow newlines after first element

                if self.match_token(&TokenKind::Colon) {
                    // It's a dictionary: {"key": value, ...}
                    self.skip_newlines(); // Allow newlines after colon
                    let first_value = self.parse_expression()?;
                    self.skip_newlines(); // Allow newlines after value
                    let mut pairs = vec![(first_expr, first_value)];

                    while self.match_token(&TokenKind::Comma) {
                        self.skip_newlines(); // Allow newlines after comma
                        if self.check(&TokenKind::RightBrace) {
                            break; // Trailing comma
                        }
                        let key = self.parse_expression()?;
                        self.skip_newlines(); // Allow newlines after key
                        self.consume(&TokenKind::Colon, "Expected ':' after dictionary key")?;
                        self.skip_newlines(); // Allow newlines after colon
                        let value = self.parse_expression()?;
                        self.skip_newlines(); // Allow newlines after value
                        pairs.push((key, value));
                    }

                    self.skip_newlines(); // Allow newlines before closing brace
                    self.consume(
                        &TokenKind::RightBrace,
                        "Expected '}' after dictionary pairs",
                    )?;
                    Ok(Expression::Dictionary(DictionaryExpression { pairs }))
                } else {
                    // It's a set: {1, 2, 3}
                    let mut elements = vec![first_expr];

                    while self.match_token(&TokenKind::Comma) {
                        self.skip_newlines(); // Allow newlines after comma
                        if self.check(&TokenKind::RightBrace) {
                            break; // Trailing comma
                        }
                        elements.push(self.parse_expression()?);
                        self.skip_newlines(); // Allow newlines after element
                    }

                    self.skip_newlines(); // Allow newlines before closing brace
                    self.consume(&TokenKind::RightBrace, "Expected '}' after set elements")?;
                    Ok(Expression::Set(SetExpression { elements }))
                }
            }
            TokenKind::If => self.parse_if_expression(),
            TokenKind::Match => self.parse_match_expression(),
            TokenKind::Await => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Await(AwaitExpression {
                    expression: Box::new(expr),
                }))
            }
            TokenKind::Spawn => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Spawn(SpawnExpression {
                    expression: Box::new(expr),
                }))
            }
            _ => {
                let token = self.peek();
                let msg = match &token.kind {
                    TokenKind::RightBrace => "Unexpected '}'. Check for mismatched braces",
                    TokenKind::RightParen => "Unexpected ')'. Check for mismatched parentheses",
                    TokenKind::RightBracket => "Unexpected ']'. Check for mismatched brackets",
                    TokenKind::Semicolon => "Unexpected ';'. Veyra uses newlines instead of semicolons",
                    TokenKind::Eof => "Unexpected end of file. Expected an expression",
                    _ => "Expected an expression (literal, identifier, array, dictionary, tuple, or function call)",
                };
                Err(self.error(msg))
            }
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        self.consume(&TokenKind::If, "Expected 'if'")?;
        let condition = self.parse_expression()?;

        // Expect 'then' keyword for expression form
        if !self.match_token(&TokenKind::Identifier) || self.previous().lexeme != "then" {
            return Err(self.error("Expected 'then' in if expression"));
        }

        let then_expr = self.parse_expression()?;
        self.consume(&TokenKind::Else, "Expected 'else' in if expression")?;
        let else_expr = self.parse_expression()?;

        Ok(Expression::If(IfExpression {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        }))
    }

    fn parse_match_expression(&mut self) -> Result<Expression> {
        self.consume(&TokenKind::Match, "Expected 'match'")?;
        let expression = self.parse_expression()?;
        self.consume(&TokenKind::LeftBrace, "Expected '{' after match expression")?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(&TokenKind::Arrow, "Expected '->' after pattern")?;
            let expression = self.parse_expression()?;
            arms.push(MatchExpressionArm {
                pattern,
                expression,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after match arms")?;

        Ok(Expression::Match(MatchExpression {
            expression: Box::new(expression),
            arms,
        }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match &self.peek().kind {
            TokenKind::Identifier => {
                let name = self.advance().lexeme.clone();
                if name == "_" {
                    Ok(Pattern::Wildcard)
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            TokenKind::Integer(n) => {
                let value = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Integer(value)))
            }
            TokenKind::Float(f) => {
                let value = *f;
                self.advance();
                Ok(Pattern::Literal(Literal::Float(value)))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(false)))
            }
            TokenKind::None => {
                self.advance();
                Ok(Pattern::Literal(Literal::None))
            }
            _ => Err(self.error("Expected pattern")),
        }
    }

    // Utility methods
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<&Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<&Token> {
        if self.check(&TokenKind::Identifier) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&TokenKind::Newline) {
            // Skip newlines
        }
    }

    fn error(&self, message: &str) -> VeyraError {
        let token = self.peek();
        let detailed_message = format!("{}, found '{}'", message, token.lexeme);
        VeyraError::parse_error(token.line, token.column, detailed_message)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
