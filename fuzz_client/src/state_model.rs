use std::collections::HashMap;
use std::cmp::PartialEq;

use crate::Protocol;
use crate::StateTransition;
use crate::Message;

// StateModel is a struct which defines a HashMap that maps ServerStates to a vector of StateTransitions
// This captures the relationship between the ServerStates and the possible transitions from anyone state.
//
// 'T' is a type paramenter which refers to a type that implements the Protocol trait. The pariculat protocol
// which has an implementation of the Protocol trait, also has definitions of the associated types in the 
// Protocol trait. The one in use here being ServerState.
pub struct StateModel<T: Protocol> {
    inner: HashMap<T::ServerState, Vec<StateTransition<T::ServerState, T>>>,
}

impl<T: Protocol + PartialEq> StateModel<T> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    // This method is used to add a StateTransition to the StateModel
    pub fn add(&mut self, current_state: T::ServerState, next_state: T::ServerState, message: &Message<T>) {
    	// This line retreives the transitions associated with the current_state
    	// If there are no transistions associated to current_state, a Vec::new() 
    	// is inserted in its place
        let transitions = self
            .inner
            .entry(current_state.clone())
            .or_insert_with(Vec::new);

        let new_transition = StateTransition {
            current_state: current_state.clone(),
            next_state: next_state.clone(),
            message: message.clone(),
        };

        // Checks if there is already a transition in the transitions vector that has
        // the same next_state, current_state, and message as new_transition. If there is
        // no duplicate transition, then new_transition is added to the transitions vector
        if !transitions.iter().any(|t| {
            t.next_state == next_state && t.current_state == current_state && t.message == *message
        }) {
            transitions.push(new_transition);
        }
    }
}
