// src/main.rs - Nova Programming Language
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use colored::*;

mod lexer;
mod parser;
mod ast;
mod error;
mod value;
mod bytecode;
mod compiler;
mod vm;
mod stdlib;
mod ui_renderer;
mod terminal_ui;
mod http_server;
mod interactive_server;  
mod styling;
mod websocket;

use crate::error::NovaError;

#[derive(Parser)]
#[command(name = "nova")]
#[command(about = "Nova Programming Language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        file: PathBuf,
        #[arg(short, long)]
        debug: bool,
    },
    Check {
        file: PathBuf,
    },
    Version,
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Run { file, debug } => run_file(file, debug),
        Commands::Check { file } => check_file(file),
        Commands::Version => show_version(),
    };
    
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run_file(file: PathBuf, debug: bool) -> Result<(), NovaError> {
    println!("{} Compiling {}...", "".green(), file.display());
    
    let source = fs::read_to_string(&file)
        .map_err(|e| NovaError::IOError(format!("Failed to read file: {}", e)))?;
    
    if debug {
        println!("\n{}", "=== LEXER OUTPUT ===".cyan().bold());
    }
    
    let tokens = lexer::tokenize(&source)?;
    
    if debug {
        for token in &tokens {
            println!("{:?}", token);
        }
        println!("\n{}", "=== PARSER OUTPUT ===".cyan().bold());
    }
    
    let ast = parser::parse(tokens)?;
    
    if debug {
        println!("{:#?}", ast);
        println!("\n{}", "=== COMPILER OUTPUT ===".cyan().bold());
    }
    
    let compiled = compiler::compile(ast)?;
    
    if debug {
        compiled.bytecode.disassemble("Nova Bytecode");
        println!("{}", "=== EXECUTION ===".cyan().bold());
    }
    
    let mut vm = vm::VM::new();
    vm.execute(compiled.bytecode, compiled.components)?;
    
    println!("\n{} Program completed successfully!", "✓".green().bold());
    
    Ok(())
}

fn check_file(file: PathBuf) -> Result<(), NovaError> {
    println!("{} Checking {}...", "🔍".cyan(), file.display());
    
    let source = fs::read_to_string(&file)
        .map_err(|e| NovaError::IOError(format!("Failed to read file: {}", e)))?;
    
    let tokens = lexer::tokenize(&source)?;
    let _ast = parser::parse(tokens)?;
    
    println!("{} No errors found!", "✓".green().bold());
    Ok(())
}

fn show_version() -> Result<(), NovaError> {
    println!("{}", "Nova Programming Language v1.0.0".green().bold());
    println!("One Language. Infinite Possibilities.");
    println!("Built with Rust 🦀");
    Ok(())
}