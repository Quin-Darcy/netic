use crate::ServerError;


pub struct Message {
	data: Vec<u8>,
	header: [u8; 4],
	payload_length: u64,
	payload: String,
	parsing_results: Vec<ServerError>,
}

impl Message {
	pub fn new(data: &[u8]) -> Self {
		let mut header: [u8; 4] = [0_u8; 4];
		let mut payload_length: u64 = 0_u64;
		let mut payload: String = String::from("");
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
		}

		Self {
			data: data.to_vec(),
			header,
			payload_length,
			payload,
			parsing_results,
		}
	}
}