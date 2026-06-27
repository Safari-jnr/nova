// src/styling.rs - Styling System for Nova UI Components

use crate::value::Value;
use std::collections::HashMap;

pub struct StyleEngine {
    theme: HashMap<String, String>,
}

impl StyleEngine {
    pub fn new() -> Self {
        let mut theme = HashMap::new();
        
        // Default theme colors
        theme.insert("primary".to_string(), "#007bff".to_string());
        theme.insert("secondary".to_string(), "#6c757d".to_string());
        theme.insert("success".to_string(), "#28a745".to_string());
        theme.insert("danger".to_string(), "#dc3545".to_string());
        theme.insert("warning".to_string(), "#ffc107".to_string());
        theme.insert("info".to_string(), "#17a2b8".to_string());
        theme.insert("light".to_string(), "#f8f9fa".to_string());
        theme.insert("dark".to_string(), "#343a40".to_string());
        
        StyleEngine { theme }
    }
    
    /// Convert Nova style object to CSS string
    pub fn to_css(&self, style_obj: &HashMap<String, Value>) -> String {
        let mut css = String::new();
        
        for (key, value) in style_obj {
            let css_key = self.camel_to_kebab(key);
            let css_value = self.value_to_css(value);
            
            if !css_value.is_empty() {
                css.push_str(&format!("{}: {}; ", css_key, css_value));
            }
        }
        
        css
    }
    
    /// Convert camelCase to kebab-case
    fn camel_to_kebab(&self, s: &str) -> String {
        let mut result = String::new();
        
        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('-');
                result.push(ch.to_lowercase().next().unwrap());
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    /// Convert Nova value to CSS value
    fn value_to_css(&self, value: &Value) -> String {
        match value {
            Value::String(s) => {
                // Check if it's a theme color
                if let Some(color) = self.theme.get(s) {
                    color.clone()
                } else {
                    s.clone()
                }
            }
            Value::Number(n) => {
                // Numbers become pixels by default
                if n.fract() == 0.0 {
                    format!("{}px", *n as i32)
                } else {
                    format!("{}px", n)
                }
            }
            Value::Boolean(b) => {
                if *b { "true".to_string() } else { "false".to_string() }
            }
            _ => String::new(),
        }
    }
    
    /// Generate CSS classes from style definitions
    pub fn generate_classes(&self, styles: &HashMap<String, Value>) -> String {
        let mut css = String::new();
        
        for (class_name, style_def) in styles {
            if let Value::Object(style_obj) = style_def {
                css.push_str(&format!(".{} {{\n", class_name));
                
                for (prop, val) in style_obj {
                    let css_prop = self.camel_to_kebab(prop);
                    let css_val = self.value_to_css(val);
                    
                    if !css_val.is_empty() {
                        css.push_str(&format!("  {}: {};\n", css_prop, css_val));
                    }
                }
                
                css.push_str("}\n\n");
            }
        }
        
        css
    }
    
    /// Parse flexbox layout properties
    pub fn parse_flex(&self, props: &HashMap<String, Value>) -> String {
        let mut css = String::from("display: flex; ");
        
        if let Some(Value::String(direction)) = props.get("direction") {
            css.push_str(&format!("flex-direction: {}; ", direction));
        }
        
        if let Some(Value::String(justify)) = props.get("justify") {
            css.push_str(&format!("justify-content: {}; ", justify));
        }
        
        if let Some(Value::String(align)) = props.get("align") {
            css.push_str(&format!("align-items: {}; ", align));
        }
        
        if let Some(gap) = props.get("gap") {
            css.push_str(&format!("gap: {}; ", self.value_to_css(gap)));
        }
        
        css
    }
    
    /// Parse grid layout properties
    pub fn parse_grid(&self, props: &HashMap<String, Value>) -> String {
        let mut css = String::from("display: grid; ");
        
        if let Some(Value::Number(cols)) = props.get("columns") {
            css.push_str(&format!("grid-template-columns: repeat({}, 1fr); ", *cols as i32));
        }
        
        if let Some(gap) = props.get("gap") {
            css.push_str(&format!("gap: {}; ", self.value_to_css(gap)));
        }
        
        css
    }
    
    /// Merge multiple style objects
    pub fn merge_styles(&self, styles: Vec<&HashMap<String, Value>>) -> HashMap<String, Value> {
        let mut merged = HashMap::new();
        
        for style in styles {
            for (key, value) in style {
                merged.insert(key.clone(), value.clone());
            }
        }
        
        merged
    }
    
    /// Generate responsive CSS
    pub fn generate_responsive(&self, breakpoints: &HashMap<String, HashMap<String, Value>>) -> String {
        let mut css = String::new();
        
        for (breakpoint, styles) in breakpoints {
            let media_query = match breakpoint.as_str() {
                "mobile" => "@media (max-width: 768px)",
                "tablet" => "@media (min-width: 769px) and (max-width: 1024px)",
                "desktop" => "@media (min-width: 1025px)",
                custom => custom,
            };
            
            css.push_str(&format!("{} {{\n", media_query));
            
            // Convert styles to CSS (simplified for now)
            for (_, style_val) in styles {
                if let Value::Object(_) = style_val {
                    // Would generate CSS here
                }
            }
            
            css.push_str("}\n\n");
        }
        
        css
    }
}

/// Helper to extract style prop from component props
pub fn extract_style(props: &HashMap<String, Value>) -> Option<HashMap<String, Value>> {
    if let Some(Value::Object(style)) = props.get("style") {
        Some(style.clone())
    } else {
        None
    }
}

/// Helper to extract className prop
pub fn extract_class_name(props: &HashMap<String, Value>) -> Option<String> {
    if let Some(Value::String(class)) = props.get("className") {
        Some(class.clone())
    } else {
        None
    }
}