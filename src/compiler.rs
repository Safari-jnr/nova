// src/compiler.rs - Nova Compiler

#[allow(unused_imports)]
use crate::ast::*;
use crate::bytecode::{Bytecode, Instruction};
use crate::error::NovaError;
use crate::value::Value;
use std::collections::HashMap;

pub struct Compiler {
    bytecode: Bytecode,
    #[allow(dead_code)]
    functions: HashMap<String, usize>,
    components: HashMap<String, ComponentDefinition>,
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub name: String,
    pub state: Option<Expression>,
    pub methods: Option<Expression>,
    pub computed: Option<Expression>,
    pub props: Option<Expression>,
    pub render: Statement,
    pub render_address: Option<usize>,  // Bytecode address of render function
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            bytecode: Bytecode::new(),
            functions: HashMap::new(),
            components: HashMap::new(),
        }
    }
    
    fn compile_program(&mut self, program: &Program) -> Result<(), NovaError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        
        self.bytecode.emit(Instruction::Halt, 0);
        Ok(())
    }
    
    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), NovaError> {
        match stmt {
            Statement::Let { name, value, position } => {
                self.compile_expression(value)?;
                self.bytecode.emit(Instruction::SetVariable(name.clone()), position.line);
            }
            
            Statement::Const { name, value, position } => {
                self.compile_expression(value)?;
                self.bytecode.emit(Instruction::SetVariable(name.clone()), position.line);
            }
            
            Statement::Function { name, params, body, position, .. } => {
                let func_body_start = self.bytecode.len() + 2;
                
                self.bytecode.emit(
                    Instruction::LoadConstant(Value::Function {
                        name: name.clone(),
                        params: params.clone(),
                        address: func_body_start + 1,
                    }),
                    position.line,
                );
                self.bytecode.emit(Instruction::SetVariable(name.clone()), position.line);
                
                let jump_idx = self.bytecode.len();
                self.bytecode.emit(Instruction::Jump(0), position.line);
                
                let actual_body_start = self.bytecode.len();
                
                if let Instruction::LoadConstant(Value::Function { ref mut address, .. }) = 
                    &mut self.bytecode.instructions[func_body_start - 2] {
                    *address = actual_body_start;
                }
                
                for param in params.iter().rev() {
                    let param_name: String = param.clone();
                    self.bytecode.emit(Instruction::SetVariable(param_name), position.line);
                }
                
                let mut has_return = false;
                for stmt in body {
                    if matches!(stmt, Statement::Return { .. }) {
                        has_return = true;
                    }
                    self.compile_statement(stmt)?;
                }
                
                if !has_return {
                    self.bytecode.emit(Instruction::LoadConstant(Value::Null), position.line);
                    self.bytecode.emit(Instruction::Return, position.line);
                }
                
                let after_func = self.bytecode.len();
                self.bytecode.instructions[jump_idx] = Instruction::Jump(after_func);
            }
            
            Statement::Return { value, position } => {
                if let Some(val) = value {
                    self.compile_expression(val)?;
                } else {
                    self.bytecode.emit(Instruction::LoadConstant(Value::Null), position.line);
                }
                self.bytecode.emit(Instruction::Return, position.line);
            }
            
            Statement::If { condition, then_branch, else_branch, position } => {
                self.compile_expression(condition)?;
                
                let jump_to_else = self.bytecode.len();
                self.bytecode.emit(Instruction::JumpIfFalse(0), position.line);
                
                for stmt in then_branch {
                    self.compile_statement(stmt)?;
                }
                
                let jump_to_end = self.bytecode.len();
                self.bytecode.emit(Instruction::Jump(0), position.line);
                
                let else_start = self.bytecode.len();
                self.bytecode.instructions[jump_to_else] = Instruction::JumpIfFalse(else_start);
                
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.compile_statement(stmt)?;
                    }
                }
                
                let after_if = self.bytecode.len();
                self.bytecode.instructions[jump_to_end] = Instruction::Jump(after_if);
            }
            
            Statement::While { condition, body, position } => {
                let loop_start = self.bytecode.len();
                
                self.compile_expression(condition)?;
                
                let jump_to_end = self.bytecode.len();
                self.bytecode.emit(Instruction::JumpIfFalse(0), position.line);
                
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                self.bytecode.emit(Instruction::Jump(loop_start), position.line);
                
                let after_loop = self.bytecode.len();
                self.bytecode.instructions[jump_to_end] = Instruction::JumpIfFalse(after_loop);
            }
            
            Statement::For { variable, iterable, body, position } => {
                self.compile_expression(iterable)?;
                
                let array_var = format!("__iter_array_{}", position.line);
                self.bytecode.emit(Instruction::SetVariable(array_var.clone()), position.line);
                
                let counter_var = format!("__iter_i_{}", position.line);
                self.bytecode.emit(Instruction::LoadConstant(Value::Number(0.0)), position.line);
                self.bytecode.emit(Instruction::SetVariable(counter_var.clone()), position.line);
                
                let loop_start = self.bytecode.len();
                
                self.bytecode.emit(Instruction::GetVariable(counter_var.clone()), position.line);
                self.bytecode.emit(Instruction::GetVariable(array_var.clone()), position.line);
                self.bytecode.emit(Instruction::ArrayLength, position.line);
                self.bytecode.emit(Instruction::Less, position.line);
                
                let jump_to_end = self.bytecode.len();
                self.bytecode.emit(Instruction::JumpIfFalse(0), position.line);
                
                self.bytecode.emit(Instruction::GetVariable(array_var.clone()), position.line);
                self.bytecode.emit(Instruction::GetVariable(counter_var.clone()), position.line);
                self.bytecode.emit(Instruction::GetIndex, position.line);
                self.bytecode.emit(Instruction::SetVariable(variable.clone()), position.line);
                
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                self.bytecode.emit(Instruction::GetVariable(counter_var.clone()), position.line);
                self.bytecode.emit(Instruction::LoadConstant(Value::Number(1.0)), position.line);
                self.bytecode.emit(Instruction::Add, position.line);
                self.bytecode.emit(Instruction::SetVariable(counter_var.clone()), position.line);
                
                self.bytecode.emit(Instruction::Jump(loop_start), position.line);
                
                let after_loop = self.bytecode.len();
                self.bytecode.instructions[jump_to_end] = Instruction::JumpIfFalse(after_loop);
            }
            
            Statement::Expression { expr, position } => {
                self.compile_expression(expr)?;
                self.bytecode.emit(Instruction::Pop, position.line);
            }
            
            Statement::Component { name, state, methods, computed, props, render, position, .. } => {
                println!("Registered component: {}", name);
                
                // Compile the render function
                let render_address = if let Statement::Function { name: _fname, params: _, body, position: fpos, .. } = render.as_ref() {
                    // Compile render function
                    let _func_start = self.bytecode.len();
                    
                    // Jump over the function body (will be filled later)
                    let jump_idx = self.bytecode.len();
                    self.bytecode.emit(Instruction::Jump(0), fpos.line);
                    
                    let actual_body_start = self.bytecode.len();
                    
                    // Compile function body
                    for stmt in body {
                        self.compile_statement(stmt)?;
                    }
                    
                    // Add return if no explicit return
                    self.bytecode.emit(Instruction::LoadConstant(Value::Null), fpos.line);
                    self.bytecode.emit(Instruction::Return, fpos.line);
                    
                    // Patch the jump
                    let after_func = self.bytecode.len();
                    self.bytecode.instructions[jump_idx] = Instruction::Jump(after_func);
                    
                    Some(actual_body_start)
                } else {
                    None
                };
                
                // Store component definition in registry
                let component_def = ComponentDefinition {
                    name: name.clone(),
                    state: state.clone(),
                    methods: methods.clone(),
                    computed: computed.clone(),
                    props: props.clone(),
                    render: render.as_ref().clone(),
                    render_address,
                };
                
                self.components.insert(name.clone(), component_def);
                
                // Print info
                if state.is_some() {
                    println!("  - Has state");
                }
                
                if methods.is_some() {
                    println!("  - Has methods");
                }
                
                if render_address.is_some() {
                    println!("  - Has render function");
                }
                
                // Store component as a "constructor" in variables
                self.bytecode.emit(
                    Instruction::LoadConstant(Value::String(format!("Component:{}", name))),
                    position.line
                );
                self.bytecode.emit(Instruction::SetVariable(name.clone()), position.line);
            }
            
            _ => {
                return Err(NovaError::CompileError(
                    "Unsupported statement type".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn compile_expression(&mut self, expr: &Expression) -> Result<(), NovaError> {
        match expr {
            Expression::Number(n) => {
                self.bytecode.emit(Instruction::LoadConstant(Value::Number(*n)), 0);
            }
            
            Expression::String(s) => {
                if s.contains("${") {
                    self.compile_template_string(s)?;
                } else {
                    self.bytecode.emit(Instruction::LoadConstant(Value::String(s.clone())), 0);
                }
            }
            
            Expression::Boolean(b) => {
                self.bytecode.emit(Instruction::LoadConstant(Value::Boolean(*b)), 0);
            }
            
            Expression::Null => {
                self.bytecode.emit(Instruction::LoadConstant(Value::Null), 0);
            }
            
            Expression::This => {
                // For now, 'this' loads a special variable
                // In Phase 2.3, this will reference the component instance
                self.bytecode.emit(Instruction::GetVariable("__this__".to_string()), 0);
            }
            
            Expression::Identifier(name) => {
                self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
            }
            
            Expression::Array(elements) => {
                for elem in elements {
                    self.compile_expression(elem)?;
                }
                self.bytecode.emit(Instruction::BuildArray(elements.len()), 0);
            }
            
            Expression::Object(properties) => {
                for (key, value) in properties {
                    // Push key as string
                    let key_string: String = key.clone();
                    self.bytecode.emit(Instruction::LoadConstant(Value::String(key_string)), 0);
                    // Push value
                    self.compile_expression(value)?;
                }
                self.bytecode.emit(Instruction::BuildObject(properties.len()), 0);
            }
            
            Expression::Binary { left, operator, right } => {
                // Handle all assignment operators
                if matches!(operator, BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign | BinaryOp::DivAssign) {
                    if let Expression::Identifier(name) = left.as_ref() {
                        match operator {
                            BinaryOp::Assign => {
                                self.compile_expression(right)?;
                                self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                            }
                            BinaryOp::AddAssign => {
                                // Load current value: x += 5 becomes x = x + 5
                                self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                                self.compile_expression(right)?;
                                self.bytecode.emit(Instruction::Add, 0);
                                self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                            }
                            BinaryOp::SubAssign => {
                                self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                                self.compile_expression(right)?;
                                self.bytecode.emit(Instruction::Subtract, 0);
                                self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                            }
                            BinaryOp::MulAssign => {
                                self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                                self.compile_expression(right)?;
                                self.bytecode.emit(Instruction::Multiply, 0);
                                self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                            }
                            BinaryOp::DivAssign => {
                                self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                                self.compile_expression(right)?;
                                self.bytecode.emit(Instruction::Divide, 0);
                                self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                            }
                            _ => unreachable!(),
                        }
                    }
                } else {
                    self.compile_expression(left)?;
                    self.compile_expression(right)?;
                    
                    match operator {
                        BinaryOp::Add => self.bytecode.emit(Instruction::Add, 0),
                        BinaryOp::Subtract => self.bytecode.emit(Instruction::Subtract, 0),
                        BinaryOp::Multiply => self.bytecode.emit(Instruction::Multiply, 0),
                        BinaryOp::Divide => self.bytecode.emit(Instruction::Divide, 0),
                        BinaryOp::Modulo => self.bytecode.emit(Instruction::Modulo, 0),
                        BinaryOp::Equal => self.bytecode.emit(Instruction::Equal, 0),
                        BinaryOp::NotEqual => self.bytecode.emit(Instruction::NotEqual, 0),
                        BinaryOp::Less => self.bytecode.emit(Instruction::Less, 0),
                        BinaryOp::LessEq => self.bytecode.emit(Instruction::LessEqual, 0),
                        BinaryOp::Greater => self.bytecode.emit(Instruction::Greater, 0),
                        BinaryOp::GreaterEq => self.bytecode.emit(Instruction::GreaterEqual, 0),
                        BinaryOp::And => self.bytecode.emit(Instruction::And, 0),
                        BinaryOp::Or => self.bytecode.emit(Instruction::Or, 0),
                        _ => {
                            return Err(NovaError::CompileError(
                                format!("Unsupported operator: {:?}", operator)
                            ));
                        }
                    }
                }
            }
            
            Expression::Unary { operator, operand } => {
                match operator {
                    UnaryOp::Increment | UnaryOp::Decrement => {
                        // Prefix: ++x or --x
                        // Must be a variable
                        if let Expression::Identifier(name) = operand.as_ref() {
                            // Load current value
                            self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                            // Add/subtract 1
                            self.bytecode.emit(Instruction::LoadConstant(Value::Number(1.0)), 0);
                            if matches!(operator, UnaryOp::Increment) {
                                self.bytecode.emit(Instruction::Add, 0);
                            } else {
                                self.bytecode.emit(Instruction::Subtract, 0);
                            }
                            // Duplicate the new value (one for store, one for result)
                            // We'll need a Dup instruction for this
                            // For now, let's compute it twice
                            self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                            // Load the new value again for the result
                            self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                        } else {
                            return Err(NovaError::CompileError(
                                "Increment/decrement only works on variables".to_string()
                            ));
                        }
                    }
                    _ => {
                        self.compile_expression(operand)?;
                        
                        match operator {
                            UnaryOp::Negate => self.bytecode.emit(Instruction::Negate, 0),
                            UnaryOp::Not => self.bytecode.emit(Instruction::Not, 0),
                            _ => {
                                return Err(NovaError::CompileError(
                                    format!("Unsupported unary operator: {:?}", operator)
                                ));
                            }
                        }
                    }
                }
            }
            
            Expression::Postfix { operand, operator } => {
                // Postfix: x++ or x--
                // Must be a variable
                if let Expression::Identifier(name) = operand.as_ref() {
                    // Load current value (this will be the result)
                    self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                    // Load again for modification
                    self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                    // Add/subtract 1
                    self.bytecode.emit(Instruction::LoadConstant(Value::Number(1.0)), 0);
                    if matches!(operator, UnaryOp::Increment) {
                        self.bytecode.emit(Instruction::Add, 0);
                    } else {
                        self.bytecode.emit(Instruction::Subtract, 0);
                    }
                    // Store new value (pops it)
                    self.bytecode.emit(Instruction::SetVariable(name.clone()), 0);
                    // Original value is still on stack as result
                } else {
                    return Err(NovaError::CompileError(
                        "Increment/decrement only works on variables".to_string()
                    ));
                }
            }
            
            Expression::Call { function, args } => {
                if let Expression::Identifier(name) = function.as_ref() {
                    match name.as_str() {
                        "print" => {
                            for arg in args {
                                self.compile_expression(arg)?;
                                self.bytecode.emit(Instruction::Print, 0);
                                self.bytecode.emit(Instruction::Pop, 0);
                            }
                            self.bytecode.emit(Instruction::LoadConstant(Value::Null), 0);
                            return Ok(());
                        }
                        "len" | "length" => {
                            if args.len() != 1 {
                                return Err(NovaError::CompileError("len() takes 1 argument".to_string()));
                            }
                            self.compile_expression(&args[0])?;
                            self.bytecode.emit(Instruction::ArrayLength, 0);
                            return Ok(());
                        }
                        _ => {
                            // Check if it's a component first
                            if self.components.contains_key(name) {
                                // It's a component - compile as regular function call
                                // This will load the "Component:Name" string and Call will handle it
                                for arg in args {
                                    self.compile_expression(arg)?;
                                }
                                self.bytecode.emit(Instruction::GetVariable(name.clone()), 0);
                                self.bytecode.emit(Instruction::Call(args.len()), 0);
                                return Ok(());
                            }
                            
                            // Otherwise, it's a builtin function
                            for arg in args {
                                self.compile_expression(arg)?;
                            }
                            self.bytecode.emit(Instruction::CallBuiltin(name.clone(), args.len()), 0);
                            return Ok(());
                        }
                    }
                }
                
                // Check if this is a method call: obj.method()
                if let Expression::Member { object, property, computed } = function.as_ref() {
                    // It's a method call!
                    // We need: [args..., object, method]
                    
                    // Compile arguments first
                    for arg in args {
                        self.compile_expression(arg)?;
                    }
                    
                    // Compile the object TWICE: once for 'this', once to get the method
                    self.compile_expression(object)?;  // For 'this'
                    self.compile_expression(object)?;  // To get the method from
                    
                    // Get the method from the second object
                    if *computed {
                        self.compile_expression(property)?;
                        self.bytecode.emit(Instruction::GetIndex, 0);
                    } else {
                        if let Expression::Identifier(prop) = property.as_ref() {
                            self.bytecode.emit(Instruction::GetProperty(prop.clone()), 0);
                        }
                    }
                    
                    // Now stack is: [args..., object, method]
                    // Call as method (this will bind 'this')
                    self.bytecode.emit(Instruction::CallMethod(args.len()), 0);
                } else {
                    // Regular function call
                    for arg in args {
                        self.compile_expression(arg)?;
                    }
                    
                    self.compile_expression(function)?;
                    self.bytecode.emit(Instruction::Call(args.len()), 0);
                }
            }
            
            Expression::Member { object, property, computed } => {
                self.compile_expression(object)?;
                
                if *computed {
                    self.compile_expression(property)?;
                    self.bytecode.emit(Instruction::GetIndex, 0);
                } else {
                    if let Expression::Identifier(prop) = property.as_ref() {
                        self.bytecode.emit(Instruction::GetProperty(prop.clone()), 0);
                    }
                }
            }
            
            Expression::Range { start, end, inclusive } => {
                self.compile_expression(start)?;
                self.compile_expression(end)?;
                self.bytecode.emit(Instruction::BuildRange(*inclusive), 0);
            }
            
            Expression::Ternary { condition, then_val, else_val } => {
                // Compile condition
                self.compile_expression(condition)?;
                
                // Jump to else branch if false
                let else_jump = self.bytecode.len();
                self.bytecode.emit(Instruction::JumpIfFalse(0), 0); // Placeholder
                
                // Compile then branch
                self.compile_expression(then_val)?;
                
                // Jump over else branch
                let end_jump = self.bytecode.len();
                self.bytecode.emit(Instruction::Jump(0), 0); // Placeholder
                
                // Patch else jump
                let else_addr = self.bytecode.len();
                self.bytecode.patch_jump(else_jump, else_addr);
                
                // Compile else branch
                self.compile_expression(else_val)?;
                
                // Patch end jump
                let end_addr = self.bytecode.len();
                self.bytecode.patch_jump(end_jump, end_addr);
            }
            
            Expression::View { elements } => {
                // Compile view block to a UI tree representation
                // For now, we'll create a simple object representing the UI
                self.compile_ui_tree(elements)?;
            }
            
            _ => {
                return Err(NovaError::CompileError(
                    "Unsupported expression type".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    fn compile_ui_tree(&mut self, elements: &[UIElement]) -> Result<(), NovaError> {
        // For now, compile UI elements as a simple array of objects
        // Each object has: { type: "ComponentName", props: {...}, children: [...] }
        
        for element in elements {
            self.compile_ui_element(element)?;
        }
        
        // Build array from all elements
        self.bytecode.emit(Instruction::BuildArray(elements.len()), 0);
        
        Ok(())
    }
    
    fn compile_ui_element(&mut self, element: &UIElement) -> Result<(), NovaError> {
        // Create object for this UI element
        // { type: "tag", props: {...}, children: [...] }
        
        let mut obj_props = Vec::new();
        
        // Add type property
        self.bytecode.emit(Instruction::LoadConstant(Value::String("type".to_string())), 0);
        self.bytecode.emit(Instruction::LoadConstant(Value::String(element.tag.clone())), 0);
        obj_props.push(());
        
        // Add props object
        self.bytecode.emit(Instruction::LoadConstant(Value::String("props".to_string())), 0);
        
        // Compile props as object
        for (prop_name, prop_value) in &element.props {
            self.bytecode.emit(Instruction::LoadConstant(Value::String(prop_name.clone())), 0);
            self.compile_expression(prop_value)?;
        }
        self.bytecode.emit(Instruction::BuildObject(element.props.len()), 0);
        obj_props.push(());
        
        // Add children array
        self.bytecode.emit(Instruction::LoadConstant(Value::String("children".to_string())), 0);
        for child in &element.children {
            self.compile_ui_element(child)?;
        }
        self.bytecode.emit(Instruction::BuildArray(element.children.len()), 0);
        obj_props.push(());
        
        // Build the element object
        self.bytecode.emit(Instruction::BuildObject(3), 0); // type, props, children
        
        Ok(())
    }
    
    fn compile_template_string(&mut self, template: &str) -> Result<(), NovaError> {
        let mut result = String::new();
        let mut chars = template.chars().peekable();
        let mut parts = Vec::new();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                if !result.is_empty() {
                    self.bytecode.emit(Instruction::LoadConstant(Value::String(result.clone())), 0);
                    parts.push(true);
                    result.clear();
                }
                
                chars.next();
                let mut var_name = String::new();
                let mut brace_count = 1;
                
                while brace_count > 0 {
                    match chars.next() {
                        Some('}') => {
                            brace_count -= 1;
                            if brace_count > 0 {
                                var_name.push('}');
                            }
                        }
                        Some('{') => {
                            brace_count += 1;
                            var_name.push('{');
                        }
                        Some(c) => var_name.push(c),
                        None => break,
                    }
                }
                
                self.bytecode.emit(Instruction::GetVariable(var_name.trim().to_string()), 0);
                self.bytecode.emit(Instruction::CallBuiltin("str".to_string(), 1), 0);
                parts.push(true);
            } else {
                result.push(ch);
            }
        }
        
        if !result.is_empty() {
            self.bytecode.emit(Instruction::LoadConstant(Value::String(result)), 0);
            parts.push(true);
        }
        
        if parts.is_empty() {
            self.bytecode.emit(Instruction::LoadConstant(Value::String(String::new())), 0);
        } else if parts.len() > 1 {
            for _ in 0..parts.len() - 1 {
                self.bytecode.emit(Instruction::Add, 0);
            }
        }
        
        Ok(())
    }
}

pub struct CompiledProgram {
    pub bytecode: Bytecode,
    pub components: HashMap<String, ComponentDefinition>,
}

pub fn compile(program: Program) -> Result<CompiledProgram, NovaError> {
    let mut compiler = Compiler::new();
    compiler.compile_program(&program)?;
    Ok(CompiledProgram {
        bytecode: compiler.bytecode,
        components: compiler.components,
    })
}