#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::str;
use std::thread;
use std::hash::Hash;
use std::cmp::PartialEq;
use std::time::Duration;
use std::collections::HashMap;
use std::net::{TcpStream, Shutdown};
use std::io::{self, BufRead, BufReader, Write};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use crate::Protocol; 
use crate::Message;
use crate::MessageSequence;
use crate::StateTransition;
use crate::StateModel;
use crate::Response;

use crate::GreetingProtocol;

pub struct FuzzConfig {
	pub generations: usize,
	pub selection_pressure: f32,
	pub sequence_mutation_rate: f32,
	pub sequence_crossover_rate: f32,
	pub message_mutation_rate: f32,
	pub message_crossover_rate: f32,
	pub message_pool_size: usize,
	pub pool_update_rate: f32,
}


pub struct Client<P: Protocol + Clone + PartialEq> {
	server_address: String,
	protocol: P,
	corpus: Vec<MessageSequence<P>>,
	state_model: StateModel<P>,
	message_pool: Vec<Message<P>>, 
}

impl<P: Protocol + Clone + PartialEq> Client<P> {
    // Initialize new client with random corpus and message_pool
    pub fn new(server_address: String, protocol: P) -> Self {
        const MESSAGE_SEQUENCE_LENGTH: usize = 10;
        const MESSAGE_POOL_LENGTH: usize = 10;
        const INITIAL_CORPUS_LENGTH: usize = 10;

        let mut corpus = Vec::new();
        for _ in 0..INITIAL_CORPUS_LENGTH {
            corpus.push(MessageSequence::random_message_sequence(
                protocol.clone(),
                MESSAGE_SEQUENCE_LENGTH,
            ));
        }

        let mut message_pool = Vec::new();
        for _ in 0..MESSAGE_POOL_LENGTH {
            message_pool.push(Message::random_message(protocol.clone()));
        }

        Self {
            server_address,
            protocol,
            corpus,
            state_model: StateModel::new(),
            message_pool,
        }
    }

    fn initialize_stream(&mut self) -> TcpStream {
    	TcpStream::connect(self.server_address.as_str())
    	.expect("Could not connect to server")
    }

    fn terminate_stream(&mut self, stream: TcpStream) {
    	stream.shutdown(Shutdown::Both).expect("Shutdown call failed");
    }

    fn send_message(&mut self, mut stream: &TcpStream, message: &Message<P>) {
    	stream.write(&message.data)
    	.expect("Failed to write to server");
    }

	fn read_response(&mut self, reader: &mut BufReader<&TcpStream>) -> Response {
		let mut buffer: Vec<u8> = Vec::new();
        	reader.read_until(b'\n', &mut buffer).expect("Could not read into buffer");
        	return Response::new(buffer);
	}

	// A new TcpStream is created and destroyed for each MessageSequence
	fn run_message_sequence(&mut self, message_sequence: &MessageSequence<P>) -> Vec<(Message<P>, Response)> {
		let stream: TcpStream = self.initialize_stream();
		let mut reader = BufReader::new(&stream);
		let mut message_response: Vec<(Message<P>, Response)> = Vec::new();

		for (index, message) in message_sequence.messages.iter().enumerate() {
			self.send_message(&stream, message);

			let response: Response = self.read_response(&mut reader);
			message_response.push((message.clone(), response));

			// wait message_sequence.timings[index] many seconds
			// before sending the next message
			if index < message_sequence.timings.len() {
				let sleep_duration: Duration = Duration::from_secs_f32(message_sequence.timings[index]);
				thread::sleep(sleep_duration);
			}
		}
		self.terminate_stream(stream);
		return message_response
	}

	pub fn fuzz(&mut self, config: FuzzConfig) {

		for _ in 0..config.generations {
			let corpus_len: usize = self.corpus.len();
			let mut message_sequence: MessageSequence<P>;
			let mut interaction_history: Vec<(Message<P>, Response)>;

			for i in 0..corpus_len {
				message_sequence = self.corpus[i].clone();
				interaction_history = self.run_message_sequence(&message_sequence);
			}
		}
	}
}