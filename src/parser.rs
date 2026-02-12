//! # SQL Parser Module
//!
//! This module provides SQL parsing functionality for the Mini Rust OLAP database.
//!
//! ## Design Philosophy
//!
//! This parser uses a recursive descent approach, which is:
//! - **Educational**: Clear and easy to understand
//! - **Dependency-free**: No external parsing libraries required
//! - **Maintainable**: Each parsing rule is a separate function
//!
//! ## Architecture
//!
//! The parser consists of three main layers:
//! 1. **Tokenizer/Lexer**: Converts raw SQL text into tokens
//! 2. **AST Definition**: Defines the structure of parsed queries
//! 3. **Parser**: Converts tokens into Abstract Syntax Tree (AST)
//!
//! ## Supported SQL Features
//!
//! - SELECT statement with column selection
//! - FROM clause with table names
//! - WHERE clause with comparison and logical operators
//! - GROUP BY clause
//! - Aggregate functions: COUNT, SUM, AVG, MIN, MAX
//! - Wildcard (*) in SELECT
//!
//! ## Example Usage
//!
//! ```ignore
//! use mini_rust_olap::parser::Parser;
//! use mini_rust_olap::error::Result;
//!
//! fn parse_query(sql: &str) -> Result<Query> {
//!     let parser = Parser::new(sql);
//!     parser.parse()
//! }
//! ```

use crate::error::{DatabaseError, Result};
use crate::types::SortDirection;

// ============================================================================
// TOKEN DEFINITIONS
// ============================================================================

/// Represents the different types of tokens that can appear in SQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Keywords
    Select,
    From,
    Where,
    Group,
    By,
    And,
    Or,
    Not,
    Order,
    Limit,
    Offset,
    Asc,
    Desc,

    // Aggregate functions
    Count,
    Sum,
    Avg,
    Min,
    Max,

    // Operators
    Equal,        // =
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Plus,         // +
    Minus,        // -
    Divide,       // /

    // Punctuation
    LeftParen,  // (
    RightParen, // )
    Comma,      // ,
    Asterisk,   // *

    // Literals and identifiers
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),

    // Special tokens
    EOF,
}

/// Represents a single token with its type and position information.
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    /// Creates a new token.
    fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Token {
            token_type,
            line,
            column,
        }
    }

    /// Returns a string representation of the token's value.
    pub fn value(&self) -> String {
        match &self.token_type {
            TokenType::Identifier(s) => s.clone(),
            TokenType::StringLiteral(s) => s.clone(),
            TokenType::NumberLiteral(s) => s.clone(),
            TokenType::Count => "COUNT".to_string(),
            TokenType::Sum => "SUM".to_string(),
            TokenType::Avg => "AVG".to_string(),
            TokenType::Min => "MIN".to_string(),
            TokenType::Max => "MAX".to_string(),
            _ => format!("{:?}", self.token_type),
        }
    }
}

// ============================================================================
// TOKENIZER / LEXER
// ============================================================================

/// Tokenizer (Lexer) that converts SQL text into a stream of tokens.
///
/// The tokenizer processes the input string character by character,
/// identifying keywords, identifiers, literals, operators, and punctuation.
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    /// Creates a new tokenizer for the given SQL input.
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenizes the entire input and returns a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            // Skip whitespace
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        // Add EOF token
        tokens.push(Token::new(TokenType::EOF, self.line, self.column));

        Ok(tokens)
    }

    /// Returns the next token from the input.
    fn next_token(&mut self) -> Result<Token> {
        let line = self.line;
        let column = self.column;
        let c = self.peek().unwrap();

        match c {
            // Single-character tokens
            '(' => {
                self.advance();
                Ok(Token::new(TokenType::LeftParen, line, column))
            }
            ')' => {
                self.advance();
                Ok(Token::new(TokenType::RightParen, line, column))
            }
            ',' => {
                self.advance();
                Ok(Token::new(TokenType::Comma, line, column))
            }
            '*' => {
                self.advance();
                Ok(Token::new(TokenType::Asterisk, line, column))
            }
            '+' => {
                self.advance();
                Ok(Token::new(TokenType::Plus, line, column))
            }
            '-' => {
                self.advance();
                Ok(Token::new(TokenType::Minus, line, column))
            }
            '/' => {
                self.advance();
                Ok(Token::new(TokenType::Divide, line, column))
            }

            // Operators
            '=' => {
                self.advance();
                Ok(Token::new(TokenType::Equal, line, column))
            }
            '!' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::new(TokenType::NotEqual, line, column))
                } else {
                    Err(DatabaseError::parser_error(format!(
                        "Expected '=' after '!' at line {}, column {}",
                        line, column
                    )))
                }
            }
            '<' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::new(TokenType::LessEqual, line, column))
                } else {
                    Ok(Token::new(TokenType::Less, line, column))
                }
            }
            '>' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::new(TokenType::GreaterEqual, line, column))
                } else {
                    Ok(Token::new(TokenType::Greater, line, column))
                }
            }

            // String literals
            '\'' => self.string_literal(line, column),

            // Numbers
            '0'..='9' => self.number_literal(line, column),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword(line, column),

            // Invalid character
            _ => Err(DatabaseError::parser_error(format!(
                "Unexpected character '{}' at line {}, column {}",
                c, line, column
            ))),
        }
    }

    /// Parses a string literal enclosed in single quotes.
    fn string_literal(&mut self, line: usize, column: usize) -> Result<Token> {
        self.advance(); // Skip opening quote
        let mut value = String::new();

        while let Some(&c) = self.peek() {
            if c == '\'' {
                self.advance(); // Skip closing quote
                return Ok(Token::new(TokenType::StringLiteral(value), line, column));
            }
            value.push(c);
            self.advance();
        }

        Err(DatabaseError::parser_error("Unterminated string literal"))
    }

    /// Parses a numeric literal (integer or float).
    fn number_literal(&mut self, line: usize, column: usize) -> Result<Token> {
        let mut value = String::new();

        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if value.is_empty() {
            return Err(DatabaseError::parser_error("Expected number"));
        }

        Ok(Token::new(TokenType::NumberLiteral(value), line, column))
    }

    /// Parses an identifier or keyword.
    fn identifier_or_keyword(&mut self, line: usize, column: usize) -> Result<Token> {
        let mut value = String::new();

        while let Some(&c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        let token_type = match value.to_uppercase().as_str() {
            "SELECT" => TokenType::Select,
            "FROM" => TokenType::From,
            "WHERE" => TokenType::Where,
            "GROUP" => TokenType::Group,
            "BY" => TokenType::By,
            "AND" => TokenType::And,
            "OR" => TokenType::Or,
            "NOT" => TokenType::Not,
            "ORDER" => TokenType::Order,
            "LIMIT" => TokenType::Limit,
            "OFFSET" => TokenType::Offset,
            "ASC" => TokenType::Asc,
            "DESC" => TokenType::Desc,
            "COUNT" => TokenType::Count,
            "SUM" => TokenType::Sum,
            "AVG" => TokenType::Avg,
            "MIN" => TokenType::Min,
            "MAX" => TokenType::Max,
            _ => TokenType::Identifier(value.to_lowercase()),
        };

        Ok(Token::new(token_type, line, column))
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Returns the current character without consuming it.
    fn peek(&self) -> Option<&char> {
        self.input.get(self.position)
    }

    /// Advances to the next character.
    fn advance(&mut self) {
        self.position += 1;
        if self.position <= self.input.len() {
            self.column += 1;
        }
    }

    /// Checks if the current character matches the expected character,
    /// and consumes it if it does.
    fn match_char(&mut self, expected: char) -> bool {
        if let Some(&c) = self.peek() {
            if c == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Returns true if we've reached the end of the input.
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

// ============================================================================
// AST DEFINITIONS
// ============================================================================

/// Represents a complete SQL query.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// SELECT query
    Select(SelectStatement),
}

/// Represents a SELECT statement with all its clauses.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    /// Columns or expressions to select
    pub select_items: Vec<SelectItem>,
    /// Table name in the FROM clause
    pub from_table: String,
    /// Optional WHERE clause condition
    pub where_clause: Option<Expression>,
    /// Optional GROUP BY columns
    pub group_by: Option<Vec<String>>,
    /// Optional ORDER BY clause
    pub order_by: Option<Vec<OrderByItem>>,
    /// Optional LIMIT clause
    pub limit: Option<usize>,
    /// Optional OFFSET clause
    pub offset: Option<usize>,
}

/// Represents an item in the SELECT clause.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectItem {
    /// Wildcard (*)
    Wildcard,
    /// An expression (column reference, aggregate function, etc.)
    Expression(Expression),
}

/// Represents an item in the ORDER BY clause.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    /// Column name to sort by
    pub column: String,
    /// Sort direction (ASC or DESC)
    pub direction: SortDirection,
}

/// Represents an expression in a SQL query.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Column reference
    Column(String),
    /// String literal
    StringLiteral(String),
    /// Number literal (integer or float)
    NumberLiteral(String),
    /// Aggregate function call
    AggregateFunction {
        /// Function name (COUNT, SUM, AVG, MIN, MAX)
        function: String,
        /// The argument to the aggregate function
        argument: Box<Expression>,
    },
    /// Binary operation (e.g., age > 25)
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    /// Unary operation (e.g., NOT condition, -value)
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
}

/// Represents binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Plus,
    Minus,
    Multiply,
    Divide,
}

/// Represents unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Minus,
}

// ============================================================================
// PARSER
// ============================================================================

/// SQL Parser that converts tokens into an Abstract Syntax Tree (AST).
///
/// This is a recursive descent parser where each grammar rule is
/// implemented as a separate method.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Creates a new parser for the given SQL input.
    pub fn new(sql: &str) -> Self {
        let mut tokenizer = Tokenizer::new(sql);
        let tokens = tokenizer.tokenize().unwrap_or_else(|e| {
            eprintln!("Tokenizer error: {}", e);
            Vec::new()
        });

        Parser {
            tokens,
            position: 0,
        }
    }

    /// Parses the SQL input into a Query AST.
    pub fn parse(&mut self) -> Result<Query> {
        self.parse_query()
    }

    /// Parses a complete query (currently only SELECT is supported).
    fn parse_query(&mut self) -> Result<Query> {
        match self.peek_token_type() {
            Some(TokenType::Select) => {
                let select_statement = self.parse_select_statement()?;
                Ok(Query::Select(select_statement))
            }
            Some(token_type) => Err(DatabaseError::parser_error(format!(
                "Expected SELECT, found {:?}",
                token_type
            ))),
            None => Err(DatabaseError::parser_error("Unexpected end of input")),
        }
    }

    /// Parses a SELECT statement.
    fn parse_select_statement(&mut self) -> Result<SelectStatement> {
        // Parse SELECT clause
        self.consume_token(TokenType::Select, "Expected SELECT")?;
        let select_items = self.parse_select_items()?;

        // Parse FROM clause
        self.consume_token(TokenType::From, "Expected FROM")?;
        let from_table = self.parse_identifier()?;

        // Parse optional WHERE clause
        let where_clause = if self.match_token(TokenType::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse optional GROUP BY clause
        let group_by = if self.match_token(TokenType::Group) {
            self.consume_token(TokenType::By, "Expected BY after GROUP")?;
            Some(self.parse_group_by_columns()?)
        } else {
            None
        };

        // Parse optional ORDER BY clause
        let order_by = if self.match_token(TokenType::Order) {
            self.consume_token(TokenType::By, "Expected BY after ORDER")?;
            Some(self.parse_order_by_items()?)
        } else {
            None
        };

        // Parse optional LIMIT clause
        let limit = if self.match_token(TokenType::Limit) {
            Some(self.parse_number_literal()?)
        } else {
            None
        };

        // Parse optional OFFSET clause
        let offset = if self.match_token(TokenType::Offset) {
            Some(self.parse_number_literal()?)
        } else {
            None
        };

        // Should be at EOF now
        self.consume_token(TokenType::EOF, "Expected end of statement")?;

        Ok(SelectStatement {
            select_items,
            from_table,
            where_clause,
            group_by,
            order_by,
            limit,
            offset,
        })
    }

    /// Parses the SELECT clause items.
    fn parse_select_items(&mut self) -> Result<Vec<SelectItem>> {
        let mut items = Vec::new();

        // First item (required)
        items.push(self.parse_select_item()?);

        // Additional items separated by commas
        while self.match_token(TokenType::Comma) {
            items.push(self.parse_select_item()?);
        }

        Ok(items)
    }

    /// Parses a single SELECT item.
    fn parse_select_item(&mut self) -> Result<SelectItem> {
        if self.match_token(TokenType::Asterisk) {
            Ok(SelectItem::Wildcard)
        } else {
            let expr = self.parse_expression()?;
            Ok(SelectItem::Expression(expr))
        }
    }

    /// Parses an expression.
    ///
    /// Expressions follow the precedence: OR > AND > comparisons > arithmetic > unary > primary.
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or_expression()
    }

    /// Parses OR expressions (lowest precedence).
    fn parse_or_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;

        while self.match_token(TokenType::Or) {
            let operator = BinaryOperator::Or;
            let right = self.parse_and_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses AND expressions.
    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison_expression()?;

        while self.match_token(TokenType::And) {
            let operator = BinaryOperator::And;
            let right = self.parse_comparison_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses comparison expressions (=, !=, <, >, <=, >=).
    fn parse_comparison_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive_expression()?;

        while let Some(token_type) = self.peek_token_type() {
            let operator = match token_type {
                TokenType::Equal => {
                    self.advance();
                    BinaryOperator::Equal
                }
                TokenType::NotEqual => {
                    self.advance();
                    BinaryOperator::NotEqual
                }
                TokenType::Less => {
                    self.advance();
                    BinaryOperator::Less
                }
                TokenType::Greater => {
                    self.advance();
                    BinaryOperator::Greater
                }
                TokenType::LessEqual => {
                    self.advance();
                    BinaryOperator::LessEqual
                }
                TokenType::GreaterEqual => {
                    self.advance();
                    BinaryOperator::GreaterEqual
                }
                _ => break,
            };

            let right = self.parse_additive_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses additive expressions (+, -).
    fn parse_additive_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(token_type) = self.peek_token_type() {
            let operator = match token_type {
                TokenType::Plus => {
                    self.advance();
                    BinaryOperator::Plus
                }
                TokenType::Minus => {
                    self.advance();
                    BinaryOperator::Minus
                }
                _ => break,
            };

            let right = self.parse_multiplicative_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses multiplicative expressions (*, /).
    fn parse_multiplicative_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;

        while let Some(token_type) = self.peek_token_type() {
            let operator = match token_type {
                TokenType::Asterisk => {
                    self.advance();
                    BinaryOperator::Multiply
                }
                TokenType::Divide => {
                    // Check if this is really division or part of a comment
                    // For now, we'll assume it's division (comments not supported)
                    self.advance();
                    BinaryOperator::Divide
                }
                _ => break,
            };

            let right = self.parse_unary_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses unary expressions (NOT, -).
    fn parse_unary_expression(&mut self) -> Result<Expression> {
        if let Some(token_type) = self.peek_token_type() {
            match token_type {
                TokenType::Not => {
                    self.advance();
                    let operand = self.parse_unary_expression()?;
                    return Ok(Expression::UnaryOp {
                        operator: UnaryOperator::Not,
                        operand: Box::new(operand),
                    });
                }
                TokenType::Minus => {
                    self.advance();
                    let operand = self.parse_unary_expression()?;
                    return Ok(Expression::UnaryOp {
                        operator: UnaryOperator::Minus,
                        operand: Box::new(operand),
                    });
                }
                _ => {}
            }
        }

        self.parse_primary_expression()
    }

    /// Parses primary expressions (literals, identifiers, aggregate functions, parenthesized expressions).
    fn parse_primary_expression(&mut self) -> Result<Expression> {
        let token_type = self.peek_token_type();

        match token_type {
            Some(TokenType::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                // Check if this is an aggregate function call
                if self.match_token(TokenType::LeftParen) {
                    self.parse_aggregate_function(name)
                } else {
                    Ok(Expression::Column(name))
                }
            }
            Some(TokenType::StringLiteral(value)) => {
                let value = value.clone();
                self.advance();
                Ok(Expression::StringLiteral(value))
            }
            Some(TokenType::NumberLiteral(value)) => {
                let value = value.clone();
                self.advance();
                Ok(Expression::NumberLiteral(value))
            }
            Some(TokenType::Count)
            | Some(TokenType::Sum)
            | Some(TokenType::Avg)
            | Some(TokenType::Min)
            | Some(TokenType::Max) => {
                let function_name = self.peek_token().unwrap().value();
                self.advance();
                self.consume_token(
                    TokenType::LeftParen,
                    "Expected '(' after aggregate function",
                )?;
                self.parse_aggregate_function(function_name)
            }
            Some(TokenType::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume_token(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Some(TokenType::Asterisk) => {
                self.advance();
                Ok(Expression::Column("*".to_string()))
            }
            Some(token_type) => Err(DatabaseError::parser_error(format!(
                "Unexpected token: {:?}",
                token_type
            ))),
            None => Err(DatabaseError::parser_error("Unexpected end of input")),
        }
    }

    /// Parses an aggregate function call.
    fn parse_aggregate_function(&mut self, function: String) -> Result<Expression> {
        let argument = self.parse_expression()?;
        self.consume_token(
            TokenType::RightParen,
            "Expected ')' after aggregate function argument",
        )?;

        Ok(Expression::AggregateFunction {
            function,
            argument: Box::new(argument),
        })
    }

    /// Parses a column name (identifier).
    fn parse_identifier(&mut self) -> Result<String> {
        match self.peek_token() {
            Some(token) => match &token.token_type {
                TokenType::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    Ok(name)
                }
                _ => Err(DatabaseError::parser_error(format!(
                    "Expected identifier, found {:?}",
                    token.token_type
                ))),
            },
            None => Err(DatabaseError::parser_error(
                "Expected identifier, found EOF",
            )),
        }
    }

    /// Parses GROUP BY column list.
    fn parse_group_by_columns(&mut self) -> Result<Vec<String>> {
        let mut columns = Vec::new();

        columns.push(self.parse_identifier()?);

        while self.match_token(TokenType::Comma) {
            columns.push(self.parse_identifier()?);
        }

        Ok(columns)
    }

    /// Parses ORDER BY item list.
    fn parse_order_by_items(&mut self) -> Result<Vec<OrderByItem>> {
        let mut items = Vec::new();

        items.push(self.parse_order_by_item()?);

        while self.match_token(TokenType::Comma) {
            items.push(self.parse_order_by_item()?);
        }

        Ok(items)
    }

    /// Parses a single ORDER BY item (column with optional direction).
    fn parse_order_by_item(&mut self) -> Result<OrderByItem> {
        let column = self.parse_identifier()?;

        // Check for optional ASC or DESC
        let direction = if self.match_token(TokenType::Desc) {
            SortDirection::Descending
        } else if self.match_token(TokenType::Asc) {
            SortDirection::Ascending
        } else {
            // Default to ASC
            SortDirection::Ascending
        };

        Ok(OrderByItem { column, direction })
    }

    /// Parses a number literal (for LIMIT and OFFSET).
    fn parse_number_literal(&mut self) -> Result<usize> {
        match self.peek_token() {
            Some(token) => match &token.token_type {
                TokenType::NumberLiteral(num_str) => {
                    let value = num_str.parse::<usize>().map_err(|_| {
                        DatabaseError::parser_error(format!(
                            "Invalid number literal '{}': must be a positive integer",
                            num_str
                        ))
                    })?;
                    self.advance();
                    Ok(value)
                }
                _ => Err(DatabaseError::parser_error(format!(
                    "Expected number literal, found {:?}",
                    token.token_type
                ))),
            },
            None => Err(DatabaseError::parser_error(
                "Expected number literal, found EOF",
            )),
        }
    }

    // ============================================================================
    // UTILITY METHODS
    // ============================================================================

    /// Returns the current token without consuming it.
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Returns the type of the current token without consuming it.
    fn peek_token_type(&self) -> Option<TokenType> {
        self.peek_token().map(|t| t.token_type.clone())
    }

    /// Advances to the next token.
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    /// Checks if the current token matches the expected type,
    /// and consumes it if it does.
    fn match_token(&mut self, expected: TokenType) -> bool {
        if let Some(token) = self.peek_token() {
            if token.token_type == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Consumes the current token if it matches the expected type,
    /// otherwise returns an error.
    fn consume_token(&mut self, expected: TokenType, error_message: &str) -> Result<()> {
        if let Some(token) = self.peek_token() {
            if token.token_type == expected {
                self.advance();
                return Ok(());
            }

            return Err(DatabaseError::parser_error(format!(
                "{} at line {}, column {}",
                error_message, token.line, token.column
            )));
        }

        Err(DatabaseError::parser_error(format!(
            "{} (found EOF)",
            error_message
        )))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic SELECT * FROM table
    #[test]
    fn test_basic_select() {
        let mut parser = Parser::new("SELECT * FROM users");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 1);
                assert!(matches!(select_stmt.select_items[0], SelectItem::Wildcard));
                assert_eq!(select_stmt.from_table, "users");
                assert!(select_stmt.where_clause.is_none());
                assert!(select_stmt.group_by.is_none());
            }
        }
    }

    /// Test SELECT with specific columns
    #[test]
    fn test_select_columns() {
        let mut parser = Parser::new("SELECT name, age, city FROM users");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 3);

                if let SelectItem::Expression(Expression::Column(name)) =
                    &select_stmt.select_items[0]
                {
                    assert_eq!(name, "name");
                } else {
                    panic!("Expected column name");
                }

                if let SelectItem::Expression(Expression::Column(age)) =
                    &select_stmt.select_items[1]
                {
                    assert_eq!(age, "age");
                } else {
                    panic!("Expected column age");
                }

                if let SelectItem::Expression(Expression::Column(city)) =
                    &select_stmt.select_items[2]
                {
                    assert_eq!(city, "city");
                } else {
                    panic!("Expected column city");
                }

                assert_eq!(select_stmt.from_table, "users");
            }
        }
    }

    /// Test SELECT with WHERE clause
    #[test]
    fn test_select_where() {
        let mut parser = Parser::new("SELECT name FROM users WHERE age > 25");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert!(select_stmt.where_clause.is_some());

                if let Some(Expression::BinaryOp {
                    left,
                    operator,
                    right,
                }) = select_stmt.where_clause
                {
                    if let Expression::Column(col) = *left {
                        assert_eq!(col, "age");
                    } else {
                        panic!("Expected column");
                    }

                    assert_eq!(operator, BinaryOperator::Greater);

                    if let Expression::NumberLiteral(num) = *right {
                        assert_eq!(num, "25");
                    } else {
                        panic!("Expected number literal");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            }
        }
    }

    /// Test SELECT with GROUP BY clause
    #[test]
    fn test_select_group_by() {
        let mut parser = Parser::new("SELECT city, COUNT(*) FROM users GROUP BY city");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 2);
                assert!(select_stmt.group_by.is_some());

                let group_by = select_stmt.group_by.unwrap();
                assert_eq!(group_by.len(), 1);
                assert_eq!(group_by[0], "city");
            }
        }
    }

    /// Test aggregate functions
    #[test]
    fn test_aggregate_functions() {
        let mut parser =
            Parser::new("SELECT COUNT(*), SUM(age), AVG(salary), MIN(age), MAX(age) FROM users");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 5);

                // Check COUNT(*)
                if let SelectItem::Expression(Expression::AggregateFunction {
                    function,
                    argument,
                }) = &select_stmt.select_items[0]
                {
                    assert_eq!(function, "COUNT");
                    if let Expression::Column(c) = &**argument {
                        assert_eq!(c, "*");
                    } else {
                        panic!("Expected column *");
                    }
                } else {
                    panic!("Expected COUNT(*)");
                }

                // Check SUM(age)
                if let SelectItem::Expression(Expression::AggregateFunction {
                    function,
                    argument,
                }) = &select_stmt.select_items[1]
                {
                    assert_eq!(function, "SUM");
                    if let Expression::Column(col) = &**argument {
                        assert_eq!(col, "age");
                    } else {
                        panic!("Expected column");
                    }
                } else {
                    panic!("Expected SUM(age)");
                }

                // Check AVG(salary)
                if let SelectItem::Expression(Expression::AggregateFunction {
                    function,
                    argument,
                }) = &select_stmt.select_items[2]
                {
                    assert_eq!(function, "AVG");
                    if let Expression::Column(col) = &**argument {
                        assert_eq!(col, "salary");
                    } else {
                        panic!("Expected column");
                    }
                } else {
                    panic!("Expected AVG(salary)");
                }

                // Check MIN(age)
                if let SelectItem::Expression(Expression::AggregateFunction {
                    function,
                    argument,
                }) = &select_stmt.select_items[3]
                {
                    assert_eq!(function, "MIN");
                    if let Expression::Column(col) = &**argument {
                        assert_eq!(col, "age");
                    } else {
                        panic!("Expected column");
                    }
                } else {
                    panic!("Expected MIN(age)");
                }

                // Check MAX(age)
                if let SelectItem::Expression(Expression::AggregateFunction {
                    function,
                    argument,
                }) = &select_stmt.select_items[4]
                {
                    assert_eq!(function, "MAX");
                    if let Expression::Column(col) = &**argument {
                        assert_eq!(col, "age");
                    } else {
                        panic!("Expected column");
                    }
                } else {
                    panic!("Expected MAX(age)");
                }
            }
        }
    }

    /// Test complex WHERE clause with AND/OR
    #[test]
    fn test_complex_where() {
        let mut parser = Parser::new("SELECT name FROM users WHERE age > 25 AND city = 'New York'");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert!(select_stmt.where_clause.is_some());

                if let Some(Expression::BinaryOp {
                    left,
                    operator,
                    right,
                }) = select_stmt.where_clause
                {
                    assert_eq!(operator, BinaryOperator::And);

                    // Check left condition: age > 25
                    if let Expression::BinaryOp {
                        left: left_left,
                        operator: left_op,
                        right: left_right,
                    } = *left
                    {
                        assert_eq!(left_op, BinaryOperator::Greater);
                        assert!(matches!(*left_left, Expression::Column(c) if c == "age"));
                        assert!(matches!(*left_right, Expression::NumberLiteral(n) if n == "25"));
                    } else {
                        panic!("Expected left comparison");
                    }

                    // Check right condition: city = 'New York'
                    if let Expression::BinaryOp {
                        left: right_left,
                        operator: right_op,
                        right: right_right,
                    } = *right
                    {
                        assert_eq!(right_op, BinaryOperator::Equal);
                        assert!(matches!(*right_left, Expression::Column(c) if c == "city"));
                        if let Expression::StringLiteral(s) = *right_right {
                            assert_eq!(s, "New York");
                        } else {
                            panic!("Expected string literal");
                        }
                    } else {
                        panic!("Expected right comparison");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            }
        }
    }

    /// Test string literals in SELECT
    #[test]
    fn test_string_literals() {
        let mut parser = Parser::new("SELECT 'hello', 'world' FROM users");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 2);

                if let SelectItem::Expression(Expression::StringLiteral(s)) =
                    &select_stmt.select_items[0]
                {
                    assert_eq!(s, "hello");
                } else {
                    panic!("Expected string literal");
                }

                if let SelectItem::Expression(Expression::StringLiteral(s)) =
                    &select_stmt.select_items[1]
                {
                    assert_eq!(s, "world");
                } else {
                    panic!("Expected string literal");
                }
            }
        }
    }

    /// Test comparison operators
    #[test]
    fn test_comparison_operators() {
        // Test all comparison operators
        let tests = vec![
            ("=", BinaryOperator::Equal),
            ("!=", BinaryOperator::NotEqual),
            ("<", BinaryOperator::Less),
            (">", BinaryOperator::Greater),
            ("<=", BinaryOperator::LessEqual),
            (">=", BinaryOperator::GreaterEqual),
        ];

        for (op_str, expected_op) in tests {
            let sql = format!("SELECT name FROM users WHERE age {} 25", op_str);
            let mut parser = Parser::new(&sql);
            let result = parser.parse();

            assert!(result.is_ok(), "Failed for operator {}", op_str);

            let query = result.unwrap();
            let Query::Select(select_stmt) = query;
            if let Some(Expression::BinaryOp { operator, .. }) = select_stmt.where_clause {
                assert_eq!(operator, expected_op, "Operator mismatch for {}", op_str);
            } else {
                panic!("Expected binary operation for {}", op_str);
            }
        }
    }

    /// Test NOT operator
    #[test]
    fn test_not_operator() {
        let mut parser = Parser::new("SELECT name FROM users WHERE NOT (age < 18)");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert!(select_stmt.where_clause.is_some());

                if let Some(Expression::UnaryOp { operator, operand }) = select_stmt.where_clause {
                    assert_eq!(operator, UnaryOperator::Not);
                    if let Expression::BinaryOp {
                        operator: inner_op, ..
                    } = &*operand
                    {
                        assert_eq!(*inner_op, BinaryOperator::Less);
                    } else {
                        panic!("Expected binary operation inside NOT");
                    }
                } else {
                    panic!("Expected unary operation");
                }
            }
        }
    }

    /// Test error on invalid syntax
    #[test]
    fn test_invalid_syntax() {
        let mut parser = Parser::new("SELECT FROM users");
        let result = parser.parse();

        assert!(result.is_err());
    }

    /// Test error on missing FROM
    #[test]
    fn test_missing_from() {
        let mut parser = Parser::new("SELECT name users");
        let result = parser.parse();

        assert!(result.is_err());
    }

    /// Test tokenizer with whitespace
    #[test]
    fn test_tokenizer_whitespace() {
        let mut tokenizer = Tokenizer::new("SELECT  *   FROM   users");
        let tokens = tokenizer.tokenize();

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Select);
        assert_eq!(tokens[1].token_type, TokenType::Asterisk);
        assert_eq!(tokens[2].token_type, TokenType::From);
        assert!(matches!(tokens[3].token_type, TokenType::Identifier(_)));
    }

    /// Test tokenizer with numbers
    #[test]
    fn test_tokenizer_numbers() {
        let mut tokenizer = Tokenizer::new("123 45.67");
        let tokens = tokenizer.tokenize();

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();

        assert!(matches!(&tokens[0].token_type, TokenType::NumberLiteral(s) if s == "123"));
        assert!(matches!(&tokens[1].token_type, TokenType::NumberLiteral(s) if s == "45.67"));
    }

    /// Test tokenizer with strings
    #[test]
    fn test_tokenizer_strings() {
        let mut tokenizer = Tokenizer::new("'hello' 'world'");
        let tokens = tokenizer.tokenize();

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();

        assert!(matches!(&tokens[0].token_type, TokenType::StringLiteral(s) if s == "hello"));
        assert!(matches!(&tokens[1].token_type, TokenType::StringLiteral(s) if s == "world"));
    }

    /// Test tokenizer with operators
    #[test]
    fn test_tokenizer_operators() {
        let mut tokenizer = Tokenizer::new("= != < > <= >=");
        let tokens = tokenizer.tokenize();

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Equal);
        assert_eq!(tokens[1].token_type, TokenType::NotEqual);
        assert_eq!(tokens[2].token_type, TokenType::Less);
        assert_eq!(tokens[3].token_type, TokenType::Greater);
        assert_eq!(tokens[4].token_type, TokenType::LessEqual);
        assert_eq!(tokens[5].token_type, TokenType::GreaterEqual);
    }

    /// Test case insensitivity
    #[test]
    fn test_case_insensitivity() {
        let tests = vec![
            "SELECT * FROM users",
            "select * from users",
            "Select * From Users",
        ];

        for sql in tests {
            let mut parser = Parser::new(sql);
            let result = parser.parse();

            assert!(result.is_ok(), "Failed for SQL: {}", sql);

            let query = result.unwrap();
            let Query::Select(select_stmt) = query;
            assert_eq!(select_stmt.from_table, "users");
        }
    }

    /// Test complex query with all clauses
    #[test]
    fn test_complex_query() {
        let sql = "SELECT city, COUNT(*), AVG(age) FROM users WHERE age >= 18 AND status = 'active' GROUP BY city";
        let mut parser = Parser::new(sql);
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 3);
                assert_eq!(select_stmt.from_table, "users");
                assert!(select_stmt.where_clause.is_some());
                assert!(select_stmt.group_by.is_some());

                let group_by = select_stmt.group_by.unwrap();
                assert_eq!(group_by.len(), 1);
                assert_eq!(group_by[0], "city");
            }
        }
    }

    /// Test unary minus
    #[test]
    fn test_unary_minus() {
        let mut parser = Parser::new("SELECT age FROM users WHERE balance > -100");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert!(select_stmt.where_clause.is_some());

                if let Some(Expression::BinaryOp {
                    left: _,
                    operator: _,
                    right,
                }) = select_stmt.where_clause
                {
                    if let Expression::UnaryOp { operator, operand } = &*right {
                        assert_eq!(*operator, UnaryOperator::Minus);
                        if let Expression::NumberLiteral(num) = &**operand {
                            assert_eq!(num, "100");
                        } else {
                            panic!("Expected number literal");
                        }
                    } else {
                        panic!("Expected unary operation");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            }
        }
    }

    /// Test arithmetic operators
    #[test]
    fn test_arithmetic_operators() {
        let mut parser = Parser::new("SELECT age + 10 FROM users");
        let result = parser.parse();

        assert!(result.is_ok());
        let query = result.unwrap();

        match query {
            Query::Select(select_stmt) => {
                assert_eq!(select_stmt.select_items.len(), 1);

                if let SelectItem::Expression(Expression::BinaryOp {
                    left,
                    operator,
                    right,
                }) = &select_stmt.select_items[0]
                {
                    assert_eq!(*operator, BinaryOperator::Plus);
                    if let Expression::Column(c) = &**left {
                        assert_eq!(c, "age");
                    } else {
                        panic!("Expected column age");
                    }
                    if let Expression::NumberLiteral(n) = &**right {
                        assert_eq!(n, "10");
                    } else {
                        panic!("Expected number literal 10");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            }
        }
    }
}
