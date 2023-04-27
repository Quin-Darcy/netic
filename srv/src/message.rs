use crate::ServerError;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
	Hello, 
	TimeRequest,
	Goodbye,
	Undefined,
}

pub struct Message {
	data: Vec<u8>,
	header: [u8; 4],
	payload_length: u64,
	payload: String,
	message_type: MessageType,
	parsing_results: Vec<ServerError>,
}

impl Message {
	pub fn new(data: &[u8]) -> Self {
		let mut header: [u8; 4] = [0_u8; 4];
		let mut payload_length: u64 = 0_u64;
		let mut payload: String = String::from("");
		let mut message_type: MessageType = MessageType::Undefined;
		let mut parsing_results: Vec<ServerError> = Vec::new();

		// Since the header is 4 bytes and the payload length is 8 bytes
		// the data must have at least 12 bytes in it
		if data.len() < 12 {
			parsing_results.push(ServerError::InsufficientMessageSize);
		} else {
			// Parse the header
			header.copy_from_slice(&data[..4]);

			// Parse the payload length
			payload_length = u64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);

			// Check if payload length matches the length of the remaining data length
			if payload_length as usize != data.len() - 12 {
				parsing_results.push(ServerError::InvalidPayloadLength);
			} else {
				// Parse the payload as a UTF-8 string
				match String::from_utf8(data[12..].to_vec()) {
					Ok(parsed_payload) => payload = parsed_payload,
					Err(_) => parsing_results.push(ServerError::NonUTF8Sequence),
				}
			}

			// Determine message type from header and payload
			let valid_headers = [[0x48, 0x45, 0x4C, 0x4F], [0x54, 0x49, 0x4D, 0x45], [0x42, 0x59, 0x45, 0x5F]];
			let valid_payloads = [String::from("Hello!\n"), String::from("What time is it?\n"), String::from("Goodbye!\n")];

			if valid_headers.contains(&header) {
				if valid_payloads.contains(&payload) {
					message_type = if header == valid_headers[0] && payload == valid_payloads[0] {
						MessageType::Hello
					} else if header == valid_headers[1] && payload == valid_payloads[1] {
						MessageType::TimeRequest
					} else if header == valid_headers[2] && payload == valid_payloads[2] {
						MessageType::Goodbye
					} else {
						parsing_results.push(ServerError::HeaderMismatch);
						MessageType::Undefined
					}
				} else {
					parsing_results.push(ServerError::UnrecognizedPayload);
				}
			} else {
				parsing_results.push(ServerError::UnrecognizedHeader);
			}
		}

		Self {
			data: data.to_vec(),
			header,
			payload_length,
			payload,
			message_type,
			parsing_results,
		}
	}
}