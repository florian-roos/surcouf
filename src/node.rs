use std::collections::HashMap;
use rand::{SeedableRng, RngExt};
use dscale::{ProcessHandle, Rank, MessagePtr, TimerId};
use dscale::{rank, send_to, broadcast, debug_process, now};
use dscale::global::configuration;
use dscale::global::kv;
use crate::protocol::{OFCMessage, Value};

struct ProposerState {
    proposal: Option<Value>,
    ballot: u64,
    gathered_states: HashMap<Rank, (u64, Option<Value>)>, 
    ack_count: usize,
}

struct AcceptorState {
    read_ballot: u64,
    impose_ballot: u64,
    estimate: Option<Value>,
}

pub struct OFCNode {
    id: Rank,
    acceptor_state: AcceptorState,
    proposer_state: Option<ProposerState>,
    is_crashed: bool,
    is_holding: bool,
    has_decided: bool,
    alpha: f32, // Probability of crash (played at each received message) (0.0 to 1.0)
    rng: Option<rand::rngs::StdRng>,
}

impl ProcessHandle for OFCNode {
    fn start(&mut self) {
        // Initialize the node's state
        self.id = rank();
        self.acceptor_state = AcceptorState {
            read_ballot: 0,
            impose_ballot: 0,
            estimate: None,
        };
        self.proposer_state = None;
        self.is_crashed = false;
        self.is_holding = false;
        self.has_decided = false;
        self.alpha = 0.0;
        self.rng = Some(rand::rngs::StdRng::seed_from_u64(configuration::seed()));
        let _number_of_processes: usize = configuration::process_number() - 1; // Exclude the orchestrator
        debug_process!("Node {} started", self.id);
    }

    fn on_message(&mut self, from: Rank, message: MessagePtr) {
        // Simulate crash based on alpha probability
        if self.is_crashed || self.has_decided || self.crash_simulation() {
            return 
        }

        if let Some(msg) = message.try_as::<OFCMessage>() {
            match *msg {
                OFCMessage::Read { ballot } => self.handle_read(from, ballot),
                OFCMessage::Impose { ballot, value } => self.handle_impose(from, ballot, value),
                OFCMessage::Abort { ballot } => self.handle_abort(ballot),
                OFCMessage::Gather { ballot, impose_ballot, estimate } => self.handle_gather(from, ballot, impose_ballot, estimate),
                OFCMessage::Ack { ballot } => self.handle_ack(ballot),
                OFCMessage::Decide { value } => self.handle_decide(from, value),
                OFCMessage::LaunchCmd => self.handle_launch(),
                OFCMessage::HoldCmd => self.handle_hold(),
                OFCMessage::CrashCmd { alpha } => self.handle_crash(alpha),
            }
        }
    }

    fn on_timer(&mut self, _id: TimerId) {
        // No timer-based actions for now
    }
}

impl OFCNode {
    fn handle_read(&mut self, from: Rank, ballot: u64) {
        debug_process!("Node {} received Read with ballot {}", self.id, ballot);
        if self.acceptor_state.read_ballot > ballot || self.acceptor_state.impose_ballot > ballot {
            send_to(from, OFCMessage::Abort { ballot });
        } else {
            self.acceptor_state.read_ballot = ballot;
            send_to(from, OFCMessage::Gather {
                ballot,
                impose_ballot: self.acceptor_state.impose_ballot,
                estimate: self.acceptor_state.estimate,
            });
        }
    }

    fn handle_impose(&mut self, from: Rank, ballot: u64, value: Value) {
        debug_process!("Node {} received Impose with ballot {}, value {:?}", self.id, ballot, value);
        if self.acceptor_state.read_ballot > ballot || self.acceptor_state.impose_ballot > ballot {
            send_to(from, OFCMessage::Abort { ballot });
        } else {
            self.acceptor_state.impose_ballot = ballot;
            self.acceptor_state.estimate = Some(value);
            send_to(from, OFCMessage::Ack { ballot });
        }
    }

    fn handle_abort(&mut self, ballot: u64) {
        debug_process!("Node {} received Abort with ballot {}", self.id, ballot);
        if let Some(ref mut proposer_state) = self.proposer_state
            && ballot == proposer_state.ballot && !self.is_holding {
                let current_prop = proposer_state.proposal;
                self.proposer_state = Some(ProposerState {
                    proposal: current_prop,
                    ballot: ballot + (configuration::process_number() - 1) as u64,
                    gathered_states: HashMap::new(),
                    ack_count: 0,
                });
                broadcast(OFCMessage::Read { ballot: self.proposer_state.as_ref().unwrap().ballot });
            }
    }

    fn handle_gather(&mut self, from: Rank, ballot: u64, impose_ballot: u64, estimate: Option<Value>) {
        debug_process!("Node {} received Gather with ballot {}, impose_ballot {}, estimate {:?}", self.id, ballot, impose_ballot, estimate);
        if let Some(ref mut proposer_state) = self.proposer_state
            && ballot == proposer_state.ballot {
                proposer_state.gathered_states.insert(from, (impose_ballot, estimate));
                if proposer_state.gathered_states.len() > (configuration::process_number() - 1) / 2 {
                    let mut highest_impose_ballot = 0;
                    let mut highest_estimate = None;
                    for &(impose_bal, est) in proposer_state.gathered_states.values() {
                        if impose_bal > highest_impose_ballot {
                            highest_impose_ballot = impose_bal;
                            highest_estimate = est;
                        }
                    }
                    let value_to_propose = highest_estimate.unwrap_or(proposer_state.proposal.unwrap());
                    for _i in 0..(configuration::process_number() - 1) {
                        broadcast(OFCMessage::Impose { ballot, value: value_to_propose });
                    }
                }
            }
    }

    fn handle_ack(&mut self, ballot: u64) {
        debug_process!("Node {} received Ack with ballot {}", self.id, ballot);
        if let Some(ref mut proposer_state) = self.proposer_state
            && ballot == proposer_state.ballot {
                proposer_state.ack_count += 1;
                if proposer_state.ack_count == (configuration::process_number() - 1) / 2{
                    debug_process!("Node {} broadcasted decide after Ack", self.id);
                    broadcast(OFCMessage::Decide { value: proposer_state.proposal.unwrap() });
                }
            }
    }

    fn handle_decide(&mut self, from: Rank, value: Value) {
        debug_process!("Node {} received Decide from {} with value {:?}", self.id, from, value);
        broadcast(OFCMessage::Decide { value });
        kv::modify::<usize>("decided_processes", |x| *x += 1);
        self.has_decided = true;
    }

    fn handle_launch(&mut self) {
        debug_process!("Node {} received LaunchCmd", self.id);
        if !self.is_holding {
            let random_bool = self.rng.as_mut().unwrap().random::<bool>();

            self.proposer_state = Some(ProposerState {
                proposal: Some(Value::new(random_bool)),
                ballot: self.id as u64 + 1,
                gathered_states: HashMap::new(),
                ack_count: 0,
            });
            broadcast(OFCMessage::Read { ballot: self.proposer_state.as_ref().unwrap().ballot });
        }
    }

    fn handle_hold(&mut self) {
        debug_process!("Node {} received HoldCmd", self.id);
        self.is_holding = true;
    }

    fn handle_crash(&mut self, alpha: f32) {
        debug_process!("Node {} received CrashCmd with alpha {}", self.id, alpha);
        self.alpha = alpha;
    }

    fn crash_simulation(&mut self) -> bool {
        if self.alpha > 0.0
            && let Some(ref mut rng) = self.rng
                && rng.random::<f32>() < self.alpha {
                    self.is_crashed = true;
                    debug_process!("Node {} has crashed", self.id);
                    return true;
                }
        false
    }
}

impl Default for OFCNode {
    fn default() -> Self {
        Self {
            id: 0, // Will be overwritten in start()
            acceptor_state: AcceptorState {
                read_ballot: 0,
                impose_ballot: 0,
                estimate: None,
            },
            proposer_state: None,
            is_crashed: false,
            is_holding: false,
            has_decided: false,
            alpha: 0.0,
            rng: None,
        }
    }
}
