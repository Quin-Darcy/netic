#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::hash::Hash;
use std::cmp::PartialEq;

use crate::Message;
use crate::Response;


// The Protocol trait is a common interface which defines a set of shared behavior 
// across all types which have an implementation of the Prtotocol trait.
//
// The declaration states that any type implementing the Protocol trait must 
// also implement the Clone trait.
pub trait Protocol: Sized+Clone {
	type MessageType: PartialEq + Clone;
	type MessageSectionsKey: PartialEq + Eq + Hash + Clone ;
	type MessageSectionsValue: PartialEq + Clone;

	type ServerState: Clone + Eq + PartialEq + Hash;

	// Note that Self is a type alias that refers to the implementing type, whereas
	// &self is a reference to the instance of the implementing type. Here, by type
	// we mean the various protocol types, SIP, DNS, QUIC, etc.
	//
	// By including <Self>, we are specifying that the Message struct should have the 
	// same implementing type (i.e., the same protocol) as the type implementing the 
	// Protocol trait. By using <Self>, we ensure that the Message being passed to the 
	// method has the same protocol as the one being implemented. This prevents, say, 
	// passing a DNS message to a SIP method. 
	//
	// Additionally, using <Self> in the method signature of the Protocol trait allows 
	// you to write a method signature that is generic and works for any type implementing 
	// the Protocol trait.

	fn random_message(&self) -> Message<Self>;
	fn build_message(&self, message_bytes: &[u8]) -> Message<Self>;
	fn mutate_message(&self, message: &Message<Self>) -> Message<Self>;
	fn crossover_message(&self, message1: &Message<Self>, message2: &Message<Self>) -> (Message<Self>, Message<Self>);

	fn parse_response(&self, response: &Response) -> Self::ServerState;
}