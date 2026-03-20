mod protocol;
mod node;
use rand::seq::SliceRandom;
use rand::{SeedableRng, RngExt};
use crate::node::OFCNode;
use dscale::{SimulationBuilder, Jiffies, BandwidthDescription, LatencyDescription, Distributions};
use dscale::global::configuration;
use dscale::global::kv;
use dscale::{ProcessHandle, Rank, MessagePtr, TimerId};
use dscale::{rank, send_to, broadcast, debug_process, now, schedule_timer_after};
use crate::protocol::{OFCMessage, Value};

fn main() {
    let n: usize = 3; // Number of nodes in the simulation
    let mut simulation = SimulationBuilder::default()
        .add_pool::<OFCNode>("Nodes", n)
        .add_pool::<Orchestrator>("Orchestrator", 1)
        .latency_topology(&[
            LatencyDescription::WithinPool("Nodes", Distributions::Uniform(Jiffies(1), Jiffies(5))),
            LatencyDescription::BetweenPools("Nodes", "Orchestrator", Distributions::Uniform(Jiffies(1), Jiffies(2))),
        ])
        .time_budget(Jiffies(1_000))
        .build();
    simulation.run();
}

#[derive(Default)]
struct Orchestrator{
    n: usize,
    f: usize,
    rng: Option<rand::rngs::StdRng>,
    hold_timer_id: Option<TimerId>,
    kv_checker_timer_id: Option<TimerId>,
}

impl ProcessHandle for Orchestrator {
    fn start(&mut self) {
        self.n = configuration::process_number() - 1; // Exclude the orchestrator itself
        self.f = 1;
        let alpha = 0.5;

        kv::set::<usize>("decieded_processes", 0);
        self.rng = Some(rand::rngs::StdRng::seed_from_u64(configuration::seed()));
        let mut node_ranks: Vec<Rank> = (0..self.n as Rank).collect();

        if let Some(ref mut rng) = self.rng{
            node_ranks.shuffle(rng);
        }

        let crash_prone_nodes = &node_ranks[0..self.f];
        for &rank in crash_prone_nodes {
            send_to(rank, OFCMessage::CrashCmd {alpha});
        }

        for i in 0..self.n {
            send_to(i as Rank, OFCMessage::LaunchCmd);
        }

        self.hold_timer_id = Some(schedule_timer_after(Jiffies(30)));
        self.kv_checker_timer_id = Some(schedule_timer_after(Jiffies(5)));
    }

    fn on_message(&mut self, _from: Rank, _message: MessagePtr) {
        // No message received by the orchestrator
    }

    fn on_timer(&mut self, id: TimerId) {
        if id == self.hold_timer_id.unwrap() {
            let leader = 0;              

            for i in 0..self.n {
                if i != leader {
                    send_to(i as Rank, OFCMessage::HoldCmd)
                }
            }
        } else {
            let decided_processes = kv::get::<usize>("decieded_processes");
        if decided_processes >= self.n - self.f {
            debug_process!("All non-crashed nodes have decided at {}", now());
        } else {
            self.kv_checker_timer_id = Some(schedule_timer_after(Jiffies(5)));
            debug_process!("No decision. Decided processes: {}/{}", decided_processes, self.n - self.f);
        }
    }
    }
}
