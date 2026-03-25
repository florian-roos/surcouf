use dscale::{SimulationBuilder, Jiffies, LatencyDescription, Distributions, BandwidthDescription};
use dscale::global::kv;
use std::fs::File;
use std::io::Write;
use crate::node::OFCNode;
use crate::orchestrator::Orchestrator;

fn parse_distribution(dist_str: &str) -> Distributions {
    if let Some(rest) = dist_str.strip_prefix("N(") {
        let rest = rest.trim_end_matches(')');
        let parts: Vec<&str> = rest.split(',').collect();
        let mean: usize = parts[0].trim().parse().unwrap();
        let std_dev: usize = parts[1].trim().parse().unwrap();
        Distributions::Normal(Jiffies(mean), Jiffies(std_dev))
    } else if let Some(rest) = dist_str.strip_prefix("B(") {
        let rest = rest.trim_end_matches(')');
        let parts: Vec<&str> = rest.split(',').collect();
        let p: f64 = parts[0].trim().parse().unwrap();
        let val: usize = parts[1].trim().parse().unwrap();
        Distributions::Bernoulli(p, Jiffies(val))
    } else {
        panic!("Unknown distribution format: {}. Use N(mean,std) or B(p,val)", dist_str);
    }
}

pub fn run_simulation(n: usize, f: usize, alpha: f32, t_le: u64, seed: u64, dist_str: &str) -> Option<u64> {
    let mut simulation = SimulationBuilder::default()
        .seed(seed)
        .add_pool::<OFCNode>("Nodes", n)
        .add_pool::<Orchestrator>("Orchestrator", 1)
        .latency_topology(&[
            LatencyDescription::WithinPool("Nodes", parse_distribution(dist_str)),
            LatencyDescription::BetweenPools("Nodes", "Orchestrator", Distributions::Uniform(Jiffies(0), Jiffies(0))),
        ])
        .nic_bandwidth(BandwidthDescription::Bounded(12500))  // 100 Mbps assuming 1 Jiffy = 1 ms.
        .time_budget(Jiffies(1_000_000))
        .build();

    kv::set::<usize>("f", f);
    kv::set::<f32>("alpha", alpha);
    kv::set::<Jiffies>("t_le", Jiffies(t_le as usize));
    kv::set::<Jiffies>("latency", Jiffies(0)); 

    simulation.run(); // Blocks until simulation finishes

    let latency = kv::get::<Jiffies>("latency");
    if latency.0 > 0 { Some(latency.0 as u64) } else { None } // Return None if it deadlocked or ran out of time
}

pub fn run_benchmark(dist_str: &str) {
    let n_values = [3, 10, 30, 70, 100]; //
    let f_values = [1, 4, 14, 34, 49]; 
    let alpha_values = vec![0.0, 0.1, 1.0]; 
    let t_le_values: Vec<u64> = vec![200, 100, 50, 30, 20, 0];
    
    let file_name = format!("results_{}.csv", dist_str);
    let mut file = File::create(&file_name).expect("Unable to create results file");
    
    // Print to both stdout and file
    println!("N, f, Alpha, T_le, Seed, Latency");
    writeln!(file, "N, f, Alpha, T_le, Seed, Latency").unwrap();
    
    for i in 0..n_values.len() {
        let n = n_values[i];
        let f = f_values[i];
        for &alpha in &alpha_values {
            for &t_le in &t_le_values {
                for seed in 1..=5 { // Repeat 5 times with different seeds for better statistics
                    let latency = run_simulation(n, f, alpha, t_le, seed, dist_str);
                    match latency {
                        Some(lat) => {
                            println!("{}, {}, {}, {}, {}, {}", n, f, alpha, t_le, seed, lat);
                            writeln!(file, "{}, {}, {}, {}, {}, {}", n, f, alpha, t_le, seed, lat).unwrap();
                        }
                        None => {
                            println!("{}, {}, {}, {}, {}, DNF", n, f, alpha, t_le, seed); 
                            writeln!(file, "{}, {}, {}, {}, {}, DNF", n, f, alpha, t_le, seed).unwrap();
                        }
                    }
                }
            }
        }
    }
}
