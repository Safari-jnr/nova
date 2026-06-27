// src/lexer.rs - Lexer for Nova
use crate::error::{NovaError, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Let, Const, Fn, If, Else, For, In, While, Return, Match,
    Try, Catch, Finally, Import, Export, From, As,
    Component, Page, State, Computed, Methods, Props,
    OnMount, OnUnmount, Watch, View, Database, Table,
    Guard, Async, Await, This,
    
    // Literals
    True, False, Null,
    Identifier(String),
    Number(f64),
    String(String),
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, NotEqual,
    Greater, GreaterEqual, Less, LessEqual,
    And, Or, Not,
    AndAnd, OrOr, Bang,  // && || !
    PlusEqual, MinusEqual, StarEqual, SlashEqual,
    PlusPlus, MinusMinus,
    
    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Comma, Dot, Colon,
    Semicolon, Arrow, Question, DotDot, DotDotEqual,
    
    Newline, Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Position,
}

impl Token {
    fn new(token_type: TokenType, position: Position) -> Self {
        Token { token_type, position }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    fn current(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }
    
    fn pos(&self) -> Position {
        Position::new(self.line, self.column)
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        if self.current() == Some('/') && self.peek() == Some('/') {
            while self.current().is_some() && self.current() != Some('\n') {
                self.advance();
            }
        }
    }
    
    fn read_number(&mut self) -> Result<TokenType, NovaError> {
    let pos = self.pos();
    let mut num_str = String::new();
    let mut has_dot = false;
    
    while let Some(ch) = self.current() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            self.advance();
        } else if ch == '.' && !has_dot {
            // Check if next character is also a dot (range operator ..)
            if self.peek() == Some('.') {
                // Don't consume the dots, they're a range operator
                break;
            }
            has_dot = true;
            num_str.push(ch);
            self.advance();
        } else {
            break;
        }
    }
    
    num_str.parse::<f64>()
        .map(TokenType::Number)
        .map_err(|_| NovaError::LexerError {
            message: format!("Invalid number: {}", num_str),
            line: pos.line,
            column: pos.column,
        })
}
    
    fn read_string(&mut self, quote: char) -> Result<TokenType, NovaError> {
        let pos = self.pos();
        let mut string = String::new();
        self.advance();
        
        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance();
                return Ok(TokenType::String(string));
            } else if ch == '\\' {
                self.advance();
                match self.current() {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some('\'') => string.push('\''),
                    Some(c) => string.push(c),
                    None => break,
                }
                self.advance();
            } else {
                string.push(ch);
                self.advance();
            }
        }
        
        Err(NovaError::LexerError {
            message: "Unterminated string".to_string(),
            line: pos.line,
            column: pos.column,
        })
    }
    
    fn read_identifier(&mut self) -> TokenType {
        let mut ident = String::new();
        
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        match ident.as_str() {
            "let" => TokenType::Let,
            "const" => TokenType::Const,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "while" => TokenType::While,
            "return" => TokenType::Return,
            "match" => TokenType::Match,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "finally" => TokenType::Finally,
            "import" => TokenType::Import,
            "export" => TokenType::Export,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "component" => TokenType::Component,
            "page" => TokenType::Page,
            "state" => TokenType::State,
            "computed" => TokenType::Computed,
            "methods" => TokenType::Methods,
            "props" => TokenType::Props,
            "onMount" => TokenType::OnMount,
            "onUnmount" => TokenType::OnUnmount,
            "watch" => TokenType::Watch,
            "view" => TokenType::View,
            "database" => TokenType::Database,
            "table" => TokenType::Table,
            "guard" => TokenType::Guard,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Identifier(ident),
        }
    }
    
    fn next_token(&mut self) -> Result<Token, NovaError> {
        self.skip_whitespace();
        self.skip_comment();
        
        let pos = self.pos();
        
        match self.current() {
            None => Ok(Token::new(TokenType::Eof, pos)),
            
            Some('\n') => {
                self.advance();
                Ok(Token::new(TokenType::Newline, pos))
            }
            
            Some(ch) if ch.is_ascii_digit() => {
                let token_type = self.read_number()?;
                Ok(Token::new(token_type, pos))
            }
            
            Some('"') | Some('\'') => {
                let quote = self.current().unwrap();
                let token_type = self.read_string(quote)?;
                Ok(Token::new(token_type, pos))
            }
            
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let token_type = self.read_identifier();
                Ok(Token::new(token_type, pos))
            }
            
            Some('+') => {
                self.advance();
                if self.current() == Some('+') {
                    self.advance();
                    Ok(Token::new(TokenType::PlusPlus, pos))
                } else if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::PlusEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Plus, pos))
                }
            }
            
            Some('-') => {
                self.advance();
                if self.current() == Some('-') {
                    self.advance();
                    Ok(Token::new(TokenType::MinusMinus, pos))
                } else if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::MinusEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Minus, pos))
                }
            }
            
            Some('*') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::StarEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Star, pos))
                }
            }
            
            Some('/') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::SlashEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Slash, pos))
                }
            }
            
            Some('%') => {
                self.advance();
                Ok(Token::new(TokenType::Percent, pos))
            }
            
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::EqualEqual, pos))
                } else if self.current() == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenType::Arrow, pos))
                } else {
                    Ok(Token::new(TokenType::Equal, pos))
                }
            }
            
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::NotEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Bang, pos))
                }
            }
            
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::GreaterEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Greater, pos))
                }
            }
            
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::LessEqual, pos))
                } else {
                    Ok(Token::new(TokenType::Less, pos))
                }
            }
            
            Some('.') => {
                self.advance();
                if self.current() == Some('.') {
                    self.advance();
                    // Check for ..= (inclusive range)
                    if self.current() == Some('=') {
                        self.advance();
                        Ok(Token::new(TokenType::DotDotEqual, pos))
                    } else {
                        Ok(Token::new(TokenType::DotDot, pos))
                    }
                } else {
                    Ok(Token::new(TokenType::Dot, pos))
                }
            }
            
            Some('(') => {
                self.advance();
                Ok(Token::new(TokenType::LeftParen, pos))
            }
            
            Some(')') => {
                self.advance();
                Ok(Token::new(TokenType::RightParen, pos))
            }
            
            Some('{') => {
                self.advance();
                Ok(Token::new(TokenType::LeftBrace, pos))
            }
            
            Some('}') => {
                self.advance();
                Ok(Token::new(TokenType::RightBrace, pos))
            }
            
            Some('[') => {
                self.advance();
                Ok(Token::new(TokenType::LeftBracket, pos))
            }
            
            Some(']') => {
                self.advance();
                Ok(Token::new(TokenType::RightBracket, pos))
            }
            
            Some(',') => {
                self.advance();
                Ok(Token::new(TokenType::Comma, pos))
            }
            
            Some(':') => {
                self.advance();
                Ok(Token::new(TokenType::Colon, pos))
            }
            
            Some(';') => {
                self.advance();
                Ok(Token::new(TokenType::Semicolon, pos))
            }
            
            Some('?') => {
                self.advance();
                Ok(Token::new(TokenType::Question, pos))
            }
            
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    Ok(Token::new(TokenType::AndAnd, pos))
                } else {
                    Err(NovaError::LexerError {
                        message: "Unexpected character '&'. Did you mean '&&'?".to_string(),
                        line: pos.line,
                        column: pos.column,
                    })
                }
            }
            
            Some('|') => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    Ok(Token::new(TokenType::OrOr, pos))
                } else {
                    Err(NovaError::LexerError {
                        message: "Unexpected character '|'. Did you mean '||'?".to_string(),
                        line: pos.line,
                        column: pos.column,
                    })
                }
            }
            
            Some(ch) => {
                Err(NovaError::LexerError {
                    message: format!("Unexpected character '{}'", ch),
                    line: pos.line,
                    column: pos.column,
                })
            }
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, NovaError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token()?;
        let is_eof = matches!(token.token_type, TokenType::Eof);
        
        if !matches!(token.token_type, TokenType::Newline) {
            tokens.push(token);
        }
        
        if is_eof {
            break;
        }
    }
    
    Ok(tokens)
}