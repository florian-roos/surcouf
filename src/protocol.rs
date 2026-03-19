use dscale::{Message};

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
}

impl Message for OFCMessage {
    fn virtual_size(&self) -> usize {
        64 // Arbitrary fixed size to emulate a realistic message size
    }
}
