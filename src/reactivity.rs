// src/reactivity.rs - Event Handling and Reactivity System for Nova

use crate::value::Value;
use crate::error::NovaError;
use std::collections::HashMap;

/// Event handler information
#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event_type: String,      // "click", "change", "submit", etc.
    pub handler_name: String,     // Method name to call
    pub component_id: String,     // Component instance ID
}

pub struct ReactivityEngine {
    event_handlers: Vec<EventHandler>,
    component_states: HashMap<String, Value>,
}

impl ReactivityEngine {
    pub fn new() -> Self {
        ReactivityEngine {
            event_handlers: Vec::new(),
            component_states: HashMap::new(),
        }
    }
    
    /// Register an event handler
    pub fn register_handler(&mut self, handler: EventHandler) {
        self.event_handlers.push(handler);
    }
    
    /// Get event handler by ID
    pub fn get_handler(&self, event_id: &str) -> Option<&EventHandler> {
        self.event_handlers.iter().find(|h| {
            format!("{}_{}", h.component_id, h.event_type) == event_id
        })
    }
    
    /// Update component state (triggers re-render)
    pub fn set_state(&mut self, component_id: &str, new_state: Value) -> Result<(), NovaError> {
        self.component_states.insert(component_id.to_string(), new_state);
        // In a full implementation, this would trigger a re-render
        Ok(())
    }
    
    /// Get component state
    pub fn get_state(&self, component_id: &str) -> Option<&Value> {
        self.component_states.get(component_id)
    }
    
    /// Generate event handler JavaScript for web
    pub fn generate_event_js(&self, event_type: &str, handler_id: &str) -> String {
        format!(
            r#"
            function handle_{}() {{
                fetch('/event/{}', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ event: '{}' }})
                }}).then(response => {{
                    if (response.ok) {{
                        window.location.reload(); // Simple re-render
                    }}
                }});
            }}
            "#,
            handler_id, handler_id, event_type
        )
    }
}

/// Convert event prop name to event type
/// onClick -> click, onChange -> change, onSubmit -> submit
pub fn event_prop_to_type(prop_name: &str) -> Option<String> {
    if prop_name.starts_with("on") && prop_name.len() > 2 {
        let event_type = &prop_name[2..];
        Some(event_type.to_lowercase())
    } else {
        None
    }
}

/// Check if a prop is an event handler
pub fn is_event_prop(prop_name: &str) -> bool {
    prop_name.starts_with("on") && prop_name.len() > 2 
        && prop_name.chars().nth(2).map(|c| c.is_uppercase()).unwrap_or(false)
}