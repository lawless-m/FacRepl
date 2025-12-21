// REPL Binary
// Interactive DSL executor for Factorio Constraint Builder

use factorio_constraint_builder::{RconBridge, RconConfig, DslParser, DslExecutor};
use anyhow::Result;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "fcb-repl")]
#[command(about = "Interactive REPL for Factorio Constraint Builder", long_about = None)]
struct Args {
    /// Factorio RCON host
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// Factorio RCON port
    #[arg(short, long, default_value = "27015")]
    port: u16,

    /// RCON password
    #[arg(short = 'P', long, env = "FACTORIO_RCON_PASSWORD")]
    password: String,

    /// Connection timeout in seconds
    #[arg(short, long, default_value = "5")]
    timeout: u64,
    
    /// Load and execute script file on startup
    #[arg(short, long)]
    script: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("fcb_repl=info,factorio_constraint_builder=info")
        .init();

    let args = Args::parse();

    // Connect to Factorio
    println!("Connecting to Factorio at {}:{}...", args.host, args.port);
    
    let config = RconConfig {
        host: args.host,
        port: args.port,
        password: args.password,
        timeout: Duration::from_secs(args.timeout),
    };

    let rcon = RconBridge::connect(config).await?;
    let mut executor = DslExecutor::new(rcon);

    println!("Connected! Type :help for help, :quit to exit.\n");

    // Load script if provided
    if let Some(script_path) = args.script {
        println!("Loading script: {}", script_path);
        match std::fs::read_to_string(&script_path) {
            Ok(content) => {
                match DslParser::parse_script(&content) {
                    Ok(commands) => {
                        println!("Executing {} commands...", commands.len());
                        for cmd in commands {
                            if let Err(e) = executor.execute(cmd).await {
                                eprintln!("Error: {}", e);
                            }
                        }
                        println!("Script execution complete.\n");
                    }
                    Err(e) => eprintln!("Parse error: {}\n", e),
                }
            }
            Err(e) => eprintln!("Failed to load script: {}\n", e),
        }
    }

    // Start REPL
    let mut rl = DefaultEditor::new()?;
    
    // Load history if it exists
    let history_path = dirs::home_dir()
        .map(|mut p| {
            p.push(".fcb_history");
            p
        });
    
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    loop {
        let readline = rl.readline("fcb> ");
        
        match readline {
            Ok(line) => {
                let line = line.trim();
                
                // Skip empty lines
                if line.is_empty() {
                    continue;
                }
                
                // Add to history
                let _ = rl.add_history_entry(line);
                
                // Handle REPL commands
                if line.starts_with(':') {
                    match handle_repl_command(line, &mut executor).await {
                        Ok(should_exit) => {
                            if should_exit {
                                break;
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    continue;
                }
                
                // Parse and execute DSL command
                match DslParser::parse_line(line) {
                    Ok(cmd) => {
                        match executor.execute(cmd).await {
                            Ok(result) => {
                                if !result.is_empty() {
                                    println!("{}", result);
                                }
                            }
                            Err(e) => eprintln!("Execution error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(path) = history_path {
        let _ = rl.save_history(&path);
    }

    println!("Goodbye!");
    Ok(())
}

async fn handle_repl_command(line: &str, _executor: &mut DslExecutor) -> Result<bool> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    match parts[0] {
        ":help" | ":h" => {
            print_help();
            Ok(false)
        }
        ":quit" | ":q" | ":exit" => {
            Ok(true)
        }
        ":clear" | ":cls" => {
            print!("\x1B[2J\x1B[1;1H");
            Ok(false)
        }
        ":load" => {
            if parts.len() < 2 {
                eprintln!("Usage: :load <file>");
            } else {
                // TODO: Implement load
                println!("Load not yet implemented");
            }
            Ok(false)
        }
        ":entities" => {
            println!("Available entities:");
            println!("  Belts: belt-yellow, belt-red, belt-blue");
            println!("  Underground: underground-belt-yellow/red/blue");
            println!("  Splitters: splitter-yellow/red/blue");
            println!("  Inserters: inserter-burner/basic/fast/long/stack");
            println!("  Assemblers: assembler-1/2/3");
            println!("  Power: power-pole-small/medium/big");
            println!("  Fluids: pipe, pump, storage-tank");
            Ok(false)
        }
        _ => {
            eprintln!("Unknown REPL command: {}", parts[0]);
            eprintln!("Type :help for available commands");
            Ok(false)
        }
    }
}

fn print_help() {
    println!("Factorio Constraint Builder REPL");
    println!();
    println!("REPL Commands:");
    println!("  :help, :h           - Show this help");
    println!("  :quit, :q, :exit    - Exit REPL");
    println!("  :clear, :cls        - Clear screen");
    println!("  :load <file>        - Load and execute script");
    println!("  :entities           - List available entity types");
    println!();
    println!("DSL Commands:");
    println!("  <entity> <x> <y> <:dir>              - Place entity");
    println!("  what-at <x> <y>                       - Query position");
    println!("  can-place <entity> <x> <y>            - Check placement");
    println!("  clear <x1> <y1> <x2> <y2>             - Clear area");
    println!("  undo [count]                          - Undo commands");
    println!();
    println!("Examples:");
    println!("  belt-blue 10 20 :n");
    println!("  assembler-2 15 15 recipe:green-circuit");
    println!("  what-at 10 20");
    println!("  clear 0 0 50 50");
    println!();
    println!("Directions: :n :s :e :w :ne :nw :se :sw");
    println!("See DSL-REFERENCE.md for complete syntax reference.");
}
