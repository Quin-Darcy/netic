#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::default::Default;
use rand::seq::SliceRandom;
use std::collections::HashMap;

use crate::Protocol;
use crate::Message;
use crate::Response;


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

pub struct GreetingServerState {
	response_code: u16,
	status_message: String,
	payload: String,
}

#[derive(Clone)]
pub struct GreetingProtocol;


// GreetingProtocol is a struct that implements the Protocol trait by providing
// concrete implementations for its associated types and methods defined in protocol_traits.rs
impl Protocol for GreetingProtocol {
	type MessageType = GreetingMessageType;
	type MessageSectionsKey = GreetingMessageSectionsKey;
	type MessageSectionsValue = GreetingMessageSectionsValue;

	type ServerState = GreetingServerState;

	fn random_message(&self) -> Message<Self> {
		let mut rng = rand::thread_rng();
		let possible_payloads: [&str; 3] = ["Hello!\n", "What time is it?\n", "Goodbye!\n"];
	
		let payload = possible_payloads.choose(&mut rng).unwrap();
	    let (message_type, header) = match *payload {
	        "Hello!\n" => (GreetingMessageType::Hello, [0x48, 0x45, 0x4C, 0x4F]),
	        "What time is it?\n" => (GreetingMessageType::TimeRequest, [0x54, 0x49, 0x4D, 0x45]),
	        "Goodbye!\n" => (GreetingMessageType::Goodbye, [0x42, 0x59, 0x45, 0x5F]),
	        _ => unreachable!(),
	    };


	    let mut sections = HashMap::new();
	    sections.insert(
	        GreetingMessageSectionsKey::Header,
	        GreetingMessageSectionsValue { header, ..Default::default() },
	    );
	    sections.insert(
	        GreetingMessageSectionsKey::Length,
	        GreetingMessageSectionsValue { length: payload.len() as u64, ..Default::default() },
	    );
	    sections.insert(
	        GreetingMessageSectionsKey::Payload,
	        GreetingMessageSectionsValue { payload: payload.as_bytes().to_vec(), ..Default::default() },
	    );

	    let data = [&header[..], &(payload.len() as u64).to_be_bytes(), &payload.as_bytes()].concat();

	    Message {
	        protocol: GreetingProtocol,
	        data,
	        message_type,
	        sections,
	    }
	}

	fn build_message(&self, message_bytes: &[u8]) -> Message<Self> {
	    const MIN_MESSAGE_LENGTH: usize = 12;

	    // Logic to build a message whose format is defined by GreetingProtocol
	    if message_bytes.len() < MIN_MESSAGE_LENGTH {
	        panic!("Error parsing raw message data");
	    }

	    let (header, remaining) = message_bytes.split_at(4);
	    let header = header.to_vec();
	    let (length, payload) = remaining.split_at(8);
	    let length: u64 = u64::from_be_bytes(length.try_into().unwrap());

	    let message_type = match std::str::from_utf8(payload) {
	        Ok("Hello!\n") => GreetingMessageType::Hello,
	        Ok("What time is it?\n") => GreetingMessageType::TimeRequest,
	        Ok("Goodbye!\n") => GreetingMessageType::Goodbye,
	        _ => panic!("Unexpected payload: {:?}", payload),
	    };

	    let header_array: [u8; 4] = header.clone().try_into().expect("Header has incorrect length");

	    let mut sections = HashMap::new();
	    sections.insert(
	        GreetingMessageSectionsKey::Header,
	        GreetingMessageSectionsValue { header: header_array, ..Default::default() },
	    );
	    sections.insert(
	        GreetingMessageSectionsKey::Length,
	        GreetingMessageSectionsValue { length, ..Default::default() },
	    );
	    sections.insert(
	        GreetingMessageSectionsKey::Payload,
	        GreetingMessageSectionsValue { payload: payload.to_vec(), ..Default::default() },
	    );

	    let data = [&header[..], &(length.to_be_bytes()), &payload[..]].concat();

	    Message {
	        protocol: GreetingProtocol,
	        data,
	        message_type,
	        sections,
	    }
	}


	fn mutate_message(&self, message: &Message<Self>) -> Message<Self> {
		// Logic to mutate the message
		todo!();
	}

	fn crossover_message(&self, message1: &Message<Self>, message2: &Message<Self>) -> Message<Self> {
		// Logic for performing crossover on two Messages
		todo!();
	}

	fn parse_response(&self, response: &Response) -> GreetingServerState {
	    let response_str: String = String::from_utf8(response.data.clone()).unwrap();
	    let response_parts: Vec<&str> = response_str.split(";").collect();

	    if let [response_code_str, status_message, payload] = response_parts.as_slice() {
	        let response_code = response_code_str.parse::<u16>().unwrap();
	        let status_message: String = String::from(*status_message);
	        let payload: String = String::from(*payload);

	        GreetingServerState {
	            response_code,
	            status_message,
	            payload,
	        }
	    } else {
	        panic!("Invalid response format");
	    }
	}
}