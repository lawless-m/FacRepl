// REPL Binary
// Interactive DSL executor for Factorio Constraint Builder

use anyhow::Result;
use clap::Parser;
use factorio_constraint_builder::{DslExecutor, DslParser, RconBridge, RconConfig};
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

    /// RCON password (or set FACTORIO_RCON_PASSWORD env var)
    #[arg(short = 'P', long, env = "FACTORIO_RCON_PASSWORD", default_value = "")]
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

    // Show connection status
    if rcon.is_connected() {
        println!("Connected to Factorio RCON server!");
    } else {
        println!("Running in OFFLINE mode (no Factorio connection)");
        println!("Commands will be simulated for testing purposes.");
    }
    println!("Type :help for help, :quit to exit.\n");

    let mut executor = DslExecutor::new(rcon);

    // Load script if provided
    if let Some(script_path) = args.script {
        load_and_execute_script(&script_path, &mut executor).await;
    }

    // Start REPL
    let mut rl = DefaultEditor::new()?;

    // Load history if it exists
    let history_path = dirs::home_dir().map(|mut p| {
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
                    Ok(cmd) => match executor.execute(cmd).await {
                        Ok(result) => {
                            if !result.is_empty() {
                                println!("{}", result);
                            }
                        }
                        Err(e) => eprintln!("Execution error: {}", e),
                    },
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

async fn load_and_execute_script(script_path: &str, executor: &mut DslExecutor) {
    println!("Loading script: {}", script_path);
    match std::fs::read_to_string(script_path) {
        Ok(content) => match DslParser::parse_script(&content) {
            Ok(commands) => {
                println!("Executing {} commands...", commands.len());
                let mut success_count = 0;
                let mut error_count = 0;

                for cmd in commands {
                    match executor.execute(cmd).await {
                        Ok(result) => {
                            success_count += 1;
                            if !result.is_empty() {
                                println!("  {}", result);
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            eprintln!("  Error: {}", e);
                        }
                    }
                }
                println!(
                    "Script execution complete: {} succeeded, {} failed.\n",
                    success_count, error_count
                );
            }
            Err(e) => eprintln!("Parse error: {}\n", e),
        },
        Err(e) => eprintln!("Failed to load script: {}\n", e),
    }
}

async fn handle_repl_command(line: &str, executor: &mut DslExecutor) -> Result<bool> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts[0] {
        ":help" | ":h" => {
            print_help();
            Ok(false)
        }
        ":quit" | ":q" | ":exit" => Ok(true),
        ":clear" | ":cls" => {
            print!("\x1B[2J\x1B[1;1H");
            Ok(false)
        }
        ":load" => {
            if parts.len() < 2 {
                eprintln!("Usage: :load <file>");
            } else {
                load_and_execute_script(parts[1], executor).await;
            }
            Ok(false)
        }
        ":save" => {
            if parts.len() < 2 {
                eprintln!("Usage: :save <file>");
            } else {
                println!("Save history to file not yet implemented");
            }
            Ok(false)
        }
        ":history" => {
            println!("Command history: {} commands", executor.history_len());
            Ok(false)
        }
        ":status" => {
            if executor.is_connected() {
                println!("Status: Connected to Factorio");
            } else {
                println!("Status: OFFLINE mode (no Factorio connection)");
            }
            Ok(false)
        }
        ":entities" => {
            print_entities();
            Ok(false)
        }
        ":recipes" => {
            print_recipes();
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
    println!("  :status             - Show connection status");
    println!("  :history            - Show command history count");
    println!("  :entities           - List available entity types");
    println!("  :recipes            - List available recipes");
    println!();
    println!("DSL Commands:");
    println!("  <entity> <x> <y> <:dir>              - Place entity");
    println!("  what-at <x> <y>                       - Query position");
    println!("  can-place <entity> <x> <y>            - Check placement");
    println!("  area <x1> <y1> <x2> <y2>              - List entities in area");
    println!("  clear <x1> <y1> <x2> <y2>             - Clear area");
    println!("  undo [count]                          - Undo commands");
    println!("  save-state <:name>                    - Save current state");
    println!("  load-state <:name>                    - Load saved state");
    println!();
    println!("Examples:");
    println!("  belt-blue 10 20 :n");
    println!("  assembler-2 15 15 :n recipe:green-circuit");
    println!("  assembler-3 20 20 :n recipe:blue-circuit module:speed-3");
    println!("  what-at 10 20");
    println!("  clear 0 0 50 50");
    println!();
    println!("Directions: :n :s :e :w :ne :nw :se :sw");
    println!("See DSL-REFERENCE.md for complete syntax reference.");
}

fn print_entities() {
    println!("Available entities:");
    println!();
    println!("  Belts:");
    println!("    belt-yellow, belt-red, belt-blue");
    println!();
    println!("  Underground Belts:");
    println!("    underground-belt-yellow, underground-belt-red, underground-belt-blue");
    println!();
    println!("  Splitters:");
    println!("    splitter-yellow, splitter-red, splitter-blue");
    println!();
    println!("  Inserters:");
    println!("    inserter-burner, inserter-basic, inserter-fast");
    println!("    inserter-long, inserter-stack");
    println!();
    println!("  Assemblers:");
    println!("    assembler-1  (crafting speed: 0.5)");
    println!("    assembler-2  (crafting speed: 0.75, 2 module slots)");
    println!("    assembler-3  (crafting speed: 1.25, 4 module slots)");
    println!();
    println!("  Power Poles:");
    println!("    power-pole-small, power-pole-medium, power-pole-big");
    println!();
    println!("  Fluids:");
    println!("    pipe, pump, storage-tank");
}

fn print_recipes() {
    println!("Available recipes:");
    println!();
    println!("  Basic components:");
    println!("    iron-gear        - 2 iron-plate -> 1 gear (0.5s)");
    println!("    copper-cable     - 1 copper-plate -> 2 cable (0.5s)");
    println!();
    println!("  Circuits:");
    println!("    green-circuit    - 1 iron + 3 cable -> 1 circuit (0.5s)");
    println!("    red-circuit      - 2 green + 2 plastic + 4 cable (6s)");
    println!("    blue-circuit     - 20 green + 2 red + acid (10s)");
    println!();
    println!("  Science packs:");
    println!("    automation-science-pack");
    println!("    logistic-science-pack");
}
