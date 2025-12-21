// Solver Binary
// Command-line constraint solver that outputs DSL scripts

use anyhow::Result;
use clap::Parser;
use factorio_constraint_builder::solver::{
    BeltType, FactorySolver, ProductionGoal, Throughput,
};

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

    /// Show calculation details
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Factorio Constraint Solver");
    println!("==========================\n");
    println!("Item: {}", args.item);
    println!("Throughput: {}", args.throughput);

    // Parse throughput
    let throughput = parse_throughput(&args.throughput)?;

    let goal = ProductionGoal {
        item: args.item.clone(),
        throughput,
    };

    println!("Target: {:.2} items/second\n", goal.items_per_second());

    // Create solver
    let solver = FactorySolver::new();

    // Show calculation details if verbose
    if args.verbose {
        if let Some(reqs) = solver.calculate_requirements(&goal) {
            println!("Machine Requirements:");
            print_requirements(&reqs, 0);
            println!();
        }
    }

    // Solve for placement
    let solution = solver.solve(&goal)?;

    println!("Solution found!");
    println!("{}\n", solution.summary());

    // Generate DSL script
    let script = solution.to_script();

    // Output DSL script
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &script)?;
        println!("DSL script written to: {}", output_path);
    } else {
        println!("DSL Script:");
        println!("-----------");
        println!("{}", script);
    }

    Ok(())
}

fn print_requirements(req: &factorio_constraint_builder::solver::MachineRequirements, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!(
        "{}{}: {:.2}/sec -> {:.1} x {}",
        prefix, req.item, req.items_per_second, req.machine_count, req.machine_type
    );
    for dep in &req.dependencies {
        print_requirements(dep, indent + 1);
    }
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

    Err(anyhow::anyhow!("Invalid throughput format. Use: N/s, N/m, or N-color-belt"))
}
