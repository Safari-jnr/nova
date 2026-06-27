// src/vm.rs - Virtual Machine for Nova
// This executes the bytecode instructions

use crate::bytecode::{Bytecode, Instruction};
use crate::error::NovaError;
use crate::stdlib;
use crate::value::Value;
use crate::compiler::ComponentDefinition;
use crate::ast::Expression;
use std::collections::HashMap;

/// Call frame for function calls
#[derive(Debug, Clone)]
struct CallFrame {
    return_address: usize,
    base_pointer: usize,
}

/// Virtual Machine
pub struct VM {
    /// The bytecode being executed
    bytecode: Option<Bytecode>,
    
    /// Instruction pointer
    ip: usize,
    
    /// Stack for values
    stack: Vec<Value>,
    
    /// Variables (global scope and local variables)
    variables: HashMap<String, Value>,
    
    /// Component definitions
    components: HashMap<String, ComponentDefinition>,
    
    /// Call stack for function calls
    call_stack: Vec<CallFrame>,
    
    /// Max stack size (prevent stack overflow)
    max_stack_size: usize,
}

impl VM {
    pub fn new() -> Self {
        let mut variables = HashMap::new();

        // --- REGISTER GLOBAL NATIVE FUNCTIONS ---
        // This makes "startWebSocket" available to your Axon scripts
        variables.insert(
            "startWebSocket".to_string(),
            Value::NativeFunction(crate::stdlib::builtin_start_websocket),
        );

        VM {
            bytecode: None,
            ip: 0,
            stack: Vec::new(),
            variables,
            components: HashMap::new(),
            call_stack: Vec::new(),
            max_stack_size: 10000,
        }
    }
    
    /// Helper: Evaluate a simple expression to a Value
    fn evaluate_literal_expr(&self, expr: &Expression) -> Result<Value, NovaError> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Null => Ok(Value::Null),
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_literal_expr(elem)?);
                }
                Ok(Value::Array(values))
            }
            Expression::Object(properties) => {
                let mut map = HashMap::new();
                for (key, value_expr) in properties {
                    let value = self.evaluate_literal_expr(value_expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }
            _ => Ok(Value::Null)
        }
    }
    
    /// Execute bytecode
    pub fn execute(&mut self, bytecode: Bytecode, components: HashMap<String, ComponentDefinition>) -> Result<(), NovaError> {
        self.bytecode = Some(bytecode);
        self.components = components;
        self.ip = 0;
        self.stack.clear();
        
        // IMPORTANT: We do NOT clear self.variables here anymore, 
        // otherwise our "startWebSocket" global would be deleted.
        // self.variables.clear(); 
        
        self.call_stack.clear();
        self.run()
    }
    
    /// Main execution loop
    fn run(&mut self) -> Result<(), NovaError> {
        loop {
            if self.stack.len() > self.max_stack_size {
                return Err(NovaError::StackOverflow);
            }
            
            let instruction: Instruction = match self.bytecode.as_ref().unwrap().get(self.ip) {
                Some(inst) => inst.clone(),
                None => return Err(NovaError::RuntimeError("Invalid instruction pointer".to_string())),
            };
            
            self.ip += 1;
            
            match instruction {
                Instruction::LoadConstant(value) => {
                    self.stack.push(value);
                }
                
                Instruction::Pop => {
                    self.stack.pop();
                }
                
                Instruction::SetVariable(name) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.variables.insert(name, value);
                }
                
                Instruction::GetVariable(name) => {
                    let value = self.variables.get(&name)
                        .ok_or_else(|| NovaError::UndefinedVariable(name.clone()))?
                        .clone();
                    self.stack.push(value);
                }
                
                Instruction::Add => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.add(&b).map_err(NovaError::RuntimeError)?);
                }
                
                Instruction::Subtract => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.subtract(&b).map_err(NovaError::RuntimeError)?);
                }
                
                Instruction::Multiply => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.multiply(&b).map_err(NovaError::RuntimeError)?);
                }
                
                Instruction::Divide => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.divide(&b).map_err(NovaError::RuntimeError)?);
                }
                
                Instruction::Modulo => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.modulo(&b).map_err(NovaError::RuntimeError)?);
                }
                
                Instruction::Negate => {
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.negate().map_err(NovaError::RuntimeError)?);
                }
                
                Instruction::Equal => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(Value::Boolean(a.equals(&b)));
                }
                
                Instruction::NotEqual => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(Value::Boolean(a.not_equals(&b)));
                }
                
                Instruction::Less => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(Value::Boolean(a.less_than(&b).map_err(NovaError::RuntimeError)?));
                }
                
                Instruction::LessEqual => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(Value::Boolean(a.less_equal(&b).map_err(NovaError::RuntimeError)?));
                }
                
                Instruction::Greater => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(Value::Boolean(a.greater_than(&b).map_err(NovaError::RuntimeError)?));
                }
                
                Instruction::GreaterEqual => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(Value::Boolean(a.greater_equal(&b).map_err(NovaError::RuntimeError)?));
                }
                
                Instruction::And => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.logical_and(&b));
                }
                
                Instruction::Or => {
                    let b = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.logical_or(&b));
                }
                
                Instruction::Not => {
                    let a = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    self.stack.push(a.logical_not());
                }
                
                Instruction::Jump(address) => { self.ip = address; }
                
                Instruction::JumpIfFalse(address) => {
                    let condition = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    if !condition.is_truthy() { self.ip = address; }
                }
                
                Instruction::JumpIfTrue(address) => {
                    let condition = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    if condition.is_truthy() { self.ip = address; }
                }
                
                Instruction::DefineFunction { name, address, param_count } => {
                    let func = Value::Function {
                        name: name.clone(),
                        params: vec!["param".to_string(); param_count],
                        address,
                    };
                    self.variables.insert(name, func);
                }
                
                Instruction::Call(arg_count) => {
                    let func = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    
                    match func {
                        Value::String(s) if s.starts_with("Component:") => {
                            let component_name = s.strip_prefix("Component:").unwrap();
                            if let Some(component_def) = self.components.get(component_name).cloned() {
                                let mut instance = HashMap::new();
                                if let Some(state_expr) = &component_def.state {
                                    instance.insert("state".to_string(), self.evaluate_literal_expr(state_expr)?);
                                } else {
                                    instance.insert("state".to_string(), Value::Object(HashMap::new()));
                                }
                                instance.insert("__component__".to_string(), Value::String(component_name.to_string()));
                                if let Some(render_addr) = component_def.render_address {
                                    instance.insert("render".to_string(), Value::Function {
                                        name: "render".to_string(),
                                        params: vec![],
                                        address: render_addr,
                                    });
                                }
                                self.stack.push(Value::Object(instance));
                            } else {
                                return Err(NovaError::RuntimeError(format!("Component '{}' not found", component_name)));
                            }
                        }
                        Value::Function { address, params, .. } => {
                            if arg_count != params.len() {
                                return Err(NovaError::RuntimeError(format!("Expected {} args, got {}", params.len(), arg_count)));
                            }
                            self.call_stack.push(CallFrame { return_address: self.ip, base_pointer: self.stack.len() });
                            self.ip = address;
                        }
                        // Handle Native Functions (like startWebSocket)
                        Value::NativeFunction(func_ptr) => {
                            let mut args = Vec::new();
                            for _ in 0..arg_count {
                                args.insert(0, self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?);
                            }
                            let result = func_ptr(args)?;
                            self.stack.push(result);
                        }
                        _ => return Err(NovaError::TypeError { expected: "callable".into(), got: func.type_name().into() }),
                    }
                }
                
                Instruction::Return => {
                    let return_value = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.stack.truncate(frame.base_pointer);
                        self.stack.push(return_value);
                        self.ip = frame.return_address;
                    }
                }

                Instruction::BuildArray(size) => {
                    let mut elements = Vec::new();
                    for _ in 0..size {
                        elements.insert(0, self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?);
                    }
                    self.stack.push(Value::Array(elements));
                }

                Instruction::BuildObject(size) => {
                    let mut map = HashMap::new();
                    for _ in 0..size {
                        let val = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                        let key = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                        if let Value::String(k) = key { map.insert(k, val); }
                        else { return Err(NovaError::RuntimeError("Keys must be strings".into())); }
                    }
                    self.stack.push(Value::Object(map));
                }

                Instruction::Print => {
                    let value = self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?;
                    println!("{}", value);
                    self.stack.push(Value::Null);
                }
                
                Instruction::CallBuiltin(name, arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..arg_count {
                        args.insert(0, self.stack.pop().ok_or_else(|| NovaError::RuntimeError("Stack underflow".to_string()))?);
                    }
                    self.stack.push(stdlib::call_builtin(&name, args)?);
                }
                
                Instruction::Halt => break,
                _ => return Err(NovaError::RuntimeError(format!("Unimplemented: {:?}", instruction))),
            }
        }
        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self { Self::new() }
}