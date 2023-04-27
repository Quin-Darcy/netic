use crate::Message;
use crate::ServerState;
use crate::StateTransitionRules;


pub struct Response {
	code: u16,
	message: String,
	// Possiby additional fields
}

impl Response {
	pub fn new(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> Self {
		let (code, response_message) = Response::evaluate_message(message, current_state, transition_rules);

		Response {
			code,
			message: response_message, 
		}
	}

	fn evaluate_message(message: &Message, current_state: &ServerState, transition_rules: &StateTransitionRules) -> (u16, String) {
		let code: u16;
		let response_message: String;

		//(code, response_message)
		todo!();
	}
}