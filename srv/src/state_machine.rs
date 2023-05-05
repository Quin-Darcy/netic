#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::Message;
use crate::MessageType;
use crate::Response;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerState {
    Initial,
    Greeted,
    Questioned,
    Terminated,
    Secret1,
    Secret2,
    Secret3,
    Secret4,
    Secret5,
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
    None,
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
        rules.insert((ServerState::Terminated, MessageType::TimeRequest), ServerState::Greeted);

        rules.insert((ServerState::Secret1, MessageType::Hello), ServerState::Greeted);        
        rules.insert((ServerState::Secret1, MessageType::TimeRequest), ServerState::Questioned);
        rules.insert((ServerState::Secret1, MessageType::Goodbye), ServerState::Terminated);

        rules.insert((ServerState::Secret2, MessageType::Hello), ServerState::Greeted);
        rules.insert((ServerState::Secret2, MessageType::TimeRequest), ServerState::Questioned);
        rules.insert((ServerState::Secret2, MessageType::Goodbye), ServerState::Terminated);
        
        rules.insert((ServerState::Secret3, MessageType::Hello), ServerState::Greeted);
        rules.insert((ServerState::Secret3, MessageType::TimeRequest), ServerState::Questioned);
        rules.insert((ServerState::Secret3, MessageType::Goodbye), ServerState::Terminated);

        rules.insert((ServerState::Secret4, MessageType::Hello), ServerState::Greeted);
        rules.insert((ServerState::Secret4, MessageType::TimeRequest), ServerState::Questioned);
        rules.insert((ServerState::Secret4, MessageType::Goodbye), ServerState::Terminated);

        rules.insert((ServerState::Secret4, MessageType::Hello), ServerState::Greeted);
        rules.insert((ServerState::Secret4, MessageType::TimeRequest), ServerState::Questioned);
        rules.insert((ServerState::Secret4, MessageType::Goodbye), ServerState::Terminated);

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
        let (response, new_state) = Response::new(message, &self.current_state, transition_rules);

        let mut sleep_duration: Duration = Duration::from_secs(0);

        if !message.parsing_results.is_empty() {
            if self.current_state == ServerState::Questioned && message.parsing_results[0].clone() == ServerError::InvalidPayloadLength {
                println!("---------------SECRET1---------------");
                self.current_state = ServerState::Secret1;
                sleep_duration = Duration::from_secs(1);

            }

            if self.current_state == ServerState::Secret1 && message.parsing_results[0].clone() == ServerError::UnrecognizedPayload {
                println!("---------------SECRET2---------------");
                self.current_state = ServerState::Secret2;
                sleep_duration = Duration::from_secs(2);
            }

            if self.current_state == ServerState::Secret2 && message.parsing_results[0].clone() == ServerError::HeaderMismatch {
                println!("---------------SECRET3---------------");
                self.current_state = ServerState::Secret3;
                sleep_duration = Duration::from_secs(3);
            }

            if self.current_state == ServerState::Secret3 && message.parsing_results[0].clone() == ServerError::InvalidPayloadLength {
                println!("---------------SECRET4---------------");
                self.current_state = ServerState::Secret4;
                sleep_duration = Duration::from_secs(4);
            }

            if self.current_state == ServerState::Secret4 && message.parsing_results[0].clone() == ServerError::NonUTF8Sequence {
                println!("---------------SECRET4---------------");
                self.current_state = ServerState::Secret5;
                sleep_duration = Duration::from_secs(5);
            }
        } else {
            self.current_state = new_state;
        }

        thread::sleep(sleep_duration);
        response
    }
}