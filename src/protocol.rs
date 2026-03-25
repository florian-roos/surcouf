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
        // Size in bytes used for bandwidth simulation.
        match self {
            OFCMessage::Read { .. }   =>   64,  // just a ballot number
            OFCMessage::Gather { .. } => 8192,  // carries the actual value (8 KB)
            OFCMessage::Impose { .. } => 8192,  // carries the value being imposed
            OFCMessage::Ack { .. }    =>   64,  // just a ballot number
            OFCMessage::Decide { .. } => 8192,  // carries the decided value
            OFCMessage::Abort { .. }  =>   64,  // just a ballot number
            OFCMessage::LaunchCmd => 0, // Command from the orchestrator, no payload
            OFCMessage::HoldCmd => 0, // Command from the orchestrator, no payload
            OFCMessage::CrashCmd { .. } => 0, // Command from the orchestrator, no payload
        }   
    }
}
