pub enum ServerState {
    Idle,
    Greeted,
    Questioned,
    Terminated,
}

pub enum ServerError {
    InvalidPayloadLength, 
    UnrecognizedHeader,
    UnrecognizedPayload,
    HeaderMismatch,
    NonUTF8Sequence,
    InsufficientMessageSize,
    InvalidStateTransition,
}


pub struct StateMachine {
    pub current_state: ServerState,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: ServerState::Idle,
        }
    }
}