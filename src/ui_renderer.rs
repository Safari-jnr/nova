// src/ui_renderer.rs - UI Component Rendering for Nova

use crate::value::Value;
use crate::styling::{StyleEngine, extract_style, extract_class_name};
use std::collections::HashMap;

pub struct UIRenderer {
    indent_level: usize,
    style_engine: StyleEngine,
}

impl UIRenderer {
    pub fn new() -> Self {
        UIRenderer { 
            indent_level: 0,
            style_engine: StyleEngine::new(),
        }
    }
    
    /// Render UI tree to HTML
    pub fn render_to_html(&mut self, ui_value: &Value) -> String {
        match ui_value {
            Value::Array(elements) => {
                let mut html = String::new();
                html.push_str("<!DOCTYPE html>\n");
                html.push_str("<html>\n");
                html.push_str("<head>\n");
                html.push_str("  <meta charset=\"UTF-8\">\n");
                html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
                html.push_str("  <title>Nova App</title>\n");
                html.push_str("  <style>\n");
                html.push_str(self.get_default_styles());
                html.push_str(self.get_interactive_styles());
                html.push_str("  </style>\n");
                html.push_str("</head>\n");
                html.push_str("<body>\n");
                
                for element in elements {
                    html.push_str(&self.render_element(element));
                }
                
                // Add interactive runtime
                html.push_str(&self.get_interactive_runtime());
                
                html.push_str("</body>\n");
                html.push_str("</html>\n");
                html
            }
            _ => String::from("<!-- Invalid UI structure -->"),
        }
    }
    
    /// Render UI tree to console (pretty print)
    pub fn render_to_console(&mut self, ui_value: &Value) -> String {
        match ui_value {
            Value::Array(elements) => {
                let mut output = String::new();
                output.push_str("╔═══════════════════════════════════════╗\n");
                output.push_str("║          NOVA UI RENDER              ║\n");
                output.push_str("╚═══════════════════════════════════════╝\n\n");
                
                for element in elements {
                    output.push_str(&self.render_element_console(element, 0));
                }
                
                output
            }
            _ => String::from("Invalid UI structure"),
        }
    }
    
    fn render_element(&mut self, element: &Value) -> String {
        if let Value::Object(obj) = element {
            let element_type = obj.get("type")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("div");
            
            let props = obj.get("props")
                .and_then(|v| if let Value::Object(p) = v { Some(p) } else { None });
            
            let children = obj.get("children")
                .and_then(|v| if let Value::Array(c) = v { Some(c) } else { None });
            
            match element_type {
                "Text" => self.render_text(props),
                "Container" => self.render_container(props, children),
                "Button" => self.render_button(props),
                "Input" => self.render_input(props),
                "Page" => self.render_page(props, children),
                "Form" => self.render_form(props, children),
                "Modal" => self.render_modal(props, children),
                _ => format!("<div data-component=\"{}\">Unknown component</div>", element_type),
            }
        } else {
            String::from("<!-- Invalid element -->")
        }
    }
    
    fn render_text(&self, props: Option<&HashMap<String, Value>>) -> String {
        let text = props
            .and_then(|p| p.get("text"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");
        
        let mut style = String::new();
        
        // Check for style object first
        if let Some(props_map) = props {
            if let Some(style_obj) = extract_style(props_map) {
                // Use style engine to convert object to CSS
                style.push_str(&self.style_engine.to_css(&style_obj));
            } else {
                // Legacy individual props
                let size = props_map.get("size")
                    .and_then(|v| if let Value::Number(n) = v { Some(*n as i32) } else { None });
                let color = props_map.get("color")
                    .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
                let weight = props_map.get("weight")
                    .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
                
                if let Some(s) = size {
                    style.push_str(&format!("font-size: {}px; ", s));
                }
                if let Some(c) = color {
                    style.push_str(&format!("color: {}; ", c));
                }
                if let Some(w) = weight {
                    style.push_str(&format!("font-weight: {}; ", w));
                }
            }
            
            // Check for className
            if let Some(class) = extract_class_name(props_map) {
                if style.is_empty() {
                    return format!("<span class=\"nova-text {}\">{}</span>", class, text);
                } else {
                    return format!("<span class=\"nova-text {}\" style=\"{}\">{}</span>", class, style, text);
                }
            }
        }
        
        if style.is_empty() {
            format!("<span class=\"nova-text\">{}</span>", text)
        } else {
            format!("<span class=\"nova-text\" style=\"{}\">{}</span>", style, text)
        }
    }
    
    fn render_container(&mut self, props: Option<&HashMap<String, Value>>, children: Option<&Vec<Value>>) -> String {
        let mut style = String::new();
        
        if let Some(p) = props {
            if let Some(Value::Number(padding)) = p.get("padding") {
                style.push_str(&format!("padding: {}px; ", padding));
            }
            if let Some(Value::Number(margin)) = p.get("margin") {
                style.push_str(&format!("margin: {}px; ", margin));
            }
            if let Some(Value::String(bg)) = p.get("backgroundColor") {
                style.push_str(&format!("background-color: {}; ", bg));
            }
        }
        
        let mut html = if style.is_empty() {
            String::from("<div class=\"nova-container\">")
        } else {
            format!("<div class=\"nova-container\" style=\"{}\">", style)
        };
        
        if let Some(child_list) = children {
            for child in child_list {
                html.push_str(&self.render_element(child));
            }
        }
        
        html.push_str("</div>");
        html
    }
    
    fn render_button(&self, props: Option<&HashMap<String, Value>>) -> String {
        let text = props
            .and_then(|p| p.get("text"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Button");
        
        let color = props
            .and_then(|p| p.get("color"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let size = props
            .and_then(|p| p.get("size"))
            .and_then(|v| if let Value::Number(n) = v { Some(*n as i32) } else { None });
        
        // Get interactive props
        let on_click = props
            .and_then(|p| p.get("onClick"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let nav = props
            .and_then(|p| p.get("nav"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let action = props
            .and_then(|p| p.get("action"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None });
        
        let mut style = String::new();
        if let Some(c) = color {
            style.push_str(&format!("background-color: {}; ", c));
        }
        if let Some(s) = size {
            style.push_str(&format!("font-size: {}px; ", s));
        }
        
        let mut attributes = String::new();
        if let Some(n) = nav {
            attributes.push_str(&format!(" data-nav=\"{}\"", n));
        }
        if let Some(a) = action {
            attributes.push_str(&format!(" data-action=\"{}\"", a));
        }
        if let Some(oc) = on_click {
            attributes.push_str(&format!(" onclick=\"{}\"", oc));
        }
        
        if style.is_empty() {
            format!("<button class=\"nova-button\"{}>{}</button>", attributes, text)
        } else {
            format!("<button class=\"nova-button\" style=\"{}\"{}>{}</button>", style, attributes, text)
        }
    }
    
    fn render_input(&self, props: Option<&HashMap<String, Value>>) -> String {
        let placeholder = props
            .and_then(|p| p.get("placeholder"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");
        
        let name = props
            .and_then(|p| p.get("name"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");
        
        let value = props
            .and_then(|p| p.get("value"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");
        
        let input_type = props
            .and_then(|p| p.get("type"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("text");
        
        format!(
            "<input class=\"nova-input\" type=\"{}\" placeholder=\"{}\" name=\"{}\" value=\"{}\" />",
            input_type, placeholder, name, value
        )
    }
    
    fn render_page(&mut self, props: Option<&HashMap<String, Value>>, children: Option<&Vec<Value>>) -> String {
        let page_id = props
            .and_then(|p| p.get("id"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("page");
        
        let active = props
            .and_then(|p| p.get("active"))
            .and_then(|v| if let Value::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        let display = if active { "block" } else { "none" };
        
        let mut html = format!("<div data-page=\"{}\" style=\"display: {};\">", page_id, display);
        
        if let Some(child_list) = children {
            for child in child_list {
                html.push_str(&self.render_element(child));
            }
        }
        
        html.push_str("</div>");
        html
    }
    
    fn render_form(&mut self, props: Option<&HashMap<String, Value>>, children: Option<&Vec<Value>>) -> String {
        let form_id = props
            .and_then(|p| p.get("id"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("form");
        
        let mut html = format!("<form id=\"{}\" class=\"nova-form\">", form_id);
        
        if let Some(child_list) = children {
            for child in child_list {
                html.push_str(&self.render_element(child));
            }
        }
        
        html.push_str("</form>");
        html
    }
    
    fn render_modal(&mut self, props: Option<&HashMap<String, Value>>, children: Option<&Vec<Value>>) -> String {
        let modal_id = props
            .and_then(|p| p.get("id"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("modal");
        
        let title = props
            .and_then(|p| p.get("title"))
            .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Modal");
        
        let mut html = format!("<div id=\"{}\" class=\"nova-modal\">", modal_id);
        html.push_str("<div class=\"nova-modal-content\">");
        html.push_str(&format!("<h2 style=\"margin-bottom: 20px;\">{}</h2>", title));
        
        if let Some(child_list) = children {
            for child in child_list {
                html.push_str(&self.render_element(child));
            }
        }
        
        html.push_str("</div></div>");
        html
    }
    
    fn render_element_console(&self, element: &Value, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        
        if let Value::Object(obj) = element {
            let element_type = obj.get("type")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Unknown");
            
            let props = obj.get("props")
                .and_then(|v| if let Value::Object(p) = v { Some(p) } else { None });
            
            let children = obj.get("children")
                .and_then(|v| if let Value::Array(c) = v { Some(c) } else { None });
            
            let mut output = String::new();
            
            // Render element type
            output.push_str(&format!("{}┌─ {} ", indent_str, element_type));
            
            // Render props
            if let Some(p) = props {
                let props_str = self.format_props_console(p);
                if !props_str.is_empty() {
                    output.push_str(&format!("({})", props_str));
                }
            }
            output.push('\n');
            
            // Render children
            if let Some(child_list) = children {
                for child in child_list {
                    output.push_str(&self.render_element_console(child, indent + 1));
                }
            }
            
            output
        } else {
            format!("{}Invalid element\n", indent_str)
        }
    }
    
    fn format_props_console(&self, props: &HashMap<String, Value>) -> String {
        let mut parts = Vec::new();
        
        for (key, value) in props {
            let value_str = match value {
                Value::String(s) => format!("\"{}\"", s),
                Value::Number(n) => n.to_string(),
                Value::Boolean(b) => b.to_string(),
                _ => "...".to_string(),
            };
            parts.push(format!("{}: {}", key, value_str));
        }
        
        parts.join(", ")
    }
    
    fn get_default_styles(&self) -> &str {
        r#"
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
      padding: 20px;
      background: #f5f5f5;
    }
    
    .nova-container {
      display: flex;
      flex-direction: column;
      gap: 10px;
      background: white;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
    
    .nova-text {
      color: #333;
      line-height: 1.5;
    }
    
    .nova-button {
      padding: 10px 20px;
      border: none;
      border-radius: 6px;
      background: #007bff;
      color: white;
      font-size: 14px;
      cursor: pointer;
      transition: background 0.2s;
    }
    
    .nova-button:hover {
      background: #0056b3;
    }
    
    .nova-input {
      padding: 10px;
      border: 1px solid #ddd;
      border-radius: 6px;
      font-size: 14px;
    }
    
    .nova-input:focus {
      outline: none;
      border-color: #007bff;
    }
  "#
    }
    
    fn get_interactive_styles(&self) -> &str {
        r#"
    /* Interactive Elements */
    [data-nav], [data-action] {
      cursor: pointer;
      user-select: none;
    }
    
    [data-nav]:hover, [data-action]:hover {
      opacity: 0.9;
      transform: translateY(-1px);
    }
    
    [data-nav].active {
      background: #2c3e50 !important;
      border: 2px solid #3498db;
    }
    
    /* Responsive Design */
    @media (max-width: 768px) {
      body {
        padding: 10px;
      }
      
      .nova-container {
        gap: 8px;
      }
      
      .nova-button {
        padding: 8px 16px;
        font-size: 13px;
      }
    }
    
    @media (max-width: 480px) {
      .nova-text {
        font-size: 14px !important;
      }
      
      .nova-button {
        width: 100%;
        margin-bottom: 8px;
      }
    }
  "#
    }
    
    
    fn get_interactive_runtime(&self) -> String {
        String::from(r#"
<script>
class NovaApp {
  constructor() {
    this.init();
  }
  
  async fetchTasks() {
    try {
      const resp = await fetch('/api/tasks');
      const tasks = await resp.json();
      this.renderTasks(tasks);
    } catch (e) {
      console.error('Error fetching tasks:', e);
    }
  }
  
  async createTask(title, priority) {
    try {
      const resp = await fetch('/api/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: title, priority: priority || 'medium', status: 'pending' })
      });
      const data = await resp.json();
      if (data.success) {
        this.showToast('Task created!');
        this.fetchTasks();
      }
    } catch (e) {
      console.error('Error creating task:', e);
    }
  }
  
  async updateTask(id, updates) {
    try {
      const resp = await fetch('/api/tasks/' + id, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates)
      });
      const data = await resp.json();
      if (data.success) {
        this.showToast('Task updated!');
        this.fetchTasks();
      }
    } catch (e) {
      console.error('Error updating task:', e);
    }
  }
  
  async deleteTask(id) {
    if (!confirm('Are you sure you want to delete this task?')) return;
    try {
      const resp = await fetch('/api/tasks/' + id, { method: 'DELETE' });
      const data = await resp.json();
      if (data.success) {
        this.showToast('Task deleted!');
        this.fetchTasks();
      }
    } catch (e) {
      console.error('Error deleting task:', e);
    }
  }
  
  renderTasks(tasks) {
    var container = document.querySelector('[data-tasks-list]');
    if (!container) {
      var loadingText = Array.from(document.querySelectorAll('.nova-text')).find(function(el) {
        return el.textContent.includes('Loading');
      });
      if (loadingText && loadingText.parentElement) {
        container = document.createElement('div');
        container.setAttribute('data-tasks-list', 'true');
        container.style.margin = '20px 0';
        loadingText.parentElement.appendChild(container);
        loadingText.style.display = 'none';
      }
    }
    if (!container) {
      console.error('Could not find tasks container');
      return;
    }
    
    if (!tasks || tasks.length === 0) {
      container.innerHTML = '<div style="padding:40px; background:#fff; border-radius:8px; text-align:center"><p style="font-size:24px; margin:0">📝 No tasks yet!</p><p style="font-size:16px; margin:10px 0 0 0; color:#7f8c8d">Create your first task above</p></div>';
      return;
    }
    
    container.innerHTML = tasks.map(function(t) {
      var statusIcon = t.status === 'completed' ? '✅' : '📝';
      var title = t.title || 'Untitled Task';
      var priority = t.priority || 'medium';
      var status = t.status || 'pending';
      
      return '<div style="margin:20px 0; padding:25px; background:white; border-radius:12px; box-shadow:0 2px 8px rgba(0,0,0,0.1)">' +
        '<h3 style="margin:0 0 10px 0; font-size:22px; font-weight:bold; color:#2c3e50">' + statusIcon + ' ' + title + '</h3>' +
        '<p style="margin:10px 0; color:#7f8c8d; font-size:14px">' + priority + ' priority • ' + status + '</p>' +
        '<div style="margin-top:15px">' +
        '<button class="nova-button" style="background:#3498db; color:#fff; border:none; padding:10px 20px; border-radius:6px; cursor:pointer; margin-right:10px" onclick="app.updateTask(' + t.id + ', {status:\'completed\'})">✅ Complete</button>' +
        '<button class="nova-button" style="background:#e74c3c; color:#fff; border:none; padding:10px 20px; border-radius:6px; cursor:pointer" onclick="app.deleteTask(' + t.id + ')">🗑️ Delete</button>' +
        '</div>' +
        '</div>';
    }).join('');
  }
  
  showToast(msg) {
    var toast = document.createElement('div');
    toast.textContent = msg;
    toast.style.cssText = 'position:fixed; top:20px; right:20px; z-index:10000; padding:16px 24px; border-radius:8px; background:#2ecc71; color:white; box-shadow:0 4px 12px rgba(0,0,0,0.3); font-weight:500';
    document.body.appendChild(toast);
    setTimeout(function() {
      toast.remove();
    }, 3000);
  }
  
  init() {
    var self = this;
    setTimeout(function() { self.fetchTasks(); }, 100);
    console.log('✅ Nova App Ready');
    console.log('🚀 API: /api/tasks');
  }
}

window.app = new NovaApp();

// Helper functions for onclick handlers
function handleCreateTask() {
  var title = document.querySelector('[name="title"]');
  var description = document.querySelector('[name="description"]');
  var priority = document.querySelector('[name="priority"]');
  var category = document.querySelector('[name="category"]');
  var dueDate = document.querySelector('[name="dueDate"]');
  
  if (!title || !title.value) {
    alert('Please enter a task title');
    return;
  }
  
  app.createTask(
    title.value,
    priority ? priority.value || 'medium' : 'medium'
  );
  
  // Clear form
  title.value = '';
  if (description) description.value = '';
  if (priority) priority.value = '';
  if (category) category.value = '';
  if (dueDate) dueDate.value = '';
}

function filterTasks(status) {
  console.log('Filter by:', status);
  app.fetchTasks();
}

function searchTasks() {
  var query = document.querySelector('[name="searchQuery"]');
  if (query) {
    console.log('Search:', query.value);
  }
  app.fetchTasks();
}
</script>
"#)
    }
}
