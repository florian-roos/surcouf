use rand::seq::SliceRandom;
use rand::{SeedableRng};
use dscale::{Jiffies};
use dscale::global::configuration;
use dscale::global::kv;
use dscale::{ProcessHandle, Rank, MessagePtr, TimerId};
use dscale::{send_to,now, schedule_timer_after};
use crate::protocol::{OFCMessage};

#[derive(Default)]
pub struct Orchestrator{
    n: usize,
    f: usize,
    rng: Option<rand::rngs::StdRng>,
    hold_timer_id: Option<TimerId>,
    kv_checker_timer_id: Option<TimerId>,
}

impl ProcessHandle for Orchestrator {
    fn start(&mut self) {
        self.n = configuration::process_number() - 1; // Exclude the orchestrator itself
        self.f = kv::get::<usize>("f"); // Number of crash-prone nodes
        let alpha = kv::get::<f32>("alpha"); // Crash probability for crash-prone nodes
        let t_le = kv::get::<Jiffies>("t_le"); // Time of leader election
        kv::set::<usize>("decided_processes", 0);

        // Shuffle the pool of nodes to randomly select f crash-prone nodes
        self.rng = Some(rand::rngs::StdRng::seed_from_u64(configuration::seed()));
        let mut node_ranks: Vec<Rank> = (0..self.n as Rank).collect();
        if let Some(ref mut rng) = self.rng{
            node_ranks.shuffle(rng);
        }

        // Send crash commands to the selected crash-prone nodes
        let crash_prone_nodes = &node_ranks[0..self.f];
        for &rank in crash_prone_nodes {
            send_to(rank, OFCMessage::CrashCmd {alpha});
        }

        for i in 0..self.n {
            send_to(i as Rank, OFCMessage::LaunchCmd);
        }

        self.hold_timer_id = Some(schedule_timer_after(t_le));
        self.kv_checker_timer_id = Some(schedule_timer_after(Jiffies(10))); // Timer to check for decisions
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
            let decided_processes = kv::get::<usize>("decided_processes");
        if decided_processes >= self.n - self.f {
            kv::set::<Jiffies>("latency", now());
        } else {
            self.kv_checker_timer_id = Some(schedule_timer_after(Jiffies(10)));
        }
    }
    }
}