use std::collections::HashMap;
use std::cmp::PartialEq;
use std::fmt::Write;
use std::fmt::Debug;

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
    pub inner: HashMap<T::ServerState, Vec<StateTransition<T::ServerState, T>>>,
}

impl<T: Protocol + PartialEq> StateModel<T> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    // This method is used to add a StateTransition to the StateModel
    pub fn add(&mut self, source_state: T::ServerState, target_state: T::ServerState, message: &Message<T>) {
    	// This line retreives the transitions associated with the current_state
    	// If there are no transistions associated to current_state, a Vec::new() 
    	// is inserted in its place
        let transitions = self
            .inner
            .entry(source_state.clone())
            .or_insert_with(Vec::new);

        let new_transition = StateTransition {
            source_state: source_state.clone(),
            target_state: target_state.clone(),
            message: message.clone(),
        };

        // Checks if there is already a transition in the transitions vector that has
        // the same next_state, current_state, and message as new_transition. If there is
        // no duplicate transition, then new_transition is added to the transitions vector
        if !transitions.iter().any(|t| {
            t.target_state == target_state && t.source_state == source_state && t.message == *message
        }) {
            transitions.push(new_transition);
        }
    }

    // Returns the number of unique ServerStates visited
    pub fn count_unique_server_states(&self) -> usize {
        self.inner.len()
    }

    pub fn to_dot_string(&self) -> String
    where
        <T as Protocol>::ServerState: Debug,
        <T as Protocol>::MessageType: Debug,
        <T as Protocol>::MessageSectionsKey: Debug,
        <T as Protocol>::MessageSectionsValue: Debug,
    {
        let mut dot_string = String::new();
        writeln!(&mut dot_string, "digraph state_graph {{").unwrap();

        for (_source_state, transitions) in &self.inner {
            for transition in transitions {
                let source_label = escape_label(&format!("{:?}", transition.source_state));
                let target_label = escape_label(&format!("{:?}", transition.target_state));

                writeln!(&mut dot_string, r#"    "{}" -> "{}""#, source_label, target_label).unwrap();
            }
        }

        writeln!(&mut dot_string, "}}").unwrap();
        dot_string
    }
}

fn escape_label(label: &str) -> String {
    label.replace("\"", "\\\"").replace("\n", "")
}
