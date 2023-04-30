#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::default::Default;
use rand::seq::SliceRandom;
use rand::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Formatter;
use std::fmt;
use std::fmt::Debug;

use crate::Protocol;
use crate::Message;
use crate::Response;


#[derive(Clone, PartialEq, Debug)]
pub enum GreetingMessageType {
	Hello,
	TimeRequest,
	Goodbye,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum GreetingMessageSectionsKey {
	Header,
	Length,
	Payload,
}

#[derive(Clone, PartialEq, Debug)]
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
        GreetingMessageSectionsValue {
        	header: [0x00, 0x00, 0x00, 0x00],
        	length: 0_u64,
        	payload: Vec::new(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct GreetingServerState {
	response_code: u16,
	status_message: String,
	payload: String,
}

impl Debug for GreetingServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}: {}",
            self.response_code, self.status_message, self.payload
        )
    }
}

#[derive(Clone, PartialEq)]
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

	    // If the given byte array is too short, this adds a padding of 
	    // zeros to the end of the byte array to make up the difference
	    let message_bytes = if message_bytes.len() < MIN_MESSAGE_LENGTH {
	        let mut padded_message = message_bytes.to_vec();
	        let padding_len = MIN_MESSAGE_LENGTH - message_bytes.len();
	        padded_message.extend(std::iter::repeat(0).take(padding_len));
	        padded_message
	    } else {
	        message_bytes.to_vec()
	    };

	    let (header, remaining) = message_bytes.split_at(4);
	    let header = header.to_vec();
	    let (length, payload) = remaining.split_at(8);
	    let length: u64 = u64::from_be_bytes(length.try_into().unwrap());

	    let payload_str = std::str::from_utf8(payload);

	    // If the payload does not match any of the standard payloads, 
	    // a random MessageType is selected
	    let message_type = match payload_str {
	        Ok("Hello!\n") => GreetingMessageType::Hello,
	        Ok("What time is it?\n") => GreetingMessageType::TimeRequest,
	        Ok("Goodbye!\n") => GreetingMessageType::Goodbye,
	        _ => {
	            let mut rng = rand::thread_rng();
	            let random_type = rng.gen_range(0..3);
	            match random_type {
	                0 => GreetingMessageType::Hello,
	                1 => GreetingMessageType::TimeRequest,
	                2 => GreetingMessageType::Goodbye,
	                _ => unreachable!(),
	            }
	        }
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
	    	println!("INVALID SERVER RESPONSE");
	    	GreetingServerState {
	    		response_code: 200,
	    		status_message: String::from("OK"),
	    		payload: String::from("Hello, client!\n"),
	    	}
	    }
	}

	fn mutate_message(&self, message: &Message<Self>) -> Message<Self> {
		// Randomly choose between byte-level mutation or section-level mutation
		let mut rng = rand::thread_rng();
		let mutation_level = rng.gen_range(0..2);  

		match mutation_level {
			0 => mutate_bytes(message),
			1 => mutate_sections(message),
			_ => panic!("Unexpected mutation_level value"),
		}
	}

	fn crossover_messages(&self, message1: &Message<Self>, message2: &Message<Self>) -> (Message<Self>, Message<Self>) {
		// Randomly choose between byte-level or section-level crossover
		let mut rng = rand::thread_rng();
		let crossover_level = rng.gen_range(0..2);

		match crossover_level {
			0 => crossover_bytes(message1, message2),
			1 => crossover_sections(message1, message2),
			_ => panic!("Unexpected crossover_level value"),
		}
	}
}

// Mutation helper functions
fn mutate_bytes(message: &Message<GreetingProtocol>) -> Message<GreetingProtocol> {
	let mut rng = rand::thread_rng();
	let mutation_type = rng.gen_range(0..5);

	let mut mutated_data = message.data.clone();
	let mutated_message: Message<GreetingProtocol>;

	// This instance is needed to access the methods within the Protocol implementation
	// of GreetingProtocol
	let protocol_instance = GreetingProtocol;

	match mutation_type {
		0 => {
			// Byte substitution
			let byte_index = rng.gen_range(0..mutated_data.len());
			let random_byte = rand::random::<u8>();
			mutated_data[byte_index] = random_byte;
		}
		1 => {
			// Byte insertion
			let byte_index = rng.gen_range(0..=mutated_data.len());
			let random_byte = rand::random::<u8>();
			mutated_data.insert(byte_index, random_byte);
		}
		2 => {
			// Byte deletion
			if !mutated_data.is_empty() {
				let byte_index = rng.gen_range(0..mutated_data.len());
				mutated_data.remove(byte_index);
			}
		}
		3 => {
			// Byte swap
			let byte_index1 = rng.gen_range(0..mutated_data.len());
			let byte_index2 = rng.gen_range(0..mutated_data.len());

			let temp_byte = mutated_data[byte_index1];
			mutated_data[byte_index1] = mutated_data[byte_index2];
			mutated_data[byte_index2] = temp_byte;
		}
		_ => {}
	}

	// Build new message from mutated_data
	mutated_message = protocol_instance.build_message(&mutated_data);
	return mutated_message;
}

fn mutate_sections(message: &Message<GreetingProtocol>) -> Message<GreetingProtocol> {
	let mut rng = rand::thread_rng();
	let mutation_type = rng.gen_range(0..3);

	let mut mutated_sections = message.sections.clone();
	let mutated_message: Message<GreetingProtocol>;

	// This instance is needed to access the methods within the Protocol implementation
	// of GreetingProtocol
	let protocol_instance = GreetingProtocol;

	match mutation_type {
		0 => {
			// Header swap
			let header_choice = rng.gen_range(0..3);

			match header_choice {
				0 => {
					mutated_sections.insert(
	        			GreetingMessageSectionsKey::Header,
	        			GreetingMessageSectionsValue { header: [0x48, 0x45, 0x4C, 0x4F], ..Default::default() },
	    			);
				}
				1 => {
					mutated_sections.insert(
	        			GreetingMessageSectionsKey::Header,
	        			GreetingMessageSectionsValue { header: [0x54, 0x49, 0x4D, 0x45], ..Default::default() },
	    			);
				}
				2 => {
					mutated_sections.insert(
	        			GreetingMessageSectionsKey::Header,
	        			GreetingMessageSectionsValue { header: [0x42, 0x59, 0x45, 0x5F], ..Default::default() },
	    			);
				}
				_ => {}
			}
		}
		1 => {
		    // Invalidate payload length
		    let payload_length = mutated_sections.get(&GreetingMessageSectionsKey::Length).unwrap().length;
		    let multiplier = rng.gen_range(2..10) as u64;

		    if let Some(new_payload_length) = multiplier.checked_mul(payload_length) {
		        mutated_sections.insert(
		            GreetingMessageSectionsKey::Length,
		            GreetingMessageSectionsValue { length: new_payload_length, ..Default::default() },
		        );
		    } else {
		        // Handle the case where an overflow occurs, e.g., by setting a maximum length value or skipping the mutation
		        let random_length: u64 = rng.gen_range(2..300) as u64;
		        mutated_sections.insert(
		            GreetingMessageSectionsKey::Length,
		            GreetingMessageSectionsValue { length: random_length, ..Default::default() },
		        );
		    }
		}
		2 => {
			// Replace random byte into payload
			let mut payload = mutated_sections.get(&GreetingMessageSectionsKey::Payload).unwrap().payload.clone();
			let byte_index = rng.gen_range(0..payload.len());
			let random_byte = rand::random::<u8>();
			payload[byte_index] = random_byte;
			
			mutated_sections.insert(
				GreetingMessageSectionsKey::Payload,
				GreetingMessageSectionsValue { payload: payload, ..Default::default() }
			);
		}
		_ => {}
	}

	// Build new message from mutated sections
    let header = mutated_sections.get(&GreetingMessageSectionsKey::Header).unwrap().header;
    let length = mutated_sections.get(&GreetingMessageSectionsKey::Length).unwrap().length.to_be_bytes();
    let payload = &mutated_sections.get(&GreetingMessageSectionsKey::Payload).unwrap().payload;

    let new_data: Vec<u8> = [&header[..], &length[..], payload].concat();

    mutated_message = protocol_instance.build_message(&new_data);
	return mutated_message;
}

// Crossover helper functions
fn crossover_bytes(message1: &Message<GreetingProtocol>, message2: &Message<GreetingProtocol>) -> (Message<GreetingProtocol>, Message<GreetingProtocol>) {
	// Logic for two-point crossover 
	let mut rng = rand::thread_rng();

	// This instance is needed to access the methods within the Protocol implementation
	// of GreetingProtocol
	let protocol_instance = GreetingProtocol;

	// Determine which parent's data vector hash more bytes
	let (small_parent_data, big_parent_data) = if message1.data.len() < message2.data.len() {
		(message1.data.clone(), message2.data.clone())
	} else {
		(message2.data.clone(), message1.data.clone())
	};

	let min_len = small_parent_data.len();
	let max_len = big_parent_data.len();

	let crossover_point1 = rng.gen_range(0..min_len);
	let crossover_point2 = rng.gen_range(crossover_point1..min_len);

	let mut small_offspring_data = small_parent_data.clone();
	let mut big_offspring_data = big_parent_data.clone();

	// This loop cross transplants the regions defined by the two crossover points
	for i in crossover_point1..=crossover_point2 {
		small_offspring_data[i] = big_parent_data[i];
		big_offspring_data[i] = small_parent_data[i];
	} 

	let offspring1 = protocol_instance.build_message(&small_offspring_data);
	let offspring2 = protocol_instance.build_message(&big_offspring_data);
	
	return (offspring1, offspring2);
}

fn crossover_sections(message1: &Message<GreetingProtocol>, message2: &Message<GreetingProtocol>) -> (Message<GreetingProtocol>, Message<GreetingProtocol>) {
    let mut rng = rand::thread_rng();
    
	// This instance is needed to access the methods within the Protocol implementation
	// of GreetingProtocol
	let protocol_instance = GreetingProtocol;
    
    let mut offspring1_sections = HashMap::new();
    let mut offspring2_sections = HashMap::new();

    // Construct the offsprings' sections by going through each key and deciding if the value
    // for that key should come from the first or second parent. Which ever choice is made, the other
    // offspring receives the value from the opposite parent
    for key in &[GreetingMessageSectionsKey::Header, GreetingMessageSectionsKey::Length, GreetingMessageSectionsKey::Payload] {
        if rng.gen_bool(0.5) {
            offspring1_sections.insert(key.clone(), message1.sections.get(key).unwrap().clone());
            offspring2_sections.insert(key.clone(), message2.sections.get(key).unwrap().clone());
        } else {
            offspring1_sections.insert(key.clone(), message2.sections.get(key).unwrap().clone());
            offspring2_sections.insert(key.clone(), message1.sections.get(key).unwrap().clone());
        }
    }

    // This closure assembles the data for the offspring by going through the respective sections 
    // which were just populated and collection all the information into a byte array which is sent
    // over to protocol_instance.build_message, returning an instance of Message.
    let build_offspring = |sections: HashMap<GreetingMessageSectionsKey, GreetingMessageSectionsValue>| {
        let header = sections.get(&GreetingMessageSectionsKey::Header).unwrap().header;
        let length = sections.get(&GreetingMessageSectionsKey::Length).unwrap().length.to_be_bytes();
        let payload = &sections.get(&GreetingMessageSectionsKey::Payload).unwrap().payload;

        let new_data: Vec<u8> = [&header[..], &length[..], payload].concat();
        protocol_instance.build_message(&new_data)
    };

    let offspring1 = build_offspring(offspring1_sections);
    let offspring2 = build_offspring(offspring2_sections);

    (offspring1, offspring2)
}
