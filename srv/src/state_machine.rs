#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

use crate::Message;
use crate::MessageType;
use crate::Response;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerState {
    Initial,
    Greeted,
    Questioned,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    InvalidPayloadLength, 
    UnrecognizedHeader,
    UnrecognizedPayload,
    HeaderMismatch,
    NonUTF8Sequence,
    InsufficientMessageSize,
    InvalidStateTransition,
}


#[derive(Debug, Clone)]
pub struct StateTransitionRules {
    rules: HashMap<(ServerState, MessageType), ServerState>,
}

impl StateTransitionRules {
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        // Add rules for state transitions based on message types
        rules.insert((ServerState::Initial, MessageType::Hello), ServerState::Greeted);
        rules.insert((ServerState::Greeted, MessageType::TimeRequest), ServerState::Questioned);
        rules.insert((ServerState::Questioned, MessageType::Goodbye), ServerState::Terminated);

        // Add a few "secret" rules
        rules.insert((ServerState::Initial, MessageType::TimeRequest), ServerState::Terminated);
        rules.insert((ServerState::Greeted, MessageType::Goodbye), ServerState::Initial);

        StateTransitionRules { rules }
    }

    pub fn get_next_state(&self, current_state: &ServerState, message_type: &MessageType) -> Option<ServerState> {
        self.rules.get(&(current_state.clone(), message_type.clone())).cloned()
    }
}

pub struct StateMachine {
    pub current_state: ServerState,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: ServerState::Initial,
        }
    }

    pub fn handle_message(&mut self, message: &Message, transition_rules: &StateTransitionRules) -> Response {
        Response::new(message, &self.current_state, transition_rules)
    }
}