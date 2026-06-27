// src/interactive_server.rs - Enhanced Full-Stack Server
//  Edit tasks (update title/priority)
//  Filter by status
//  Search functionality
//  Categories/tags
//  Due dates
//  User authentication

use crate::value::Value;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Enhanced application state with auth and categories
#[derive(Clone)]
pub struct AppState {
    pub tasks: Arc<Mutex<Vec<HashMap<String, Value>>>>,
    pub categories: Arc<Mutex<Vec<String>>>,
    pub users: Arc<Mutex<HashMap<String, String>>>, // username -> password hash
    pub sessions: Arc<Mutex<HashMap<String, String>>>, // session_token -> username
    pub ui_html: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(html: String) -> Self {
        let initial_tasks = vec![
            {
                let mut task = HashMap::new();
                task.insert("id".to_string(), Value::Number(1.0));
                task.insert("title".to_string(), Value::String("Build Nova Framework".to_string()));
                task.insert("status".to_string(), Value::String("completed".to_string()));
                task.insert("priority".to_string(), Value::String("high".to_string()));
                task.insert("category".to_string(), Value::String("Development".to_string()));
                task.insert("dueDate".to_string(), Value::String("2026-02-15".to_string()));
                task
            },
            {
                let mut task = HashMap::new();
                task.insert("id".to_string(), Value::Number(2.0));
                task.insert("title".to_string(), Value::String("Add CRUD Features".to_string()));
                task.insert("status".to_string(), Value::String("in-progress".to_string()));
                task.insert("priority".to_string(), Value::String("high".to_string()));
                task.insert("category".to_string(), Value::String("Development".to_string()));
                task.insert("dueDate".to_string(), Value::String("2026-03-01".to_string()));
                task
            },
            {
                let mut task = HashMap::new();
                task.insert("id".to_string(), Value::Number(3.0));
                task.insert("title".to_string(), Value::String("Deploy to Production".to_string()));
                task.insert("status".to_string(), Value::String("pending".to_string()));
                task.insert("priority".to_string(), Value::String("medium".to_string()));
                task.insert("category".to_string(), Value::String("DevOps".to_string()));
                task.insert("dueDate".to_string(), Value::String("2026-03-15".to_string()));
                task
            },
        ];
        
        let initial_categories = vec![
            "Development".to_string(),
            "DevOps".to_string(),
            "Design".to_string(),
            "Marketing".to_string(),
            "Personal".to_string(),
        ];
        
        // Demo user: admin / password
        let mut users = HashMap::new();
        users.insert("admin".to_string(), "password".to_string()); // In production, use proper hashing!
        users.insert("demo".to_string(), "demo".to_string());
        
        AppState {
            tasks: Arc::new(Mutex::new(initial_tasks)),
            categories: Arc::new(Mutex::new(initial_categories)),
            users: Arc::new(Mutex::new(users)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ui_html: Arc::new(Mutex::new(html)),
        }
    }
}

pub struct InteractiveServer {
    port: u16,
    state: AppState,
}

impl InteractiveServer {
    pub fn new(port: u16, html: String) -> Self {
        InteractiveServer {
            port,
            state: AppState::new(html),
        }
    }
    
    pub fn start(&self) {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).expect("Failed to bind to address");
        
        println!("\n🚀 Nova Enhanced Server Running!");
        println!("📱 Open: http://localhost:{}", self.port);
        println!("🔐 Demo Login: admin / password");
        println!("✨ Full CRUD, Search, Filter, Categories, Auth");
        println!("Press Ctrl+C to stop\n");
        
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let state = self.state.clone();
                    thread::spawn(move || {
                        let mut buffer = [0; 8192];
                        let bytes_read = stream.read(&mut buffer).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                        
                        let lines: Vec<&str> = request.lines().collect();
                        if lines.is_empty() {
                            return;
                        }
                        
                        let parts: Vec<&str> = lines[0].split_whitespace().collect();
                        if parts.len() < 2 {
                            return;
                        }
                        
                        let method = parts[0];
                        let path = parts[1];
                        
                        let response = handle_request(method, path, &request, &state);
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}

fn handle_request(method: &str, path: &str, request: &str, state: &AppState) -> String {
    match (method, path) {
        ("GET", "/") => {
            let html = state.ui_html.lock().unwrap();
            http_response(200, "text/html", &html)
        }
        
        // ===== TASKS ENDPOINTS =====
        ("GET", path) if path.starts_with("/api/tasks") => {
            let query = extract_query_params(path);
            let tasks = state.tasks.lock().unwrap();
            
            // Filter by status, category, or search
            let filtered: Vec<_> = tasks.iter()
                .filter(|task| {
                    if let Some(status) = query.get("status") {
                        if let Some(Value::String(task_status)) = task.get("status") {
                            if task_status != status {
                                return false;
                            }
                        }
                    }
                    
                    if let Some(category) = query.get("category") {
                        if let Some(Value::String(task_cat)) = task.get("category") {
                            if task_cat != category {
                                return false;
                            }
                        }
                    }
                    
                    if let Some(search) = query.get("search") {
                        if let Some(Value::String(title)) = task.get("title") {
                            if !title.to_lowercase().contains(&search.to_lowercase()) {
                                return false;
                            }
                        }
                    }
                    
                    true
                })
                .cloned()
                .collect();
            
            let json = tasks_to_json(&filtered);
            http_response(200, "application/json", &json)
        }
        
        ("POST", "/api/tasks") => {
            let body = extract_body(request);
            let new_task = parse_task_json(&body);
            
            let mut tasks = state.tasks.lock().unwrap();
            let new_id = tasks.len() as f64 + 1.0;
            
            let mut task = new_task;
            task.insert("id".to_string(), Value::Number(new_id));
            
            // Set defaults
            if !task.contains_key("status") {
                task.insert("status".to_string(), Value::String("pending".to_string()));
            }
            if !task.contains_key("priority") {
                task.insert("priority".to_string(), Value::String("medium".to_string()));
            }
            if !task.contains_key("category") {
                task.insert("category".to_string(), Value::String("Uncategorized".to_string()));
            }
            
            tasks.push(task);
            
            let json = format!("{{\"success\": true, \"id\": {}}}", new_id);
            http_response(200, "application/json", &json)
        }
        
        ("PUT", path) if path.starts_with("/api/tasks/") => {
            let id_str = path.strip_prefix("/api/tasks/").unwrap_or("0").split('?').next().unwrap_or("0");
            let id = id_str.parse::<f64>().unwrap_or(0.0);
            
            let body = extract_body(request);
            let updates = parse_task_json(&body);
            
            let mut tasks = state.tasks.lock().unwrap();
            if let Some(task) = tasks.iter_mut().find(|t| {
                if let Some(Value::Number(tid)) = t.get("id") {
                    *tid == id
                } else {
                    false
                }
            }) {
                for (key, value) in updates {
                    task.insert(key, value);
                }
                http_response(200, "application/json", "{\"success\": true}")
            } else {
                http_response(404, "application/json", "{\"error\": \"Task not found\"}")
            }
        }
        
        ("DELETE", path) if path.starts_with("/api/tasks/") => {
            let id_str = path.strip_prefix("/api/tasks/").unwrap_or("0");
            let id = id_str.parse::<f64>().unwrap_or(0.0);
            
            let mut tasks = state.tasks.lock().unwrap();
            tasks.retain(|t| {
                if let Some(Value::Number(tid)) = t.get("id") {
                    *tid != id
                } else {
                    true
                }
            });
            
            http_response(200, "application/json", "{\"success\": true}")
        }
        
        // ===== CATEGORIES ENDPOINTS =====
        ("GET", "/api/categories") => {
            let categories = state.categories.lock().unwrap();
            let json = format!("[{}]", 
                categories.iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            http_response(200, "application/json", &json)
        }
        
        ("POST", "/api/categories") => {
            let body = extract_body(request);
            if let Some(name) = extract_json_field(&body, "name") {
                let mut categories = state.categories.lock().unwrap();
                if !categories.contains(&name) {
                    categories.push(name);
                    http_response(200, "application/json", "{\"success\": true}")
                } else {
                    http_response(400, "application/json", "{\"error\": \"Category already exists\"}")
                }
            } else {
                http_response(400, "application/json", "{\"error\": \"Missing name\"}")
            }
        }
        
        // ===== AUTHENTICATION ENDPOINTS =====
        ("POST", "/api/auth/login") => {
            let body = extract_body(request);
            let username = extract_json_field(&body, "username").unwrap_or_default();
            let password = extract_json_field(&body, "password").unwrap_or_default();
            
            let users = state.users.lock().unwrap();
            
            if let Some(stored_password) = users.get(&username) {
                if stored_password == &password {
                    // Generate session token
                    let token = format!("session_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs());
                    
                    let mut sessions = state.sessions.lock().unwrap();
                    sessions.insert(token.clone(), username.clone());
                    
                    let json = format!("{{\"success\": true, \"token\": \"{}\", \"username\": \"{}\"}}", 
                        token, username);
                    http_response(200, "application/json", &json)
                } else {
                    http_response(401, "application/json", "{\"error\": \"Invalid credentials\"}")
                }
            } else {
                http_response(401, "application/json", "{\"error\": \"Invalid credentials\"}")
            }
        }
        
        ("POST", "/api/auth/logout") => {
            let body = extract_body(request);
            if let Some(token) = extract_json_field(&body, "token") {
                let mut sessions = state.sessions.lock().unwrap();
                sessions.remove(&token);
            }
            http_response(200, "application/json", "{\"success\": true}")
        }
        
        ("GET", "/api/auth/verify") => {
            let query = extract_query_params(path);
            if let Some(token) = query.get("token") {
                let sessions = state.sessions.lock().unwrap();
                if let Some(username) = sessions.get(token) {
                    let json = format!("{{\"valid\": true, \"username\": \"{}\"}}", username);
                    return http_response(200, "application/json", &json);
                }
            }
            http_response(200, "application/json", "{\"valid\": false}")
        }
        
        _ => {
            http_response(404, "text/plain", "Not Found")
        }
    }
}

fn http_response(status: u16, content_type: &str, body: &str) -> String {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "Unknown",
    };
    
    format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         \r\n\
         {}",
        status, status_text, content_type, body.len(), body
    )
}

fn extract_body(request: &str) -> String {
    if let Some(pos) = request.find("\r\n\r\n") {
        request[pos + 4..].to_string()
    } else {
        String::new()
    }
}

fn extract_query_params(path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    if let Some(query_start) = path.find('?') {
        let query = &path[query_start + 1..];
        for pair in query.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];
                params.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    params
}

fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\"", field);
    if let Some(start) = json.find(&pattern) {
        if let Some(colon) = json[start..].find(':') {
            let after_colon = &json[start + colon + 1..];
            if let Some(quote_start) = after_colon.find('"') {
                if let Some(quote_end) = after_colon[quote_start + 1..].find('"') {
                    let value = &after_colon[quote_start + 1..quote_start + 1 + quote_end];
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn parse_task_json(json: &str) -> HashMap<String, Value> {
    let mut task = HashMap::new();
    
    let fields = vec!["title", "status", "priority", "category", "dueDate", "description"];
    
    for field in fields {
        if let Some(value) = extract_json_field(json, field) {
            task.insert(field.to_string(), Value::String(value));
        }
    }
    
    task
}

fn tasks_to_json(tasks: &[HashMap<String, Value>]) -> String {
    let mut items = Vec::new();
    
    for task in tasks {
        let mut fields = Vec::new();
        
        for (key, value) in task {
            let val_str = match value {
                Value::String(s) => format!("\"{}\"", s),
                Value::Number(n) => n.to_string(),
                Value::Boolean(b) => b.to_string(),
                _ => "null".to_string(),
            };
            fields.push(format!("\"{}\": {}", key, val_str));
        }
        
        items.push(format!("{{{}}}", fields.join(", ")));
    }
    
    format!("[{}]", items.join(", "))
}

pub fn start_interactive_server(html: String, port: u16) {
    let server = InteractiveServer::new(port, html);
    server.start();
}