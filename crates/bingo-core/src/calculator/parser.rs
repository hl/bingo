//! Parser for calculator DSL expressions
//!
//! Implements a recursive descent parser with operator precedence climbing
//! for parsing mathematical and logical expressions into AST nodes.

use crate::calculator::ast::{BinaryOperator, Expression, UnaryOperator};
use anyhow::{Result, anyhow};
use std::fmt;

/// Token types recognized by the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),

    // Identifiers and keywords
    Identifier(String),
    If,
    Then,
    Else,
    Cond,
    When,
    Default,
    True,
    False,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    And,
    Or,
    Not,
    Concat,
    Contains,
    StartsWith,
    EndsWith,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Colon,

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::Identifier(name) => write!(f, "{}", name),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Cond => write!(f, "cond"),
            Token::When => write!(f, "when"),
            Token::Default => write!(f, "default"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Power => write!(f, "**"),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::LessThan => write!(f, "<"),
            Token::LessThanEqual => write!(f, "<="),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterThanEqual => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Concat => write!(f, "++"),
            Token::Contains => write!(f, "contains"),
            Token::StartsWith => write!(f, "starts_with"),
            Token::EndsWith => write!(f, "ends_with"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Lexer for tokenizing calculator expressions
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();

        Self { input: chars, position: 0, current_char }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let mut number = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            let value = number
                .parse::<f64>()
                .map_err(|e| anyhow!("Invalid float '{}': {}", number, e))?;
            Ok(Token::Float(value))
        } else {
            let value = number
                .parse::<i64>()
                .map_err(|e| anyhow!("Invalid integer '{}': {}", number, e))?;
            Ok(Token::Integer(value))
        }
    }

    fn read_string(&mut self) -> Result<Token> {
        let mut string = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(Token::String(string));
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some(other) => {
                        string.push('\\');
                        string.push(other);
                    }
                    None => return Err(anyhow!("Unterminated string literal")),
                }
                self.advance();
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(anyhow!("Unterminated string literal"))
    }

    fn read_identifier(&mut self) -> Token {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        match identifier.as_str() {
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "cond" => Token::Cond,
            "when" => Token::When,
            "default" => Token::Default,
            "true" => Token::True,
            "false" => Token::False,
            "contains" => Token::Contains,
            "starts_with" => Token::StartsWith,
            "ends_with" => Token::EndsWith,
            _ => Token::Identifier(identifier),
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::Eof),
            Some(ch) => match ch {
                '0'..='9' => self.read_number(),
                '"' => self.read_string(),
                'a'..='z' | 'A'..='Z' | '_' => Ok(self.read_identifier()),
                '+' => {
                    if self.peek() == Some('+') {
                        self.advance();
                        self.advance();
                        Ok(Token::Concat)
                    } else {
                        self.advance();
                        Ok(Token::Plus)
                    }
                }
                '-' => {
                    self.advance();
                    Ok(Token::Minus)
                }
                '*' => {
                    if self.peek() == Some('*') {
                        self.advance();
                        self.advance();
                        Ok(Token::Power)
                    } else {
                        self.advance();
                        Ok(Token::Star)
                    }
                }
                '/' => {
                    self.advance();
                    Ok(Token::Slash)
                }
                '%' => {
                    self.advance();
                    Ok(Token::Percent)
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        Ok(Token::Equal)
                    } else {
                        Err(anyhow!("Unexpected character '='. Did you mean '=='?"))
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        Ok(Token::NotEqual)
                    } else {
                        self.advance();
                        Ok(Token::Not)
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        Ok(Token::LessThanEqual)
                    } else {
                        self.advance();
                        Ok(Token::LessThan)
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        Ok(Token::GreaterThanEqual)
                    } else {
                        self.advance();
                        Ok(Token::GreaterThan)
                    }
                }
                '&' => {
                    if self.peek() == Some('&') {
                        self.advance();
                        self.advance();
                        Ok(Token::And)
                    } else {
                        Err(anyhow!("Unexpected character '&'. Did you mean '&&'?"))
                    }
                }
                '|' => {
                    if self.peek() == Some('|') {
                        self.advance();
                        self.advance();
                        Ok(Token::Or)
                    } else {
                        Err(anyhow!("Unexpected character '|'. Did you mean '||'?"))
                    }
                }
                '(' => {
                    self.advance();
                    Ok(Token::LeftParen)
                }
                ')' => {
                    self.advance();
                    Ok(Token::RightParen)
                }
                '[' => {
                    self.advance();
                    Ok(Token::LeftBracket)
                }
                ']' => {
                    self.advance();
                    Ok(Token::RightBracket)
                }
                '{' => {
                    self.advance();
                    Ok(Token::LeftBrace)
                }
                '}' => {
                    self.advance();
                    Ok(Token::RightBrace)
                }
                ',' => {
                    self.advance();
                    Ok(Token::Comma)
                }
                '.' => {
                    self.advance();
                    Ok(Token::Dot)
                }
                ':' => {
                    self.advance();
                    Ok(Token::Colon)
                }
                _ => Err(anyhow!("Unexpected character '{}'", ch)),
            },
        }
    }
}

/// Parser for calculator expressions
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        let current_token = lexer.next_token()?;
        Ok(Self { lexer, current_token })
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance()
        } else {
            Err(anyhow!(
                "Expected {}, found {}",
                expected,
                self.current_token
            ))
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;

        while matches!(self.current_token, Token::Or) {
            self.advance()?;
            let right = self.parse_and_expression()?;
            left = Expression::binary(left, BinaryOperator::Or, right);
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_equality_expression()?;

        while matches!(self.current_token, Token::And) {
            self.advance()?;
            let right = self.parse_equality_expression()?;
            left = Expression::binary(left, BinaryOperator::And, right);
        }

        Ok(left)
    }

    fn parse_equality_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison_expression()?;

        while matches!(self.current_token, Token::Equal | Token::NotEqual) {
            let op = match self.current_token {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_comparison_expression()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_string_expression()?;

        while matches!(
            self.current_token,
            Token::LessThan | Token::LessThanEqual | Token::GreaterThan | Token::GreaterThanEqual
        ) {
            let op = match self.current_token {
                Token::LessThan => BinaryOperator::LessThan,
                Token::LessThanEqual => BinaryOperator::LessThanOrEqual,
                Token::GreaterThan => BinaryOperator::GreaterThan,
                Token::GreaterThanEqual => BinaryOperator::GreaterThanOrEqual,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_string_expression()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_string_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive_expression()?;

        while matches!(
            self.current_token,
            Token::Concat | Token::Contains | Token::StartsWith | Token::EndsWith
        ) {
            let op = match self.current_token {
                Token::Concat => BinaryOperator::Concat,
                Token::Contains => BinaryOperator::Contains,
                Token::StartsWith => BinaryOperator::StartsWith,
                Token::EndsWith => BinaryOperator::EndsWith,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_additive_expression()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative_expression()?;

        while matches!(self.current_token, Token::Plus | Token::Minus) {
            let op = match self.current_token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_multiplicative_expression()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_power_expression()?;

        while matches!(
            self.current_token,
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = match self.current_token {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_power_expression()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_power_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;

        // Power is right-associative
        if matches!(self.current_token, Token::Power) {
            self.advance()?;
            let right = self.parse_power_expression()?;
            left = Expression::binary(left, BinaryOperator::Power, right);
        }

        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression> {
        match self.current_token {
            Token::Minus => {
                self.advance()?;
                let operand = self.parse_unary_expression()?;
                Ok(Expression::unary(UnaryOperator::Negate, operand))
            }
            Token::Not => {
                self.advance()?;
                let operand = self.parse_unary_expression()?;
                Ok(Expression::unary(UnaryOperator::Not, operand))
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match self.current_token {
                Token::Dot => {
                    self.advance()?;
                    if let Token::Identifier(field) = &self.current_token {
                        let field_name = field.clone();
                        self.advance()?;
                        expr = Expression::field(expr, &field_name);
                    } else {
                        return Err(anyhow!("Expected field name after '.'"));
                    }
                }
                Token::LeftParen => {
                    // Function call
                    if let Expression::Variable(name) = expr {
                        self.advance()?; // consume '('
                        let mut args = Vec::new();

                        if !matches!(self.current_token, Token::RightParen) {
                            args.push(self.parse_expression()?);

                            while matches!(self.current_token, Token::Comma) {
                                self.advance()?;
                                args.push(self.parse_expression()?);
                            }
                        }

                        self.expect(Token::RightParen)?;
                        expr = Expression::call(&name, args);
                    } else {
                        return Err(anyhow!("Only identifiers can be called as functions"));
                    }
                }
                Token::LeftBracket => {
                    // Array indexing
                    self.advance()?; // consume '['
                    let index = self.parse_expression()?;
                    self.expect(Token::RightBracket)?;
                    expr = Expression::index(expr, index);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Integer(value) => {
                let val = *value;
                self.advance()?;
                Ok(Expression::int(val))
            }
            Token::Float(value) => {
                let val = *value;
                self.advance()?;
                Ok(Expression::float(val))
            }
            Token::String(value) => {
                let val = value.clone();
                self.advance()?;
                Ok(Expression::string(val))
            }
            Token::True => {
                self.advance()?;
                Ok(Expression::bool(true))
            }
            Token::False => {
                self.advance()?;
                Ok(Expression::bool(false))
            }
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.advance()?;
                Ok(Expression::var(&var_name))
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                // Array literal [1, 2, 3]
                self.advance()?; // consume '['
                let mut elements = Vec::new();

                if !matches!(self.current_token, Token::RightBracket) {
                    elements.push(self.parse_expression()?);

                    while matches!(self.current_token, Token::Comma) {
                        self.advance()?;
                        // Allow trailing comma
                        if matches!(self.current_token, Token::RightBracket) {
                            break;
                        }
                        elements.push(self.parse_expression()?);
                    }
                }

                self.expect(Token::RightBracket)?;
                Ok(Expression::array(elements))
            }
            Token::LeftBrace => {
                // Object literal {key: value, ...}
                self.advance()?; // consume '{'
                let mut fields = Vec::new();

                if !matches!(self.current_token, Token::RightBrace) {
                    // Parse first key-value pair
                    let key = self.parse_object_key()?;
                    self.expect(Token::Colon)?;
                    let value = self.parse_expression()?;
                    fields.push((key, value));

                    while matches!(self.current_token, Token::Comma) {
                        self.advance()?;
                        // Allow trailing comma
                        if matches!(self.current_token, Token::RightBrace) {
                            break;
                        }
                        let key = self.parse_object_key()?;
                        self.expect(Token::Colon)?;
                        let value = self.parse_expression()?;
                        fields.push((key, value));
                    }
                }

                self.expect(Token::RightBrace)?;
                Ok(Expression::object(fields))
            }
            Token::If => {
                self.advance()?;
                let condition = self.parse_expression()?;
                self.expect(Token::Then)?;
                let then_expr = self.parse_expression()?;
                self.expect(Token::Else)?;
                let else_expr = self.parse_expression()?;
                Ok(Expression::conditional(condition, then_expr, else_expr))
            }
            Token::Cond => {
                self.advance()?;
                let mut conditions = Vec::new();
                let mut default_value = None;

                // Parse condition-value pairs
                while matches!(self.current_token, Token::When) {
                    self.advance()?; // consume 'when'
                    let condition = self.parse_expression()?;
                    self.expect(Token::Then)?;
                    let value = self.parse_expression()?;
                    conditions.push((condition, value));
                }

                // Parse optional default value
                if matches!(self.current_token, Token::Default) {
                    self.advance()?; // consume 'default'
                    default_value = Some(self.parse_expression()?);
                }

                Ok(Expression::conditional_set(conditions, default_value))
            }
            _ => Err(anyhow!("Unexpected token: {}", self.current_token)),
        }
    }

    /// Parse an object key (identifier or string literal)
    fn parse_object_key(&mut self) -> Result<String> {
        match &self.current_token {
            Token::Identifier(name) => {
                let key = name.clone();
                self.advance()?;
                Ok(key)
            }
            Token::String(s) => {
                let key = s.clone();
                self.advance()?;
                Ok(key)
            }
            _ => Err(anyhow!(
                "Expected identifier or string for object key, found {}",
                self.current_token
            )),
        }
    }
}

/// Parse an expression string into an AST
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer)?;
    let expr = parser.parse_expression()?;

    // Ensure we've consumed all tokens
    if !matches!(parser.current_token, Token::Eof) {
        return Err(anyhow!(
            "Unexpected token after expression: {}",
            parser.current_token
        ));
    }

    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculator::ast::BinaryOperator;

    #[test]
    fn test_lexer_basic() {
        let mut lexer = Lexer::new("123 + 45.67");

        assert_eq!(lexer.next_token().unwrap(), Token::Integer(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Plus);
        assert_eq!(lexer.next_token().unwrap(), Token::Float(45.67));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new(r#""hello world""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("hello world".to_string())
        );
    }

    #[test]
    fn test_parser_arithmetic() {
        let expr = parse_expression("2 + 3 * 4").unwrap();

        // Should parse as 2 + (3 * 4) due to operator precedence
        match expr {
            Expression::BinaryOp { left, operator: BinaryOperator::Add, right } => {
                assert_eq!(left.as_ref(), &Expression::int(2));
                match right.as_ref() {
                    Expression::BinaryOp {
                        left: mult_left,
                        operator: BinaryOperator::Multiply,
                        right: mult_right,
                    } => {
                        assert_eq!(mult_left.as_ref(), &Expression::int(3));
                        assert_eq!(mult_right.as_ref(), &Expression::int(4));
                    }
                    _ => panic!("Expected multiplication on right side"),
                }
            }
            _ => panic!("Expected addition at top level"),
        }
    }

    #[test]
    fn test_parser_function_call() {
        let expr = parse_expression("max(10, 20)").unwrap();

        match expr {
            Expression::FunctionCall { name, args } => {
                assert_eq!(name, "max");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Expression::int(10));
                assert_eq!(args[1], Expression::int(20));
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parser_conditional() {
        let expr = parse_expression("if x > 0 then x else 0").unwrap();

        match expr {
            Expression::Conditional { condition, then_expr, else_expr } => {
                match condition.as_ref() {
                    Expression::BinaryOp {
                        left: cond_left,
                        operator: BinaryOperator::GreaterThan,
                        right: cond_right,
                    } => {
                        assert_eq!(cond_left.as_ref(), &Expression::var("x"));
                        assert_eq!(cond_right.as_ref(), &Expression::int(0));
                    }
                    _ => panic!("Expected comparison in condition"),
                }
                assert_eq!(then_expr.as_ref(), &Expression::var("x"));
                assert_eq!(else_expr.as_ref(), &Expression::int(0));
            }
            _ => panic!("Expected conditional expression"),
        }
    }

    #[test]
    fn test_parser_conditional_set() {
        let expr = parse_expression(
            "cond when rating >= 4.5 then 0.15 when rating >= 4.0 then 0.10 default 0.0",
        )
        .unwrap();

        match expr {
            Expression::ConditionalSet { conditions, default_value } => {
                assert_eq!(conditions.len(), 2);

                // Check first condition
                let (condition, value) = &conditions[0];
                {
                    match condition {
                        Expression::BinaryOp {
                            left,
                            operator: BinaryOperator::GreaterThanOrEqual,
                            right,
                        } => {
                            assert_eq!(left.as_ref(), &Expression::var("rating"));
                            assert_eq!(right.as_ref(), &Expression::float(4.5));
                        }
                        _ => panic!("Expected rating >= 4.5 condition"),
                    }
                    assert_eq!(value, &Expression::float(0.15));
                }

                // Check second condition
                let (condition, value) = &conditions[1];
                {
                    match condition {
                        Expression::BinaryOp {
                            left,
                            operator: BinaryOperator::GreaterThanOrEqual,
                            right,
                        } => {
                            assert_eq!(left.as_ref(), &Expression::var("rating"));
                            assert_eq!(right.as_ref(), &Expression::float(4.0));
                        }
                        _ => panic!("Expected rating >= 4.0 condition"),
                    }
                    assert_eq!(value, &Expression::float(0.10));
                }

                // Check default value
                assert!(default_value.is_some());
                assert_eq!(
                    default_value.as_ref().unwrap().as_ref(),
                    &Expression::float(0.0)
                );
            }
            _ => panic!("Expected conditional set expression"),
        }
    }

    #[test]
    fn test_parser_array_literal() {
        let expr = parse_expression("[1, 2, 3]").unwrap();

        match expr {
            Expression::ArrayLiteral { elements } => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expression::int(1));
                assert_eq!(elements[1], Expression::int(2));
                assert_eq!(elements[2], Expression::int(3));
            }
            _ => panic!("Expected array literal"),
        }
    }

    #[test]
    fn test_parser_empty_array() {
        let expr = parse_expression("[]").unwrap();

        match expr {
            Expression::ArrayLiteral { elements } => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected empty array literal"),
        }
    }

    #[test]
    fn test_parser_object_literal() {
        let expr = parse_expression(r#"{"name": "John", age: 30}"#).unwrap();

        match expr {
            Expression::ObjectLiteral { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[0].1, Expression::string("John".to_string()));
                assert_eq!(fields[1].0, "age");
                assert_eq!(fields[1].1, Expression::int(30));
            }
            _ => panic!("Expected object literal"),
        }
    }

    #[test]
    fn test_parser_empty_object() {
        let expr = parse_expression("{}").unwrap();

        match expr {
            Expression::ObjectLiteral { fields } => {
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected empty object literal"),
        }
    }

    #[test]
    fn test_parser_array_indexing() {
        let expr = parse_expression("arr[0]").unwrap();

        match expr {
            Expression::ArrayIndex { array, index } => {
                assert_eq!(array.as_ref(), &Expression::var("arr"));
                assert_eq!(index.as_ref(), &Expression::int(0));
            }
            _ => panic!("Expected array indexing"),
        }
    }

    #[test]
    fn test_parser_nested_structures() {
        let expr = parse_expression(r#"{"users": [1, 2, 3], "active": true}"#).unwrap();

        match expr {
            Expression::ObjectLiteral { fields } => {
                assert_eq!(fields.len(), 2);

                // Check first field: "users": [1, 2, 3]
                assert_eq!(fields[0].0, "users");
                match &fields[0].1 {
                    Expression::ArrayLiteral { elements } => {
                        assert_eq!(elements.len(), 3);
                        assert_eq!(elements[0], Expression::int(1));
                        assert_eq!(elements[1], Expression::int(2));
                        assert_eq!(elements[2], Expression::int(3));
                    }
                    _ => panic!("Expected array in users field"),
                }

                // Check second field: "active": true
                assert_eq!(fields[1].0, "active");
                assert_eq!(fields[1].1, Expression::bool(true));
            }
            _ => panic!("Expected object literal"),
        }
    }
}
