// src/terminal_ui.rs - Terminal UI Runtime for Nova
// Renders UI components to terminal using colored output

use crate::value::Value;
use std::collections::HashMap;

pub struct TerminalUI {
    width: usize,
}

impl TerminalUI {
    pub fn new() -> Self {
        TerminalUI {
            width: 80,
        }
    }
    
    /// Run UI in terminal (interactive mode)
    pub fn run(&mut self, ui_value: &Value) {
        self.clear_screen();
        self.render(ui_value);
    }
    
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }
    
    fn render(&self, ui_value: &Value) {
        match ui_value {
            Value::Array(elements) => {
                for element in elements {
                    self.render_element(element, 0);
                }
            }
            _ => println!("Invalid UI structure"),
        }
    }
    
    fn render_element(&self, element: &Value, indent: usize) {
        if let Value::Object(obj) = element {
            let element_type = obj.get("type")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("div");
            
            let props = obj.get("props")
                .and_then(|v| if let Value::Object(p) = v { Some(p) } else { None });
            
            let children = obj.get("children")
                .and_then(|v| if let Value::Array(c) = v { Some(c) } else { None });
            
            match element_type {
                "Text" => self.render_text_terminal(props, indent),
                "Container" => self.render_container_terminal(props, children, indent),
                "Button" => self.render_button_terminal(props, indent),
                "Input" => self.render_input_terminal(props, indent),
                _ => println!("{}Unknown: {}", "  ".repeat(indent), element_type),
            }
        }
    }
    
    fn render_text_terminal(&self, props: Option<&HashMap<String, Value>>, indent: usize) {
        let text = props
            .and_then(|p| p.get("text"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");
        
        let size = props
            .and_then(|p| p.get("size"))
            .and_then(|v| if let Value::Number(n) = v { Some(*n as i32) } else { None });
        
        let color = props
            .and_then(|p| p.get("color"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let weight = props
            .and_then(|p| p.get("weight"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let padding = "  ".repeat(indent);
        
        // Apply formatting
        let mut formatted = String::new();
        
        // Size affects styling
        if let Some(s) = size {
            if s >= 32 {
                formatted.push_str("\x1B[1m"); // Bold for large text
            }
        }
        
        // Color
        match color {
            Some("blue") | Some("#3498db") => formatted.push_str("\x1B[34m"),
            Some("green") | Some("#2ecc71") => formatted.push_str("\x1B[32m"),
            Some("red") | Some("#e74c3c") => formatted.push_str("\x1B[31m"),
            Some("yellow") => formatted.push_str("\x1B[33m"),
            Some("cyan") => formatted.push_str("\x1B[36m"),
            _ => formatted.push_str("\x1B[37m"), // White
        }
        
        // Weight
        if weight == Some("bold") {
            formatted.push_str("\x1B[1m");
        }
        
        formatted.push_str(text);
        formatted.push_str("\x1B[0m"); // Reset
        
        println!("{}{}", padding, formatted);
    }
    
    fn render_container_terminal(&self, props: Option<&HashMap<String, Value>>, children: Option<&Vec<Value>>, indent: usize) {
        let padding = "  ".repeat(indent);
        
        // Check for background color
        let has_bg = props
            .and_then(|p| p.get("backgroundColor"))
            .is_some();
        
        if has_bg {
            println!("{}┌─────────────────────────────────┐", padding);
        }
        
        if let Some(child_list) = children {
            for child in child_list {
                self.render_element(child, indent + 1);
            }
        }
        
        if has_bg {
            println!("{}└─────────────────────────────────┘", padding);
        } else {
            // Just spacing
            println!();
        }
    }
    
    fn render_button_terminal(&self, props: Option<&HashMap<String, Value>>, indent: usize) {
        let text = props
            .and_then(|p| p.get("text"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Button");
        
        let color = props
            .and_then(|p| p.get("color"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let padding = "  ".repeat(indent);
        
        let color_code = match color {
            Some("blue") | Some("#3498db") | Some("#007bff") => "\x1B[44m\x1B[37m", // Blue bg, white text
            Some("green") | Some("#2ecc71") | Some("#28a745") => "\x1B[42m\x1B[37m",
            Some("red") | Some("#e74c3c") => "\x1B[41m\x1B[37m",
            _ => "\x1B[47m\x1B[30m", // White bg, black text
        };
        
        println!("{}{}  [ {} ]  \x1B[0m", padding, color_code, text);
    }
    
    fn render_input_terminal(&self, props: Option<&HashMap<String, Value>>, indent: usize) {
        let placeholder = props
            .and_then(|p| p.get("placeholder"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");
        
        let padding = "  ".repeat(indent);
        println!("{}[ {} ______________________ ]", padding, placeholder);
    }
}


