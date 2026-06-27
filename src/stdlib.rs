// src/stdlib.rs - Standard Library for Nova
// Built-in functions available to all Nova programs

use crate::value::Value;
use crate::error::NovaError;
use crate::ui_renderer::UIRenderer;
use crate::terminal_ui::TerminalUI;
use crate::interactive_server;

/// Call a built-in function
pub fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, NovaError> {
    match name {
        // UI rendering
        "renderHTML" => builtin_render_html(args),
        "renderConsole" => builtin_render_console(args),
        "renderTerminal" => builtin_render_terminal(args),
        "serveHTTP" => builtin_serve_http(args),
        
        // Component methods
        "setState" => builtin_set_state(args),
        
        // Database operations
        "dbInsert" => builtin_db_insert(args),
        "dbSelect" => builtin_db_select(args),
        "dbUpdate" => builtin_db_update(args),
        "dbDelete" => builtin_db_delete(args),
        "dbQuery" => builtin_db_query(args),
        
        // File I/O
        "writeFile" => builtin_write_file(args),
        
        // Array methods
        "push" => builtin_push(args),
        "pop" => builtin_pop(args),
        "shift" => builtin_shift(args),
        "contains" => builtin_contains(args),
        "join" => builtin_join(args),
        "reverse" => builtin_reverse(args),
        "sort" => builtin_sort(args),
        
        // String methods
        "toUpper" | "upper" => builtin_to_upper(args),
        "toLower" | "lower" => builtin_to_lower(args),
        "trim" => builtin_trim(args),
        "split" => builtin_split(args),
        "replace" => builtin_replace(args),
        "startsWith" => builtin_starts_with(args),
        "endsWith" => builtin_ends_with(args),
        "charAt" => builtin_char_at(args),
        
        // Math functions
        "sqrt" => builtin_sqrt(args),
        "pow" => builtin_pow(args),
        "abs" => builtin_abs(args),
        "round" => builtin_round(args),
        "floor" => builtin_floor(args),
        "ceil" => builtin_ceil(args),
        "min" => builtin_min(args),
        "max" => builtin_max(args),
        
        // Type functions
        "type" => builtin_type(args),
        "str" => builtin_str(args),
        "num" => builtin_num(args),
        
        _ => Err(NovaError::UndefinedFunction(name.to_string())),
    }
}

// ============================================================================
// ARRAY METHODS
// ============================================================================

/// push(array, value) - Add element to end of array
fn builtin_push(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("push() takes 2 arguments".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.push(args[1].clone());
            Ok(Value::Array(new_arr))
        }
        _ => Err(NovaError::TypeError {
            expected: "array".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// pop(array) - Remove and return last element
fn builtin_pop(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("pop() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            Ok(new_arr.pop().unwrap_or(Value::Null))
        }
        _ => Err(NovaError::TypeError {
            expected: "array".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// shift(array) - Remove and return first element
fn builtin_shift(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("shift() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(arr[0].clone())
            }
        }
        _ => Err(NovaError::TypeError {
            expected: "array".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// contains(array, value) - Check if array contains value
fn builtin_contains(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("contains() takes 2 arguments".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            Ok(Value::Boolean(arr.contains(&args[1])))
        }
        Value::String(s) => {
            if let Value::String(search) = &args[1] {
                Ok(Value::Boolean(s.contains(search.as_str())))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Err(NovaError::TypeError {
            expected: "array or string".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// join(array, separator) - Join array elements into string
fn builtin_join(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("join() takes 2 arguments".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            let sep = args[1].to_string();
            let strings: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
            Ok(Value::String(strings.join(&sep)))
        }
        _ => Err(NovaError::TypeError {
            expected: "array".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// reverse(array) - Reverse array
fn builtin_reverse(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("reverse() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.reverse();
            Ok(Value::Array(new_arr))
        }
        _ => Err(NovaError::TypeError {
            expected: "array".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// sort(array) - Sort array
fn builtin_sort(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("sort() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.sort_by(|a, b| {
                match (a, b) {
                    (Value::Number(x), Value::Number(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            Ok(Value::Array(new_arr))
        }
        _ => Err(NovaError::TypeError {
            expected: "array".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

// ============================================================================
// STRING METHODS
// ============================================================================

/// toUpper(string) - Convert to uppercase
fn builtin_to_upper(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("toUpper() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Ok(Value::String(args[0].to_string().to_uppercase())),
    }
}

/// toLower(string) - Convert to lowercase
fn builtin_to_lower(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("toLower() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Ok(Value::String(args[0].to_string().to_lowercase())),
    }
}

/// trim(string) - Remove whitespace from both ends
fn builtin_trim(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("trim() takes 1 argument".to_string()));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Ok(Value::String(args[0].to_string().trim().to_string())),
    }
}

/// split(string, separator) - Split string into array
fn builtin_split(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("split() takes 2 arguments".to_string()));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sep)) => {
            let parts: Vec<Value> = s.split(sep.as_str())
                .map(|p| Value::String(p.to_string()))
                .collect();
            Ok(Value::Array(parts))
        }
        _ => Err(NovaError::RuntimeError("split() requires two strings".to_string())),
    }
}

/// replace(string, from, to) - Replace occurrences
fn builtin_replace(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 3 {
        return Err(NovaError::RuntimeError("replace() takes 3 arguments".to_string()));
    }
    
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::String(from), Value::String(to)) => {
            Ok(Value::String(s.replace(from.as_str(), to.as_str())))
        }
        _ => Err(NovaError::RuntimeError("replace() requires three strings".to_string())),
    }
}

/// startsWith(string, prefix) - Check if string starts with prefix
fn builtin_starts_with(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("startsWith() takes 2 arguments".to_string()));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => {
            Ok(Value::Boolean(s.starts_with(prefix.as_str())))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

/// endsWith(string, suffix) - Check if string ends with suffix
fn builtin_ends_with(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("endsWith() takes 2 arguments".to_string()));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => {
            Ok(Value::Boolean(s.ends_with(suffix.as_str())))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

/// charAt(string, index) - Get character at index
fn builtin_char_at(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("charAt() takes 2 arguments".to_string()));
    }
    
    match (&args[0], &args[1]) {
        (Value::String(s), Value::Number(idx)) => {
            let i = *idx as usize;
            if i < s.len() {
                Ok(Value::String(s.chars().nth(i).unwrap().to_string()))
            } else {
                Ok(Value::Null)
            }
        }
        _ => Err(NovaError::RuntimeError("charAt() requires string and number".to_string())),
    }
}

// ============================================================================
// MATH FUNCTIONS
// ============================================================================

/// sqrt(number) - Square root
fn builtin_sqrt(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("sqrt() takes 1 argument".to_string()));
    }
    
    match args[0].as_number() {
        Some(n) => Ok(Value::Number(n.sqrt())),
        None => Err(NovaError::TypeError {
            expected: "number".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// pow(base, exponent) - Power
fn builtin_pow(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("pow() takes 2 arguments".to_string()));
    }
    
    match (args[0].as_number(), args[1].as_number()) {
        (Some(base), Some(exp)) => Ok(Value::Number(base.powf(exp))),
        _ => Err(NovaError::RuntimeError("pow() requires two numbers".to_string())),
    }
}

/// abs(number) - Absolute value
fn builtin_abs(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("abs() takes 1 argument".to_string()));
    }
    
    match args[0].as_number() {
        Some(n) => Ok(Value::Number(n.abs())),
        None => Err(NovaError::TypeError {
            expected: "number".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// round(number) - Round to nearest integer
fn builtin_round(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("round() takes 1 argument".to_string()));
    }
    
    match args[0].as_number() {
        Some(n) => Ok(Value::Number(n.round())),
        None => Err(NovaError::TypeError {
            expected: "number".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// floor(number) - Round down
fn builtin_floor(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("floor() takes 1 argument".to_string()));
    }
    
    match args[0].as_number() {
        Some(n) => Ok(Value::Number(n.floor())),
        None => Err(NovaError::TypeError {
            expected: "number".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// ceil(number) - Round up
fn builtin_ceil(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("ceil() takes 1 argument".to_string()));
    }
    
    match args[0].as_number() {
        Some(n) => Ok(Value::Number(n.ceil())),
        None => Err(NovaError::TypeError {
            expected: "number".to_string(),
            got: args[0].type_name().to_string(),
        }),
    }
}

/// min(a, b, ...) - Minimum value
fn builtin_min(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.is_empty() {
        return Err(NovaError::RuntimeError("min() requires at least 1 argument".to_string()));
    }
    
    let numbers: Vec<f64> = args.iter()
        .filter_map(|v| v.as_number())
        .collect();
    
    if numbers.is_empty() {
        return Err(NovaError::RuntimeError("min() requires numeric arguments".to_string()));
    }
    
    Ok(Value::Number(numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b))))
}

/// max(a, b, ...) - Maximum value
fn builtin_max(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.is_empty() {
        return Err(NovaError::RuntimeError("max() requires at least 1 argument".to_string()));
    }
    
    let numbers: Vec<f64> = args.iter()
        .filter_map(|v| v.as_number())
        .collect();
    
    if numbers.is_empty() {
        return Err(NovaError::RuntimeError("max() requires numeric arguments".to_string()));
    }
    
    Ok(Value::Number(numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))))
}

// ============================================================================
// TYPE FUNCTIONS
// ============================================================================

/// type(value) - Get type of value
fn builtin_type(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("type() takes 1 argument".to_string()));
    }
    
    Ok(Value::String(args[0].type_name().to_string()))
}

/// str(value) - Convert to string
fn builtin_str(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("str() takes 1 argument".to_string()));
    }
    
    Ok(Value::String(args[0].to_string()))
}

/// num(value) - Convert to number
fn builtin_num(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("num() takes 1 argument".to_string()));
    }
    
    match args[0].as_number() {
        Some(n) => Ok(Value::Number(n)),
        None => Err(NovaError::RuntimeError(
            format!("Cannot convert {} to number", args[0].type_name())
        )),
    }
}

// ============================================================================
// UI RENDERING FUNCTIONS
// ============================================================================

fn builtin_render_html(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("renderHTML() takes 1 argument".to_string()));
    }
    
    let mut renderer = UIRenderer::new();
    let html = renderer.render_to_html(&args[0]);
    Ok(Value::String(html))
}

fn builtin_render_console(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("renderConsole() takes 1 argument".to_string()));
    }
    
    let mut renderer = UIRenderer::new();
    let output = renderer.render_to_console(&args[0]);
    Ok(Value::String(output))
}

fn builtin_render_terminal(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("renderTerminal() takes 1 argument".to_string()));
    }
    
    let mut terminal = TerminalUI::new();
    terminal.run(&args[0]);
    Ok(Value::Null)
}

fn builtin_serve_http(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() < 1 || args.len() > 2 {
        return Err(NovaError::RuntimeError("serveHTTP() takes 1-2 arguments (ui, port?)".to_string()));
    }
    
    let port = if args.len() == 2 {
        match &args[1] {
            Value::Number(n) => *n as u16,
            _ => 3000,
        }
    } else {
        3000
    };
    
    // Generate HTML
    let mut renderer = UIRenderer::new();
    let html = renderer.render_to_html(&args[0]);
    
    // Start interactive server with API
    interactive_server::start_interactive_server(html, port);
    
    Ok(Value::Null)
}

// ============================================================================
// COMPONENT REACTIVITY
// ============================================================================

fn builtin_set_state(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("setState() takes 2 arguments (component, newState)".to_string()));
    }
    
    // In a full implementation, this would:
    // 1. Update the component's state
    // 2. Mark component for re-render
    // 3. Schedule render update
    
    // For now, we'll return the new state object
    // The component will need to manually update this.state
    Ok(args[1].clone())
}

// ============================================================================
// DATABASE OPERATIONS
// ============================================================================

fn builtin_db_insert(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("dbInsert() takes 2 arguments (table, data)".to_string()));
    }
    
    // Placeholder implementation
    // In full version: use database.rs
    Ok(Value::String("Row inserted".to_string()))
}

fn builtin_db_select(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() < 1 {
        return Err(NovaError::RuntimeError("dbSelect() takes at least 1 argument".to_string()));
    }
    
    // Placeholder: return empty array
    Ok(Value::Array(vec![]))
}

fn builtin_db_update(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() < 2 {
        return Err(NovaError::RuntimeError("dbUpdate() takes at least 2 arguments".to_string()));
    }
    
    Ok(Value::Number(0.0))
}

fn builtin_db_delete(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() < 1 {
        return Err(NovaError::RuntimeError("dbDelete() takes at least 1 argument".to_string()));
    }
    
    Ok(Value::Number(0.0))
}

fn builtin_db_query(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 1 {
        return Err(NovaError::RuntimeError("dbQuery() takes 1 argument (SQL query)".to_string()));
    }
    
    // Placeholder: return empty array
    Ok(Value::Array(vec![]))
}

// ============================================================================
// FILE I/O FUNCTIONS
// ============================================================================

fn builtin_write_file(args: Vec<Value>) -> Result<Value, NovaError> {
    if args.len() != 2 {
        return Err(NovaError::RuntimeError("writeFile() takes 2 arguments (filename, content)".to_string()));
    }
    
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => return Err(NovaError::RuntimeError("First argument must be a string (filename)".to_string())),
    };
    
    let content = match &args[1] {
        Value::String(s) => s,
        _ => return Err(NovaError::RuntimeError("Second argument must be a string (content)".to_string())),
    };
    
    use std::fs;
    match fs::write(filename, content) {
        Ok(_) => Ok(Value::String(format!("File '{}' written successfully", filename))),
        Err(e) => Err(NovaError::RuntimeError(format!("Failed to write file: {}", e))),
    }
}

pub fn builtin_start_websocket(args: Vec<crate::value::Value>) -> Result<crate::value::Value, crate::error::NovaError> {
    use crate::value::Value;
    use crate::error::NovaError;
    
    if args.len() != 1 {
        return Err(NovaError::RuntimeError(
            "startWebSocket requires 1 argument (port)".to_string()
        ));
    }
    
    let port = match &args[0] {
        Value::Number(n) => *n as u16,
        _ => return Err(NovaError::RuntimeError(
            "Port must be a number".to_string()
        )),
    };
    
    println!("🔌 Starting WebSocket server on port {}...", port);
    
    std::thread::spawn(move || {
        crate::websocket::start_websocket_server(port);
    });
    
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    Ok(Value::String(format!("WebSocket server started on port {}", port)))
}