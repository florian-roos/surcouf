use dscale::{ProcessHandle, Rank, rank, MessagePtr, send_to, debug_process};
use protocol::{OFCMessage, Value};

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

impl dscale::ProcessHandle for OFCNode {
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

        dscale::debug_process!("Node {} started", self.id);
    }
}

