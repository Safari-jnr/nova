// src/parser.rs - Parser for Nova
use crate::ast::*;
use crate::error::NovaError;
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.current_token().token_type, TokenType::Eof)
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.current_token().token_type)
            == std::mem::discriminant(token_type)
    }
    
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        let index = self.current + offset;
        if index < self.tokens.len() {
            Some(&self.tokens[index])
        } else {
            None
        }
    }
    
    fn expect(&mut self, token_type: TokenType, message: &str) -> Result<&Token, NovaError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            let pos = self.current_token().position;
            Err(NovaError::ParseError {
                message: message.to_string(),
                line: pos.line,
            })
        }
    }
    
    fn parse_program(&mut self) -> Result<Program, NovaError> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(Program { statements })
    }
    
    fn parse_statement(&mut self) -> Result<Statement, NovaError> {
        let pos = self.current_token().position;
        
        match &self.current_token().token_type {
            TokenType::Let => self.parse_let(),
            TokenType::Const => self.parse_const(),
            TokenType::Fn => self.parse_function(),
            TokenType::Return => self.parse_return(),
            TokenType::If => self.parse_if(),
            TokenType::For => self.parse_for(),
            TokenType::While => self.parse_while(),
            TokenType::Component => self.parse_component(),
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression {
                    expr,
                    position: pos,
                })
            }
        }
    }
    
    fn parse_let(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        
        let name = match &self.advance().token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => {
                return Err(NovaError::ParseError {
                    message: "Expected identifier after 'let'".to_string(),
                    line: pos.line,
                })
            }
        };
        
        self.expect(TokenType::Equal, "Expected '=' after variable name")?;
        
        let value = self.parse_expression()?;
        
        Ok(Statement::Let {
            name,
            value,
            position: pos,
        })
    }
    
    fn parse_const(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        
        let name = match &self.advance().token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => {
                return Err(NovaError::ParseError {
                    message: "Expected identifier after 'const'".to_string(),
                    line: pos.line,
                })
            }
        };
        
        self.expect(TokenType::Equal, "Expected '=' after constant name")?;
        
        let value = self.parse_expression()?;
        
        Ok(Statement::Const {
            name,
            value,
            position: pos,
        })
    }
    
    fn parse_function(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        let is_async = false;
        
        let name = match &self.advance().token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => {
                return Err(NovaError::ParseError {
                    message: "Expected function name".to_string(),
                    line: pos.line,
                })
            }
        };
        
        self.expect(TokenType::LeftParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                match &self.advance().token_type {
                    TokenType::Identifier(p) => params.push(p.clone()),
                    _ => {
                        return Err(NovaError::ParseError {
                            message: "Expected parameter name".to_string(),
                            line: pos.line,
                        })
                    }
                }
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.expect(TokenType::RightParen, "Expected ')' after parameters")?;
        self.expect(TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = self.parse_block()?;
        
        Ok(Statement::Function {
            name,
            params,
            body,
            is_async,
            position: pos,
        })
    }
    
    fn parse_return(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        
        let value = if self.is_at_end() || self.check(&TokenType::RightBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        Ok(Statement::Return { value, position: pos })
    }
    
    fn parse_if(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        let condition = self.parse_expression()?;
        
        self.expect(TokenType::LeftBrace, "Expected '{' after if condition")?;
        let then_branch = self.parse_block()?;
        
        let else_branch = if self.match_token(&[TokenType::Else]) {
            self.expect(TokenType::LeftBrace, "Expected '{' after else")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            position: pos,
        })
    }
    
    fn parse_for(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        
        let variable = match &self.advance().token_type {
            TokenType::Identifier(v) => v.clone(),
            _ => {
                return Err(NovaError::ParseError {
                    message: "Expected variable name in for loop".to_string(),
                    line: pos.line,
                })
            }
        };
        
        self.expect(TokenType::In, "Expected 'in' after loop variable")?;
        let iterable = self.parse_expression()?;
        
        self.expect(TokenType::LeftBrace, "Expected '{' after for expression")?;
        let body = self.parse_block()?;
        
        Ok(Statement::For {
            variable,
            iterable,
            body,
            position: pos,
        })
    }
    
    fn parse_while(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position;
        let condition = self.parse_expression()?;
        
        self.expect(TokenType::LeftBrace, "Expected '{' after while condition")?;
        let body = self.parse_block()?;
        
        Ok(Statement::While {
            condition,
            body,
            position: pos,
        })
    }
    
    fn parse_component(&mut self) -> Result<Statement, NovaError> {
        let pos = self.advance().position; // consume 'component'
        
        // Parse component name
        let name = match &self.advance().token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => {
                return Err(NovaError::ParseError {
                    message: "Expected component name after 'component'".to_string(),
                    line: pos.line,
                })
            }
        };
        
        self.expect(TokenType::LeftBrace, "Expected '{' after component name")?;
        
        // Initialize component parts
        let mut state = None;
        let mut computed = None;
        let mut methods = None; // For methods = { ... } syntax
        let mut methods_map: Vec<(String, Statement)> = Vec::new(); // For fn name() { } syntax
        let mut props = None;
        let mut lifecycle = Vec::new();
        let mut render = None;
        
        // Parse component body
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match &self.current_token().token_type {
                TokenType::State => {
                    self.advance(); // consume 'state'
                    self.expect(TokenType::Equal, "Expected '=' after 'state'")?;
                    state = Some(self.parse_expression()?);
                }
                TokenType::Computed => {
                    self.advance(); // consume 'computed'
                    self.expect(TokenType::Equal, "Expected '=' after 'computed'")?;
                    computed = Some(self.parse_expression()?);
                }
                TokenType::Methods => {
                    self.advance(); // consume 'methods'
                    self.expect(TokenType::Equal, "Expected '=' after 'methods'")?;
                    methods = Some(self.parse_expression()?);
                }
                TokenType::Props => {
                    self.advance(); // consume 'props'
                    self.expect(TokenType::Equal, "Expected '=' after 'props'")?;
                    props = Some(self.parse_expression()?);
                }
                TokenType::OnMount => {
                    self.advance(); // consume 'onMount'
                    self.expect(TokenType::LeftParen, "Expected '(' after 'onMount'")?;
                    self.expect(TokenType::RightParen, "Expected ')' after 'onMount'")?;
                    self.expect(TokenType::LeftBrace, "Expected '{' after 'onMount()'")?;
                    let body = self.parse_block()?;
                    lifecycle.push(LifecycleHook::OnMount(body));
                }
                TokenType::OnUnmount => {
                    self.advance(); // consume 'onUnmount'
                    self.expect(TokenType::LeftParen, "Expected '(' after 'onUnmount'")?;
                    self.expect(TokenType::RightParen, "Expected ')' after 'onUnmount'")?;
                    self.expect(TokenType::LeftBrace, "Expected '{' after 'onUnmount()'")?;
                    let body = self.parse_block()?;
                    lifecycle.push(LifecycleHook::OnUnmount(body));
                }
                TokenType::Fn => {
                    // Parse function (could be render or other methods)
                    let func = self.parse_function()?;
                    
                    // Check if it's the render function
                    if let Statement::Function { name: func_name, .. } = &func {
                        if func_name == "render" {
                            render = Some(Box::new(func));
                        } else {
                            // Other methods - collect them
                            methods_map.push((func_name.clone(), func));
                        }
                    }
                }
                _ => {
                    return Err(NovaError::ParseError {
                        message: format!("Unexpected token in component body: {:?}", self.current_token().token_type),
                        line: self.current_token().position.line,
                    });
                }
            }
        }
        
        self.expect(TokenType::RightBrace, "Expected '}' after component body")?;
        
        // Use methods object if defined, otherwise create from methods_map
        // For MVP, we'll prioritize the methods = { } syntax if present
        let final_methods = if methods.is_some() {
            methods
        } else if !methods_map.is_empty() {
            // Methods were defined as individual functions
            // For now, just mark that methods exist
            Some(Expression::Null)
        } else {
            None
        };
        
        // Ensure render function exists (required for now)
        let render_stmt = render.unwrap_or_else(|| {
            Box::new(Statement::Expression {
                expr: Expression::Null,
                position: pos,
            })
        });
        
        Ok(Statement::Component {
            name,
            state,
            computed,
            methods: final_methods,
            props,
            lifecycle,
            render: render_stmt,
            position: pos,
        })
    }
    
    fn parse_block(&mut self) -> Result<Vec<Statement>, NovaError> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }
    
    fn parse_expression(&mut self) -> Result<Expression, NovaError> {
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<Expression, NovaError> {
        let expr = self.parse_ternary()?;
        
        if self.match_token(&[TokenType::Equal, TokenType::PlusEqual, TokenType::MinusEqual, TokenType::StarEqual, TokenType::SlashEqual]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::Equal => BinaryOp::Assign,
                TokenType::PlusEqual => BinaryOp::AddAssign,
                TokenType::MinusEqual => BinaryOp::SubAssign,
                TokenType::StarEqual => BinaryOp::MulAssign,
                TokenType::SlashEqual => BinaryOp::DivAssign,
                _ => unreachable!(),
            };
            
            let value = self.parse_assignment()?;
            return Ok(Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(value),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_ternary(&mut self) -> Result<Expression, NovaError> {
        let mut expr = self.parse_or()?;
        
        if self.match_token(&[TokenType::Question]) {
            let then_val = self.parse_or()?;
            self.expect(TokenType::Colon, "Expected ':' in ternary expression")?;
            let else_val = self.parse_ternary()?; // Right associative
            
            expr = Expression::Ternary {
                condition: Box::new(expr),
                then_val: Box::new(then_val),
                else_val: Box::new(else_val),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_or(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_and()?;
        
        while self.match_token(&[TokenType::Or, TokenType::OrOr]) {
            let right = self.parse_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(&[TokenType::And, TokenType::AndAnd]) {
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_comparison()?;
        
        while self.match_token(&[TokenType::EqualEqual, TokenType::NotEqual]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_range()?;
        
        while self.match_token(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEq,
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEq,
                _ => unreachable!(),
            };
            
            let right = self.parse_range()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_range(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_term()?;
        
        // Check for range operators: .. or ..=
        if self.match_token(&[TokenType::DotDot, TokenType::DotDotEqual]) {
            let inclusive = matches!(self.tokens[self.current - 1].token_type, TokenType::DotDotEqual);
            let right = self.parse_term()?;
            
            left = Expression::Range {
                start: Box::new(left),
                end: Box::new(right),
                inclusive,
            };
        }
        
        Ok(left)
    }
    
    fn parse_term(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_factor()?;
        
        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            
            let right = self.parse_factor()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_factor(&mut self) -> Result<Expression, NovaError> {
        let mut left = self.parse_unary()?;
        
        while self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expression, NovaError> {
        if self.match_token(&[TokenType::Minus, TokenType::Not, TokenType::Bang, TokenType::PlusPlus, TokenType::MinusMinus]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::Minus => UnaryOp::Negate,
                TokenType::Not | TokenType::Bang => UnaryOp::Not,
                TokenType::PlusPlus => UnaryOp::Increment,
                TokenType::MinusMinus => UnaryOp::Decrement,
                _ => unreachable!(),
            };
            
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator,
                operand: Box::new(operand),
            });
        }
        
        self.parse_postfix()
    }
    
    fn parse_postfix(&mut self) -> Result<Expression, NovaError> {
        let mut expr = self.parse_call()?;
        
        // Check for postfix operators
        if self.match_token(&[TokenType::PlusPlus, TokenType::MinusMinus]) {
            let operator = match self.tokens[self.current - 1].token_type {
                TokenType::PlusPlus => UnaryOp::Increment,
                TokenType::MinusMinus => UnaryOp::Decrement,
                _ => unreachable!(),
            };
            
            expr = Expression::Postfix {
                operand: Box::new(expr),
                operator,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_call(&mut self) -> Result<Expression, NovaError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::LeftBracket]) {
                // Bracket notation: obj[index]
                let index = self.parse_expression()?;
                self.expect(TokenType::RightBracket, "Expected ']' after index")?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: Box::new(index),
                    computed: true,
                };
            } else if self.match_token(&[TokenType::Dot]) {
                // Allow both identifiers and keywords as property names
                let property_name = match &self.current_token().token_type {
                    TokenType::Identifier(name) => name.clone(),
                    // Allow keywords as property names
                    TokenType::State => "state".to_string(),
                    TokenType::Methods => "methods".to_string(),
                    TokenType::Computed => "computed".to_string(),
                    TokenType::Props => "props".to_string(),
                    TokenType::This => "this".to_string(),
                    TokenType::Component => "component".to_string(),
                    TokenType::Let => "let".to_string(),
                    TokenType::Const => "const".to_string(),
                    TokenType::Fn => "fn".to_string(),
                    TokenType::If => "if".to_string(),
                    TokenType::Else => "else".to_string(),
                    TokenType::For => "for".to_string(),
                    TokenType::While => "while".to_string(),
                    TokenType::Return => "return".to_string(),
                    TokenType::True => "true".to_string(),
                    TokenType::False => "false".to_string(),
                    TokenType::Null => "null".to_string(),
                    TokenType::And => "and".to_string(),
                    TokenType::Or => "or".to_string(),
                    TokenType::Not => "not".to_string(),
                    TokenType::In => "in".to_string(),
                    _ => {
                        return Err(NovaError::ParseError {
                            message: "Expected property name after '.'".to_string(),
                            line: 0,
                        })
                    }
                };
                self.advance();
                let property = Expression::Identifier(property_name);
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: false,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, function: Expression) -> Result<Expression, NovaError> {
        let mut args = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.expect(TokenType::RightParen, "Expected ')' after arguments")?;
        
        Ok(Expression::Call {
            function: Box::new(function),
            args,
        })
    }
    
    fn parse_primary(&mut self) -> Result<Expression, NovaError> {
        match &self.current_token().token_type.clone() {
            TokenType::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            TokenType::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expression::Null)
            }
            TokenType::This => {
                self.advance();
                Ok(Expression::This)
            }
            TokenType::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Expression::Number(num))
            }
            TokenType::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expression::String(string))
            }
            TokenType::Identifier(name) => {
                let id = name.clone();
                self.advance();
                Ok(Expression::Identifier(id))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                if !self.check(&TokenType::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenType::RightBracket, "Expected ']' after array elements")?;
                Ok(Expression::Array(elements))
            }
            TokenType::LeftBrace => {
                self.advance();
                let mut properties = Vec::new();
                
                if !self.check(&TokenType::RightBrace) {
                    loop {
                        // Parse key (must be identifier or string)
                        let key = match &self.current_token().token_type {
                            TokenType::Identifier(k) => {
                                let key = k.clone();
                                self.advance();
                                key
                            }
                            TokenType::String(k) => {
                                let key = k.clone();
                                self.advance();
                                key
                            }
                            _ => {
                                let pos = self.current_token().position;
                                return Err(NovaError::ParseError {
                                    message: "Expected property name in object literal".to_string(),
                                    line: pos.line,
                                });
                            }
                        };
                        
                        self.expect(TokenType::Colon, "Expected ':' after property name")?;
                        let value = self.parse_expression()?;
                        properties.push((key, value));
                        
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenType::RightBrace, "Expected '}' after object properties")?;
                Ok(Expression::Object(properties))
            }
            TokenType::View => {
                self.advance(); // consume 'view'
                self.expect(TokenType::LeftBrace, "Expected '{' after 'view'")?;
                
                let elements = self.parse_ui_elements()?;
                
                self.expect(TokenType::RightBrace, "Expected '}' after view block")?;
                Ok(Expression::View { elements })
            }
            _ => {
                let pos = self.current_token().position;
                Err(NovaError::ParseError {
                    message: format!("Unexpected token: {:?}", self.current_token().token_type),
                    line: pos.line,
                })
            }
        }
    }
    
    fn parse_ui_elements(&mut self) -> Result<Vec<UIElement>, NovaError> {
        let mut elements = Vec::new();
        
        // Parse UI elements until we hit closing brace
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            elements.push(self.parse_ui_element()?);
        }
        
        Ok(elements)
    }
    
    fn parse_ui_element(&mut self) -> Result<UIElement, NovaError> {
        // UI elements look like: ComponentName(prop: value, ...) { children }
        // Example: Container(padding: 20) { Text("Hello") }
        
        // Get component tag name
        let tag = match &self.current_token().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => {
                return Err(NovaError::ParseError {
                    message: "Expected component name".to_string(),
                    line: self.current_token().position.line,
                })
            }
        };
        self.advance();
        
        let mut props = Vec::new();
        let mut children = Vec::new();
        
        // Parse props if there's a parenthesis
        if self.match_token(&[TokenType::LeftParen]) {
            if !self.check(&TokenType::RightParen) {
                loop {
                    // Check if this is name:value syntax or just a value
                    // Look ahead to see if there's a colon
                    let is_named_prop = matches!(&self.current_token().token_type, TokenType::Identifier(_))
                        && self.peek_ahead(1).map(|t| matches!(t.token_type, TokenType::Colon)).unwrap_or(false);
                    
                    if is_named_prop {
                        // Parse prop name
                        let prop_name = match &self.current_token().token_type {
                            TokenType::Identifier(name) => name.clone(),
                            _ => {
                                return Err(NovaError::ParseError {
                                    message: "Expected property name".to_string(),
                                    line: self.current_token().position.line,
                                })
                            }
                        };
                        self.advance();
                        
                        self.expect(TokenType::Colon, "Expected ':' after property name")?;
                        
                        // Parse property value (expression)
                        let prop_value = self.parse_expression()?;
                        props.push((prop_name, prop_value));
                    } else {
                        // Shorthand: just a value, use "text" or "children" as property name
                        // For Text components, first arg is "text"
                        let prop_value = self.parse_expression()?;
                        let prop_name = if props.is_empty() {
                            "text".to_string()  // First prop defaults to "text"
                        } else {
                            format!("arg{}", props.len())  // Additional args are arg1, arg2, etc.
                        };
                        props.push((prop_name, prop_value));
                    }
                    
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.expect(TokenType::RightParen, "Expected ')' after component props")?;
        }
        
        // Parse children if there's a brace
        if self.match_token(&[TokenType::LeftBrace]) {
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                children.push(self.parse_ui_element()?);
            }
            self.expect(TokenType::RightBrace, "Expected '}' after component children")?;
        }
        
        Ok(UIElement {
            tag,
            props,
            children,
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, NovaError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}