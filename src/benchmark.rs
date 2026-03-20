use dscale::{SimulationBuilder, Jiffies, LatencyDescription, Distributions};
use dscale::global::kv;
use crate::node::OFCNode;
use crate::orchestrator::Orchestrator;

pub fn run_simulation(n: usize, f: usize, alpha: f32, tle: u64, seed: u64) -> Option<u64> {
    let mut simulation = SimulationBuilder::default()
        .seed(seed)
        .add_pool::<OFCNode>("Nodes", n)
        .add_pool::<Orchestrator>("Orchestrator", 1)
        .latency_topology(&[
            LatencyDescription::WithinPool("Nodes", Distributions::Normal(Jiffies(10), Jiffies(3))),
            LatencyDescription::BetweenPools("Nodes", "Orchestrator", Distributions::Uniform(Jiffies(0), Jiffies(0))),
        ])
        .time_budget(Jiffies(1_000_000))
        .build();

    kv::set::<usize>("param_f", f);
    kv::set::<f32>("param_alpha", alpha);
    kv::set::<u64>("param_tle", tle);
    kv::set::<u64>("result_latency", 0); // Initialize result

    simulation.run(); // Blocks until simulation finishes

    let latency = kv::get::<u64>("result_latency");
    if latency > 0 { Some(latency) } else { None } // Return None if it deadlocked or ran out of time
}

pub fn run_benchmark() {
    let n_values = [3, 10, 30, 70, 100]; //
    let f_values = [1, 4, 14, 34, 49]; 
    let alpha_values = vec![0.0, 0.1, 1.0]; 
    let tle_values = vec![200, 100, 50, 30, 20, 10];
    
    println!("N, f, Alpha, Tle, Seed, Latency");
    
    for i in 0..n_values.len() {
        let n = n_values[i];
        let f = f_values[i];
        for &alpha in &alpha_values {
            for &tle in &tle_values {
                for seed in 1..=5 { // Repeat 5 times with different seeds for better statistics
                    let latency = run_simulation(n, f, alpha, tle, seed);
                    match latency {
                        Some(lat) => println!("{}, {}, {}, {}, {}, {}", n, f, alpha, tle, seed, lat),
                        None => println!("{}, {}, {}, {}, {}, Did not finish", n, f, alpha, tle, seed), 
                    }
                }
            }
        }
    }
}