// src/database.rs - Built-in Database System for Nova

use crate::value::Value;
use crate::error::NovaError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Database table schema
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: String,  // "number", "string", "boolean", "date"
    pub nullable: bool,
    pub default: Option<Value>,
}

/// In-memory database (can be backed by SQLite later)
#[derive(Clone)]
pub struct NovaDatabase {
    tables: Arc<Mutex<HashMap<String, Table>>>,
    name: String,
}

#[derive(Debug, Clone)]
struct Table {
    schema: TableSchema,
    rows: Vec<HashMap<String, Value>>,
    next_id: i64,
}

impl NovaDatabase {
    pub fn new(name: &str) -> Self {
        NovaDatabase {
            tables: Arc::new(Mutex::new(HashMap::new())),
            name: name.to_string(),
        }
    }
    
    /// Create a new table
    pub fn create_table(&self, schema: TableSchema) -> Result<(), NovaError> {
        let mut tables = self.tables.lock().unwrap();
        
        if tables.contains_key(&schema.name) {
            return Err(NovaError::RuntimeError(
                format!("Table '{}' already exists", schema.name)
            ));
        }
        
        let table = Table {
            schema,
            rows: Vec::new(),
            next_id: 1,
        };
        
        tables.insert(table.schema.name.clone(), table);
        Ok(())
    }
    
    /// Insert a row
    pub fn insert(&self, table_name: &str, data: HashMap<String, Value>) -> Result<Value, NovaError> {
        let mut tables = self.tables.lock().unwrap();
        
        let table = tables.get_mut(table_name)
            .ok_or_else(|| NovaError::RuntimeError(format!("Table '{}' not found", table_name)))?;
        
        // Add auto-increment ID if table has primary key
        let mut row = data.clone();
        if let Some(pk) = &table.schema.primary_key {
            if !row.contains_key(pk) {
                row.insert(pk.clone(), Value::Number(table.next_id as f64));
                table.next_id += 1;
            }
        }
        
        // Validate columns
        for col in &table.schema.columns {
            if !row.contains_key(&col.name) && !col.nullable {
                if let Some(default) = &col.default {
                    row.insert(col.name.clone(), default.clone());
                } else {
                    return Err(NovaError::RuntimeError(
                        format!("Column '{}' is required", col.name)
                    ));
                }
            }
        }
        
        table.rows.push(row.clone());
        Ok(Value::Object(row))
    }
    
    /// Select rows (simple query)
    pub fn select(&self, table_name: &str, filter: Option<&str>) -> Result<Value, NovaError> {
        let tables = self.tables.lock().unwrap();
        
        let table = tables.get(table_name)
            .ok_or_else(|| NovaError::RuntimeError(format!("Table '{}' not found", table_name)))?;
        
        let mut results = Vec::new();
        
        for row in &table.rows {
            // Simple filter: column = value
            let mut matches = true;
            
            if let Some(filter_str) = filter {
                // Parse simple filter: "name = 'Alice'" or "age > 18"
                // For now, return all rows (full query parsing would go here)
                matches = true;
            }
            
            if matches {
                results.push(Value::Object(row.clone()));
            }
        }
        
        Ok(Value::Array(results))
    }
    
    /// Update rows
    pub fn update(&self, table_name: &str, updates: HashMap<String, Value>, filter: Option<&str>) -> Result<usize, NovaError> {
        let mut tables = self.tables.lock().unwrap();
        
        let table = tables.get_mut(table_name)
            .ok_or_else(|| NovaError::RuntimeError(format!("Table '{}' not found", table_name)))?;
        
        let mut updated_count = 0;
        
        for row in &mut table.rows {
            // Apply filter (simplified)
            let matches = true; // In full version, parse filter
            
            if matches {
                for (key, value) in &updates {
                    row.insert(key.clone(), value.clone());
                }
                updated_count += 1;
            }
        }
        
        Ok(updated_count)
    }
    
    /// Delete rows
    pub fn delete(&self, table_name: &str, filter: Option<&str>) -> Result<usize, NovaError> {
        let mut tables = self.tables.lock().unwrap();
        
        let table = tables.get_mut(table_name)
            .ok_or_else(|| NovaError::RuntimeError(format!("Table '{}' not found", table_name)))?;
        
        let initial_len = table.rows.len();
        
        // Simple delete all for now (in full version, apply filter)
        if filter.is_none() {
            table.rows.clear();
        }
        
        Ok(initial_len - table.rows.len())
    }
    
    /// Count rows
    pub fn count(&self, table_name: &str) -> Result<usize, NovaError> {
        let tables = self.tables.lock().unwrap();
        
        let table = tables.get(table_name)
            .ok_or_else(|| NovaError::RuntimeError(format!("Table '{}' not found", table_name)))?;
        
        Ok(table.rows.len())
    }
    
    /// Execute SQL-like query
    pub fn query(&self, sql: &str) -> Result<Value, NovaError> {
        // Parse SQL and execute
        // For MVP: Simple SELECT * FROM table
        
        if sql.to_lowercase().starts_with("select") {
            // Extract table name
            let parts: Vec<&str> = sql.split_whitespace().collect();
            if let Some(idx) = parts.iter().position(|&s| s.to_lowercase() == "from") {
                if let Some(table_name) = parts.get(idx + 1) {
                    return self.select(table_name, None);
                }
            }
        }
        
        Err(NovaError::RuntimeError("Invalid SQL query".to_string()))
    }
}

/// Global database registry (simple implementation without lazy_static)
static mut DATABASES: Option<HashMap<String, NovaDatabase>> = None;

fn init_databases() -> &'static mut HashMap<String, NovaDatabase> {
    unsafe {
        if DATABASES.is_none() {
            DATABASES = Some(HashMap::new());
        }
        DATABASES.as_mut().unwrap()
    }
}

/// Get or create database
pub fn get_database(name: &str) -> NovaDatabase {
    let dbs = init_databases();
    
    if !dbs.contains_key(name) {
        dbs.insert(name.to_string(), NovaDatabase::new(name));
    }
    
    dbs.get(name).unwrap().clone()
}

/// Create database from schema
pub fn create_database(name: &str, tables: Vec<TableSchema>) -> Result<(), NovaError> {
    let db = get_database(name);
    
    for table in tables {
        db.create_table(table)?;
    }
    
    Ok(())
}