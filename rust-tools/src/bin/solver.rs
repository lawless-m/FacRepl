// Solver Binary
// Command-line constraint solver that outputs DSL scripts

use factorio_constraint_builder::solver::{ProductionGoal, Throughput, BeltType, ConstraintSolver, PrologSolver};
use factorio_constraint_builder::dsl::Command;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "fcb-solver")]
#[command(about = "Constraint solver for Factorio factory layouts", long_about = None)]
struct Args {
    /// Item to produce
    #[arg(short, long)]
    item: String,

    /// Throughput specification (e.g., "1-blue-belt", "45/s", "2700/m")
    #[arg(short, long)]
    throughput: String,
    
    /// Output file for DSL script
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("Factorio Constraint Solver");
    println!("Item: {}", args.item);
    println!("Throughput: {}", args.throughput);
    
    // Parse throughput
    let throughput = parse_throughput(&args.throughput)?;
    
    let goal = ProductionGoal {
        item: args.item,
        throughput,
    };
    
    println!("Target: {:.2} items/second", goal.items_per_second());
    
    // Solve using Prolog
    let prolog_file = std::env::var("PROLOG_FILE")
        .unwrap_or_else(|_| "prolog/factorio.pl".to_string());

    let solver = PrologSolver::new(prolog_file);
    let solution = solver.solve(goal)?;
    
    println!("\nSolution found!");
    println!("Commands: {}", solution.commands.len());
    println!("Machines: {:?}", solution.statistics.machine_count);
    
    // Output DSL script
    if let Some(output_path) = args.output {
        println!("\nWriting DSL script to: {}", output_path);
        let dsl_text = format_commands_as_dsl(&solution.commands);
        std::fs::write(&output_path, dsl_text)?;
        println!("Wrote {} commands to {}", solution.commands.len(), output_path);
    } else {
        println!("\nDSL Script:");
        for cmd in &solution.commands {
            println!("{}", format_command_as_dsl(cmd));
        }
    }
    
    Ok(())
}

fn format_command_as_dsl(cmd: &Command) -> String {
    match cmd {
        Command::Place(place) => {
            let mut parts = vec![
                place.entity_type.clone(),
                place.position.x.to_string(),
                place.position.y.to_string(),
            ];

            if let Some(dir) = &place.direction {
                parts.push(format!(":{}", dir.to_keyword()));
            }

            if let Some(recipe) = &place.recipe {
                parts.push(format!("recipe:{}", recipe));
            }

            for module in &place.modules {
                parts.push(format!("module:{}", module));
            }

            if let Some(priority) = &place.input_priority {
                parts.push(format!("in-priority:{}", priority.to_keyword()));
            }

            if let Some(priority) = &place.output_priority {
                parts.push(format!("out-priority:{}", priority.to_keyword()));
            }

            if let Some(filter) = &place.filter {
                parts.push(format!("filter:{}", filter));
            }

            parts.join(" ")
        }
        Command::Query(_) => "; Query command not supported in scripts".to_string(),
        Command::Clear(_) => "; Clear command".to_string(),
        Command::Comment(s) => format!("; {}", s),
        _ => "; Unknown command".to_string(),
    }
}

fn format_commands_as_dsl(commands: &[Command]) -> String {
    commands.iter()
        .map(format_command_as_dsl)
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_throughput(s: &str) -> Result<Throughput> {
    // Parse formats like:
    // "1-blue-belt", "2-red-belt", "0.5-yellow-belt"
    // "45/s", "2700/m"
    
    if s.contains("belt") {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() >= 2 {
            let belt_type = match parts[1] {
                "yellow" => BeltType::Yellow,
                "red" => BeltType::Red,
                "blue" => BeltType::Blue,
                _ => return Err(anyhow::anyhow!("Invalid belt type")),
            };
            return Ok(Throughput::BeltFull(belt_type));
        }
    }
    
    if s.ends_with("/s") {
        let rate: f64 = s.trim_end_matches("/s").parse()?;
        return Ok(Throughput::ItemsPerSecond(rate));
    }
    
    if s.ends_with("/m") {
        let rate: f64 = s.trim_end_matches("/m").parse()?;
        return Ok(Throughput::ItemsPerMinute(rate));
    }
    
    Err(anyhow::anyhow!("Invalid throughput format"))
}
