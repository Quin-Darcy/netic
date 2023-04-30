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
	pub fn from_message(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> (Self, ServerState) {
		// Check if there were any errors while parsing the message
		if !message.parsing_results.is_empty() {
			return (ServerResponse::ErrorResponse(message.parsing_results[0].clone()), current_state.clone());
		}

		// Given the current_state and the message received, we can call the get_next_state which uses the transition rules
		// to define what the next server state should be and returns it. Provided a next_state is returned, then we return 
		// message field of the Response struct. Otherwise, the return is set to the particular ServerError
		match transition_rules.get_next_state(current_state, &message.message_type) {
			Some(next_state) => match message.message_type {
				MessageType::Hello => (ServerResponse::HelloResponse(String::from("Hello, client!\n")), next_state.clone()),
				MessageType::TimeRequest => (ServerResponse::TimeResponse(String::from("The time is now.\n")), next_state.clone()),
				MessageType::Goodbye => (ServerResponse::GoodbyeResponse(String::from("Goodbye, client!\n")), next_state.clone()),
				_ => (ServerResponse::ErrorResponse(ServerError::InvalidStateTransition), current_state.clone()),
			}
			None => (ServerResponse::ErrorResponse(ServerError::InvalidStateTransition), current_state.clone()),
		}
	}
}

pub struct Response {
	pub response_string: String,
	// Possiby additional fields
}

impl Response {
	pub fn new(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> (Self, ServerState) {
		let (response_string, new_state) = Response::evaluate_message(message, current_state, transition_rules);

		(Response { response_string }, new_state.clone())
	}

	fn evaluate_message(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> (String, ServerState) {
	    let (server_response, new_server_state) = ServerResponse::from_message(message, current_state, transition_rules);
	    let (code, status, response_message) = match server_response {
	        ServerResponse::HelloResponse(msg) => (200, "OK", msg),
	        ServerResponse::TimeResponse(msg) => (200, "OK", msg),
	        ServerResponse::GoodbyeResponse(msg) => (200, "OK", msg),
	        ServerResponse::ErrorResponse(err) => {
	            let (code, msg) = match err {
	                ServerError::InvalidPayloadLength => (400, "Invalid payload length\n"),
	                ServerError::UnrecognizedHeader => (400, "Unrecognized header\n"),
	                ServerError::UnrecognizedPayload => (400, "Unrecognized payload\n"),
	                ServerError::HeaderMismatch => (400, "Header mismatch\n"),
	                ServerError::NonUTF8Sequence => (400, "Non-UTF8 sequence\n"),
	                ServerError::InsufficientMessageSize => (400, "Insufficient message size\n"),
	                ServerError::InvalidStateTransition => (400, "Invalid state transition\n"),
	            };
	            (code, "ERROR", msg.to_string())
	        }
	    };

	    (format!("{};{};{}", code, status, response_message), new_server_state.clone())
	}
}