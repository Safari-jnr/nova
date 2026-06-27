// src/bytecode.rs - Bytecode instructions for Nova VM
// These are the low-level operations the VM understands

use crate::value::Value;
use serde::{Deserialize, Serialize};

/// Bytecode instruction
/// Think of these like CPU instructions, but for our VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Stack operations
    /// Push a constant value onto the stack
    LoadConstant(Value),
    
    /// Pop a value from the stack
    Pop,
    
    // Variable operations
    /// Store value in variable: stack.pop() -> variables[name]
    SetVariable(String),
    
    /// Load variable onto stack: variables[name] -> stack.push()
    GetVariable(String),
    
    // Arithmetic operations
    /// Pop two values, add them, push result
    Add,
    
    /// Pop two values, subtract them, push result
    Subtract,
    
    /// Pop two values, multiply them, push result
    Multiply,
    
    /// Pop two values, divide them, push result
    Divide,
    
    /// Pop two values, modulo them, push result
    Modulo,
    
    /// Pop one value, negate it, push result
    Negate,
    
    // Comparison operations
    /// Pop two values, compare equal, push boolean
    Equal,
    
    /// Pop two values, compare not equal, push boolean
    NotEqual,
    
    /// Pop two values, compare less than, push boolean
    Less,
    
    /// Pop two values, compare less or equal, push boolean
    LessEqual,
    
    /// Pop two values, compare greater than, push boolean
    Greater,
    
    /// Pop two values, compare greater or equal, push boolean
    GreaterEqual,
    
    // Logical operations
    /// Pop two values, logical AND, push result
    And,
    
    /// Pop two values, logical OR, push result
    Or,
    
    /// Pop one value, logical NOT, push result
    Not,
    
    // Control flow
    /// Jump to instruction at address
    Jump(usize),
    
    /// Pop value, if falsy jump to address
    JumpIfFalse(usize),
    
    /// Pop value, if truthy jump to address
    JumpIfTrue(usize),
    
    // Function operations
    /// Define a function (name, param_count, body_address)
    DefineFunction {
        name: String,
        param_count: usize,
        address: usize,
    },
    
    /// Call a function (pops function and args from stack)
    Call(usize), // arg_count
    
    /// Call a method on an object (binds 'this')
    /// Stack: [object, method, arg1, arg2, ...]
    CallMethod(usize), // arg_count
    
    /// Return from function (pops return value, restores previous frame)
    Return,
    
    // Array operations
    /// Create array from top N stack values
    BuildArray(usize),
    
    /// Build range array from start..end
    BuildRange(bool), // inclusive
    
    /// Get array element: array[index]
    GetIndex,
    
    /// Set array element: array[index] = value
    SetIndex,

    ArrayLength,
    
    // Object operations
    /// Create object from top N key-value pairs on stack
    BuildObject(usize),
    
    /// Get object property: object.property
    GetProperty(String),
    
    /// Set object property: object.property = value
    SetProperty(String),
    
    // Built-in functions
    /// Call built-in print function
    Print,
    
    /// Call built-in function
    CallBuiltin(String, usize), // (name, arg_count)
    
    // Special
    /// Halt execution
    Halt,
}

/// Compiled bytecode program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    /// The instructions
    pub instructions: Vec<Instruction>,
    
    /// Debug information (line numbers for each instruction)
    pub line_numbers: Vec<usize>,
}

impl Bytecode {
    pub fn new() -> Self {
        Bytecode {
            instructions: Vec::new(),
            line_numbers: Vec::new(),
        }
    }
    
    /// Add an instruction
    pub fn emit(&mut self, instruction: Instruction, line: usize) {
        self.instructions.push(instruction);
        self.line_numbers.push(line);
    }
    
    /// Get instruction at index
    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }
    
    /// Get current instruction count
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
    
    /// Patch a jump instruction with the correct address
    pub fn patch_jump(&mut self, index: usize, target: usize) {
        if let Some(instruction) = self.instructions.get_mut(index) {
            match instruction {
                Instruction::Jump(_) => {
                    *instruction = Instruction::Jump(target);
                }
                Instruction::JumpIfFalse(_) => {
                    *instruction = Instruction::JumpIfFalse(target);
                }
                Instruction::JumpIfTrue(_) => {
                    *instruction = Instruction::JumpIfTrue(target);
                }
                _ => {
                    // Not a jump instruction, ignore
                }
            }
        }
    }
    
    /// Disassemble for debugging
    pub fn disassemble(&self, name: &str) {
        println!("=== {} ===", name);
        for (i, instruction) in self.instructions.iter().enumerate() {
            let line = self.line_numbers.get(i).unwrap_or(&0);
            println!("{:04} (line {}) {:?}", i, line, instruction);
        }
        println!();
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}