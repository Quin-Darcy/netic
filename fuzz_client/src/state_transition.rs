use crate::Protocol;
use crate::Message;

// General purpose structure which will be used by all protocols
//
// The 'S' and 'P' here are type parameters. They are used to create a generic
// structure that can work with different types. The 'S' represents the type
// of server state, while 'P' represents a type that implements the Protocol trait.
//
// The 'P: Protocol' constraint enforces that the type 'P' must implement the 
// Protocol trait. This ensures that the Message<P> field must be a message that 
// belongs to a protocol that implements the Protocol trait.  
pub struct StateTransition<S, P: Protocol> {
	pub source_state: S,
	pub message: Message<P>,
	pub target_state: S,
}