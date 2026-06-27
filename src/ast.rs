// src/ast.rs - Abstract Syntax Tree for Nova
// This defines the structure of Nova programs after parsing
// Think of it like a tree of your code's meaning

use crate::error::Position;

/// Main program structure
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A statement (does something, doesn't return a value)
#[derive(Debug, Clone)]
pub enum Statement {
    // Variable declarations
    // let x = 5
    Let {
        name: String,
        value: Expression,
        position: Position,
    },
    
    // const PI = 3.14
    Const {
        name: String,
        value: Expression,
        position: Position,
    },
    
    // Function declaration
    // fn add(a, b) { return a + b }
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        is_async: bool,
        position: Position,
    },
    
    // Component declaration
    // component Counter { ... }
    Component {
        name: String,
        state: Option<Expression>,           // state = { count: 0 }
        computed: Option<Expression>,        // computed = { ... }
        methods: Option<Expression>,         // methods = { ... }
        props: Option<Expression>,           // props = { ... }
        lifecycle: Vec<LifecycleHook>,      // onMount, onUnmount, watch
        render: Box<Statement>,              // fn render() { ... }
        position: Position,
    },
    
    // Page declaration (like component but for routing)
    Page {
        name: String,
        guards: Vec<String>,                 // Route guards
        state: Option<Expression>,
        methods: Option<Expression>,
        lifecycle: Vec<LifecycleHook>,
        render: Box<Statement>,
        position: Position,
    },
    
    // Database declaration
    // database MyDB { table users { ... } }
    Database {
        name: String,
        tables: Vec<TableDefinition>,
        position: Position,
    },
    
    // Return statement
    // return x + 5
    Return {
        value: Option<Expression>,
        position: Position,
    },
    
    // If statement
    // if condition { ... } else { ... }
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        position: Position,
    },
    
    // For loop
    // for i in 0..10 { ... }
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
        position: Position,
    },
    
    // While loop
    // while condition { ... }
    While {
        condition: Expression,
        body: Vec<Statement>,
        position: Position,
    },
    
    // Match expression
    // match x { "A" => ..., _ => ... }
    Match {
        value: Expression,
        arms: Vec<MatchArm>,
        position: Position,
    },
    
    // Try-catch
    // try { ... } catch error { ... } finally { ... }
    Try {
        try_block: Vec<Statement>,
        catch_var: Option<String>,
        catch_block: Option<Vec<Statement>>,
        finally_block: Option<Vec<Statement>>,
        position: Position,
    },
    
    // Expression as statement
    // someFunction()
    Expression {
        expr: Expression,
        position: Position,
    },
    
    // Import statement
    // import { foo, bar } from "./module.no"
    Import {
        items: Vec<String>,
        from: String,
        position: Position,
    },
    
    // Export statement
    // export fn hello() { ... }
    Export {
        item: Box<Statement>,
        position: Position,
    },
}

/// An expression (evaluates to a value)
#[derive(Debug, Clone)]
pub enum Expression {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    
    // this keyword (component instance reference)
    This,
    
    // Identifier (variable name)
    Identifier(String),
    
    // Array literal
    // [1, 2, 3]
    Array(Vec<Expression>),
    
    // Object literal
    // { name: "John", age: 30 }
    Object(Vec<(String, Expression)>),
    
    // Binary operations
    // x + y, a * b, etc.
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    
    // Unary operations
    // -x, not flag
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },
    
    // Postfix operations
    // x++, x--
    Postfix {
        operand: Box<Expression>,
        operator: UnaryOp,
    },
    
    // Function call
    // greet("World")
    Call {
        function: Box<Expression>,
        args: Vec<Expression>,
    },
    
    // Member access
    // user.name, array[0]
    Member {
        object: Box<Expression>,
        property: Box<Expression>,
        computed: bool,  // true for array[0], false for obj.prop
    },
    
    // Arrow function
    // fn(x) => x * 2
    Arrow {
        params: Vec<String>,
        body: Box<Expression>,
    },
    
    // Template string
    // "Hello, ${name}!"
    Template {
        parts: Vec<String>,      // ["Hello, ", "!"]
        expressions: Vec<Expression>,  // [name]
    },
    
    // Ternary
    // condition ? true_val : false_val
    Ternary {
        condition: Box<Expression>,
        then_val: Box<Expression>,
        else_val: Box<Expression>,
    },
    
    // Range
    // 0..10
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,  // 0..10 vs 0..=10
    },
    
    // Await expression
    // await fetchData()
    Await {
        expr: Box<Expression>,
    },
    
    // UI View (special for components)
    // view { Container() { Text("Hello") } }
    View {
        elements: Vec<UIElement>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,         // +
    Subtract,    // -
    Multiply,    // *
    Divide,      // /
    Modulo,      // %
    
    // Comparison
    Equal,       // ==
    NotEqual,    // !=
    Less,        // <
    LessEq,      // <=
    Greater,     // >
    GreaterEq,   // >=
    
    // Logical
    And,         // and
    Or,          // or
    
    // Assignment
    Assign,      // =
    AddAssign,   // +=
    SubAssign,   // -=
    MulAssign,   // *=
    DivAssign,   // /=
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,      // -x
    Not,         // not x
    Increment,   // ++x
    Decrement,   // --x
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

/// Pattern for match
#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Expression),
    Identifier(String),
    Wildcard,  // _
}

/// Lifecycle hooks for components
#[derive(Debug, Clone)]
pub enum LifecycleHook {
    OnMount(Vec<Statement>),
    OnUnmount(Vec<Statement>),
    Watch {
        target: String,
        handler: Vec<Statement>,
    },
}

/// Database table definition
#[derive(Debug, Clone)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

/// Database column definition
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub name: String,
    pub col_type: ColumnType,
    pub constraints: Vec<ColumnConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Number,
    Text,
    Boolean,
    DateTime,
}

#[derive(Debug, Clone)]
pub enum ColumnConstraint {
    Primary,
    AutoIncrement,
    Unique,
    Required,
    Default(Expression),
    Foreign { table: String, column: String },
    Encrypted,
    Min(i64),
    Max(i64),
}

/// UI Element (for view blocks)
#[derive(Debug, Clone)]
pub struct UIElement {
    pub tag: String,                        // "Container", "Text", "Button"
    pub props: Vec<(String, Expression)>,   // Properties
    pub children: Vec<UIElement>,           // Nested elements
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
}

// Helper methods for common checks
impl Expression {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::Number(_)
                | Expression::String(_)
                | Expression::Boolean(_)
                | Expression::Null
        )
    }
}

impl Statement {
    pub fn is_declaration(&self) -> bool {
        matches!(
            self,
            Statement::Let { .. }
                | Statement::Const { .. }
                | Statement::Function { .. }
                | Statement::Component { .. }
        )
    }
}