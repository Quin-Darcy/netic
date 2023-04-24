#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::default::Default;
use std::hash::Hash;


use crate::Protocol; 
use crate::protocols::GreetingProtocol;


// Message is a generic struct that depends on a type P implementing
// the Protocol trait. So when we create an instance of, say, Message<SIP>
// the Message struct will use the associated types and methods defined in
// the SIPProtocol implementation of the Protcol trait. This allows Message
// to store data that is specific to the SIPProtocol. Additionally, when
// methods like build_message, validate_message, etc, need to be used, the 
// Message struct will use the implementations provided by SIPProtocol.
pub struct Message<P: Protocol> {
	pub protocol: P, // This gives us an instance of the type implementing the Protocol trait
	pub data: Vec<u8>,
	pub message_type: P::MessageType,
	pub sections: HashMap<P::MessageSectionsKey, P::MessageSectionsValue>,
}

impl<P: Protocol> Message<P> {
	// This is the method which creates a new Message instance using the default values
	// and the Default implementation of P::MessageType 
	pub fn new(protocol: P) -> Self where <P as Protocol>::MessageType: Default {
		Self {
			protocol,
			data: Vec::new(),
			message_type: Default::default(),
			sections: HashMap::new(),
		}
	}

	// Below are wrapper methods for the protocol specific implementation
	// of the methods by the same name. This approach causes Message 
	// to become a higher-level abstraction that encapsualtes the details
	// of working with specific protocols. 

	// from_bytes and random_message are responsible for creating new Message
	// instances and they don't need to be called on an existing instance. Instead
	// they take protocol as an argument.
	pub fn random_message(protocol: P) -> Self {
		protocol.random_message()
	}

	pub fn from_bytes(protocol: P, message_bytes: &[u8]) -> Self {
		protocol.build_message(message_bytes)
	}

	pub fn mutate_message(&mut self) {
		let mutated_message = self.protocol.mutate_message(self);

		self.data = mutated_message.data;
		self.message_type = mutated_message.message_type;
		self.sections = mutated_message.sections;
	}

	pub fn crossover_messages(&self, other: &Self) -> (Self, Self) {
		self.protocol.crossover_messages(self, other)
	}
}

// Since PartialEq and Clone cannot be derived automatically for Message
// because it has a generic type parameter P which is bound by the Protocol
// trait which does not implement PartialEq or Clone, then we must manually 
// implement these traits. 
impl<P: Protocol> PartialEq for Message<P> {
    fn eq(&self, other: &Self) -> bool {
        self.message_type == other.message_type
            && self.data == other.data
            && self.sections == other.sections
    }
}

impl<P: Protocol + Clone> Clone for Message<P> {
    fn clone(&self) -> Self {
        Self {
        	protocol: self.protocol.clone(),
        	data: self.data.clone(),
        	message_type: self.message_type.clone(),
        	sections: self.sections.clone(),
        }
    }
}

