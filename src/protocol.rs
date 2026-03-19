use dscale::{Message, Rank};

#[derive(Debug, Clone, Copy)]
pub enum OFCMessage {
    Read {
        ballot: u64,
    },
    Gather {
        ballot: u64,
        impose_ballot: u64,
        estimate: Option<Value>,
    },
    Abort {
        ballot: u64,
    },
    Ack {
        ballot: u64,
    },
    Impose {
        ballot: u64,
        value: Value,
    },
    Decide {
        value: Value,
    },

    LaunchCmd,
    HoldCmd,
    // Command to order the node to be crash-prone or not (true for crash-prone, false for not crash-prone)
    CrashCmd {
        alpha: f32, // Probability of crash (played at each received message) (0.0 to 1.0)
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Value(bool);

impl Value {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn get(&self) -> bool {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ballot{
    ballot_number: u64,
    process_number: u64,
}

impl Ballot {
    pub fn new(id: Rank, process_number: u64) -> Self {
        Self {
            ballot_number: (id as u64) + 1, 
            process_number,
        }
    }

    pub fn get(&self) -> u64 {
        self.ballot_number
    }

    pub fn increment(&mut self) {
        self.ballot_number += self.process_number;
    }
}

impl Message for OFCMessage {
    fn virtual_size(&self) -> usize {
        64 // Arbitrary fixed size to emulate a realistic message size
    }
}
