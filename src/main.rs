use std::env;

mod protocol;
mod node;
mod orchestrator;
mod benchmark;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <distribution>", args[0]);
        eprintln!("Example: {} 'N(10,1)' or 'B(0.9,20)'", args[0]);
        std::process::exit(1);
    }

    let dist_str = &args[1];
    println!("Running benchmark with network latency distribution: {}", dist_str);
    benchmark::run_benchmark(dist_str);
}

