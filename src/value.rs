// src/value.rs - Runtime value types for Nova VM

use std::fmt;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::NovaError;

/// Type for native Rust functions that Axon can call
pub type NativeFn = fn(Vec<Value>) -> Result<Value, NovaError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function {
        name: String,
        params: Vec<String>,
        address: usize,
    },
    #[serde(skip)]
    NativeFunction(NativeFn),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Function { address: a, .. }, Value::Function { address: b, .. }) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => *a as usize == *b as usize,
            _ => false,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Function { .. } | Value::NativeFunction(_) => true,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Function { name, params, .. } => {
                format!("fn {}({})", name, params.join(", "))
            }
            Value::NativeFunction(_) => "[native function]".to_string(),
        }
    }
    
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function { .. } | Value::NativeFunction(_) => "function",
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    pub fn as_boolean(&self) -> bool {
        self.is_truthy()
    }
    
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b.to_string()))),
            (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a.to_string(), b))),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::Array(result))
            }
            _ => Err(format!("Cannot add {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => Ok(Value::Number(a - b)),
            _ => Err(format!("Cannot subtract {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (Value::String(s), Value::Number(n)) | (Value::Number(n), Value::String(s)) => {
                if *n >= 0.0 && n.fract() == 0.0 {
                    Ok(Value::String(s.repeat(*n as usize)))
                } else {
                    Err("String repetition requires positive integer".to_string())
                }
            }
            _ => Err(format!("Cannot multiply {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => {
                if b == 0.0 { Err("Division by zero".to_string()) } 
                else { Ok(Value::Number(a / b)) }
            }
            _ => Err(format!("Cannot divide {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => {
                if b == 0.0 { Err("Modulo by zero".to_string()) } 
                else { Ok(Value::Number(a % b)) }
            }
            _ => Err(format!("Cannot modulo {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn equals(&self, other: &Value) -> bool { self == other }
    pub fn not_equals(&self, other: &Value) -> bool { self != other }
    
    pub fn less_than(&self, other: &Value) -> Result<bool, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => Ok(a < b),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn less_equal(&self, other: &Value) -> Result<bool, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => Ok(a <= b),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn greater_than(&self, other: &Value) -> Result<bool, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => Ok(a > b),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn greater_equal(&self, other: &Value) -> Result<bool, String> {
        match (self.as_number(), other.as_number()) {
            (Some(a), Some(b)) => Ok(a >= b),
            _ => Err(format!("Cannot compare {} and {}", self.type_name(), other.type_name())),
        }
    }
    
    pub fn logical_and(&self, other: &Value) -> Value {
        if self.is_truthy() { other.clone() } else { self.clone() }
    }
    
    pub fn logical_or(&self, other: &Value) -> Value {
        if self.is_truthy() { self.clone() } else { other.clone() }
    }
    
    pub fn logical_not(&self) -> Value {
        Value::Boolean(!self.is_truthy())
    }
    
    pub fn negate(&self) -> Result<Value, String> {
        match self.as_number() {
            Some(n) => Ok(Value::Number(-n)),
            None => Err(format!("Cannot negate {}", self.type_name())),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}