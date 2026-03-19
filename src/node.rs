use std::collections::HashMap;
use dscale::{ProcessHandle, Rank, MessagePtr, TimerId};
use dscale::{rank, send_to,  debug_process, now};
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
                OFCMessage::Abort{ballot} => {}
                OFCMessage::Gather{ballot, impose_ballot, estimate} => {}
                OFCMessage::Ack{ballot} => {}
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


