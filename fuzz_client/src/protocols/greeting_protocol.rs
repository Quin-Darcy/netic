#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::default::Default;

use crate::Protocol;
use crate::Message;


#[derive(Clone, PartialEq, Debug)]
pub enum GreetingMessageType {
	Hello,
	TimeRequest,
	Goodbye,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub enum GreetingMessageSectionsKey {
	Header,
	Length,
	Payload,
}

pub struct GreetingMessageSectionsValue {
	pub header: [u8; 4],
	pub length: u64,
	pub payload: Vec<u8>
}

impl Default for GreetingMessageType {
    fn default() -> Self {
        Self::Hello
    }
}

impl Default for GreetingMessageSectionsKey {
    fn default() -> Self {
        Self::Header
    }
}

impl Default for GreetingMessageSectionsValue {
    fn default() -> Self {
        //Self::Text(String::new())
        GreetingMessageSectionsValue {
        	header: [0x00, 0x00, 0x00, 0x00],
        	length: 0_u64,
        	payload: Vec::new(),
        }
    }
}

pub struct GreetingProtocol;


// GreetingProtocol is a struct that implements the Protocol trait by providing
// concrete implementations for its associated types and methods defined in protocol_traits.rs
impl Protocol for GreetingProtocol {
	type MessageType = GreetingMessageType;
	type MessageSectionsKey = GreetingMessageSectionsKey;
	type MessageSectionsValue = GreetingMessageSectionsValue;

	fn random_message(&self) -> Message<Self> {
		todo!();
	}

	fn build_message(&self, message_bytes: &[u8]) -> Message<Self> {
		// Logic to build a message whose format is defined by GreetingProtocol
		todo!();
	}

	fn mutate_message(&self, message: &Message<Self>) -> Message<Self> {
		// Logic to mutate the message
		todo!();
	}

	fn crossover_message(&self, message1: &Message<Self>, message2: &Message<Self>) -> Message<Self> {
		// Logic for performing crossover on two Messages
		todo!();
	}
}