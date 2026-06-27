// src/error.rs - Error types for Nova
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum NovaError {
    #[error("Syntax Error at line {line}, column {column}: {message}")]
    LexerError {
        message: String,
        line: usize,
        column: usize,
    },
    
    #[error("Parse Error at line {line}: {message}")]
    ParseError {
        message: String,
        line: usize,
    },
    
    #[error("Compile Error: {0}")]
    CompileError(String),
    
    #[error("Runtime Error: {0}")]
    RuntimeError(String),
    
    #[error("Type Error: Expected {expected}, got {got}")]
    TypeError {
        expected: String,
        got: String,
    },
    
    #[error("Variable '{0}' not found")]
    UndefinedVariable(String),
    
    #[error("Function '{0}' not found")]
    UndefinedFunction(String),
    
    #[error("IO Error: {0}")]
    IOError(String),
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Index {index} out of bounds (length: {length})")]
    IndexOutOfBounds {
        index: usize,
        length: usize,
    },
    
    #[error("Stack overflow: Maximum call depth exceeded")]
    StackOverflow,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}