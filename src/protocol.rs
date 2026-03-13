pub enum OFCMessage {
    Read {
        ballot: u64,
    },
    Gather {
        ballot: u64,
        impose_ballot: u64,
        estimate: Value,
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
    CrashCmd {
        probability: f32, // Probability of crash (played at each received message) (0.0 to 1.0)
    },
}

pub struct Value(bool);

impl Value {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn get(&self) -> bool {
        self.0
    }
}
