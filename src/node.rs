use std::collections::HashMap;
use dscale::{ProcessHandle, Rank, MessagePtr, TimerId};
use dscale::{rank, send_to, broadcast, debug_process, now};
use dscale::global::configuration;
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

struct OFCNode {
    id: Rank,
    acceptor_state: AcceptorState,
    proposer_state: Option<ProposerState>,
    is_crashed: bool,
    alpha: f32, // Probability of crash (played at each received message) (0.0 to 1.0)
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
        self.alpha = 0.0;
        let number_of_processes: usize = configuration::process_number();
        debug_process!("Node {} started", self.id);
    }

    fn on_message(&mut self, from: Rank, message: MessagePtr) {
        // Simulate crash based on alpha probability
        if self.is_crashed{ // || crash_simulation(&mut self) further implemented{
            return 
        }

        if let Some(msg) = message.try_as::<OFCMessage>() {
            match *msg {
                OFCMessage::Read { ballot } => {
                    debug_process!("Node {} received Read with ballot {}", self.id, ballot);
                    if self.acceptor_state.read_ballot > ballot || self.acceptor_state.impose_ballot > ballot {
                        send_to(from, OFCMessage::Abort{ballot})
                    } else {
                        self.acceptor_state.read_ballot = ballot;
                        send_to(from, OFCMessage::Gather{
                                                            ballot,
                                                            impose_ballot: self.acceptor_state.impose_ballot,
                                                            estimate: self.acceptor_state.estimate});
                    }
                }
                OFCMessage::Impose { ballot, value } => {
                    debug_process!("Node {} received Impose with ballot {}, value {:?}", self.id, ballot, value);
                    if self.acceptor_state.read_ballot > ballot || self.acceptor_state.impose_ballot > ballot {
                        send_to(from, OFCMessage::Abort{ballot})
                    } else {
                        self.acceptor_state.impose_ballot = ballot;
                        self.acceptor_state.estimate = Some(value);
                        send_to(from, OFCMessage::Ack{ballot})
                    }
                }
                OFCMessage::Abort{ballot} => {
                    //retry with higher ballot
                }
                OFCMessage::Gather{ballot, impose_ballot, estimate} => {
                    debug_process!("Node {} received Gather with ballot {}, impose_ballot {}, estimate {:?}", self.id, ballot, impose_ballot, estimate);
                    if let Some(ref mut proposer_state) = self.proposer_state {
                         if ballot == self.proposer_state.ballot {
                            self.proposer_state.gathered_states.insert(from, (impose_ballot, estimate));
                            if self.proposer_state.gathered_states.len() > configuration::process_number() / 2 {
                                let mut highest_impose_ballot = 0;
                                let mut highest_estimate = None;
                                for &(_, (impose_ballot, estimate)) in &self.proposer_state.gathered_states {
                                    if impose_ballot > highest_impose_ballot {
                                        highest_impose_ballot = impose_ballot;
                                        highest_estimate = estimate;
                                    }
                                }
                                let value_to_propose = highest_estimate.unwrap_or(self.proposer_state.proposal.unwrap());
                                for i in 0..number_of_processes {
                                    broadcast(OFCMessage::Impose{ballot: highest_impose_ballot, value: value_to_propose});
                                }
                            }
                    }
                }
                OFCMessage::Ack{ballot} => {
                    debug_process!("Node {} received Ack with ballot {}", self.id, ballot);
                    if let Some(ref mut proposer_state) = self.proposer_state {
                        if ballot == proposer_state.ballot {
                            proposer_state.ack_count += 1;
                            if proposer_state.ack_count > configuration::process_number() / 2 {
                                broadcast(OFCMessage::Decide{value: proposer_state.proposal.unwrap()});
                            }
                        }
                    }
                }
                OFCMessage::Decide{value} => {}
                OFCMessage::LaunchCmd => {}
                OFCMessage::HoldCmd => {}
                OFCMessage::CrashCmd{alpha} => {}
            }
        }
    }

    fn on_timer(&mut self, _id: TimerId) {
        // No timer-based actions for now
    }
}


