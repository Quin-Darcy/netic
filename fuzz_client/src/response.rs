pub struct Response {
	pub data: Vec<u8>,
}

impl Response {
	pub fn new(buffer: Vec<u8>) -> Self {
		Self {
			data: buffer,
		}
	}
}