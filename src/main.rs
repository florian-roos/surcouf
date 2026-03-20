mod protocol;
mod node;
mod orchestrator;
mod benchmark;

fn main() {
    println!("Starting Surcouf OFC Benchmarks...");
    benchmark::run_benchmark();
}

