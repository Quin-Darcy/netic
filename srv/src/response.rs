#![allow(dead_code)]
#![allow(unused_variables)]

use crate::{Message, MessageType};
use crate::{ServerState, ServerError};
use crate::StateTransitionRules;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerResponse {
	HelloResponse(String),
	TimeResponse(String),
	GoodbyeResponse(String),
	ErrorResponse(ServerError),
}

impl ServerResponse {
	pub fn from_message(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> Self {
		// Check if there were any errors while parsing the message
		if !message.parsing_results.is_empty() {
			return ServerResponse::ErrorResponse(message.parsing_results[0].clone());
		}

		// Given the current_state and the message received, we can call the get_next_state which uses the transition rules
		// to define what the next server state should be and returns it. Provided a next_state is returned, then we return 
		// message field of the Response struct. Otherwise, the return is set to the particular ServerError
		match transition_rules.get_next_state(current_state, &message.message_type) {
			Some(next_state) => match message.message_type {
				MessageType::Hello => ServerResponse::HelloResponse(String::from("Hello, client!\n")),
				MessageType::TimeRequest => ServerResponse::TimeResponse(String::from("The time is now.\n")),
				MessageType::Goodbye => ServerResponse::GoodbyeResponse(String::from("Goodbye, client!\n")),
				_ => ServerResponse::ErrorResponse(ServerError::InvalidStateTransition),
			}
			None => ServerResponse::ErrorResponse(ServerError::InvalidStateTransition),
		}
	}
}

pub struct Response {
	pub response_string: String,
	// Possiby additional fields
}

impl Response {
	pub fn new(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> Self {
		let response_string = Response::evaluate_message(message, current_state, transition_rules);

		Response { response_string }
	}

	fn evaluate_message(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> String {
	    let server_response = ServerResponse::from_message(message, current_state, transition_rules);
	    let (code, status, response_message) = match server_response {
	        ServerResponse::HelloResponse(msg) => (200, "OK", msg),
	        ServerResponse::TimeResponse(msg) => (200, "OK", msg),
	        ServerResponse::GoodbyeResponse(msg) => (200, "OK", msg),
	        ServerResponse::ErrorResponse(err) => {
	            let (code, msg) = match err {
	                ServerError::InvalidPayloadLength => (400, "Invalid payload length"),
	                ServerError::UnrecognizedHeader => (400, "Unrecognized header"),
	                ServerError::UnrecognizedPayload => (400, "Unrecognized payload"),
	                ServerError::HeaderMismatch => (400, "Header mismatch"),
	                ServerError::NonUTF8Sequence => (400, "Non-UTF8 sequence"),
	                ServerError::InsufficientMessageSize => (400, "Insufficient message size"),
	                ServerError::InvalidStateTransition => (400, "Invalid state transition"),
	            };
	            (code, "ERROR", msg.to_string())
	        }
	    };

	    format!("{};{};{}", code, status, response_message)
	}
}