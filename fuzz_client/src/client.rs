#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]


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
	pub corpus: Vec<MessageSequence<P>>,
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
	// Send every MessageSequence in the current corpus and collect the Message sent
	// with the Responses received and return this collection
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

	// Take the interection history (Vec<Message<P>, Response)>) of each MessageSequnce sent and 
	// construct the resultant StateTransitions from this information and return a vector of all the 
	// StateTransitions
	fn process_trace(&mut self, corpus_trace: Vec<Vec<(Message<P>, Response)>>) -> Vec<StateTransition<P::ServerState, P>> {
	    let mut state_transitions: Vec<StateTransition<P::ServerState, P>> = Vec::new();

	    for interaction_history in corpus_trace {
	        // Option is used here to represent the possibility of having
	        // a server state or not since the previous state is unknown
	        // at the beginning of an interaction history.
	        let mut previous_server_state: Option<P::ServerState> = None;

	        for (message, response) in interaction_history {
	            let target_state: P::ServerState = self.protocol.parse_response(&response);
	            
	            // If previous_server_state is not None, then we can initialize
	            // an instance of StateTransition as we have every field's value
	            if let Some(source_state) = previous_server_state {
	                let state_transition = StateTransition {
	                    source_state,
	                    message: message.clone(),
	                    target_state: target_state.clone(),
	                };
	                state_transitions.push(state_transition);
	            }

	            // The server state prompted by the current message,
	            // i.e, target_state is the next message's previous_server_state
	            previous_server_state = Some(target_state);
	        }
	    }

	    return state_transitions;
	}

	// Go through each StateTransition in the processed trace and use them to update state_model
    fn update_state_model(&mut self, state_transitions: Vec<StateTransition<P::ServerState, P>>) {
        for transition in state_transitions {
            self.state_model.add(transition.source_state.clone(), transition.target_state.clone(), &transition.message);
        }
    }

    // Go through each MessageSequence within the corpus and mutate it according to the mutation rate
    fn mutate_corpus(&mut self, message_sequence_mutation_rate: f32, message_mutation_rate: f32) {
        let mut rng = rand::thread_rng();

        for message_sequence in &mut self.corpus {
            if rng.gen::<f32>() < message_sequence_mutation_rate {
                message_sequence.mutate_message_sequence(self.protocol.clone(), message_mutation_rate, &self.message_pool);
            }
        }
    }

    // Perform crossover on the MessageSequences within corpus
    pub fn crossover_corpus(&mut self, message_sequence_crossover_rate: f32, message_crossover_rate: f32) {
        let mut rng = rand::thread_rng();
        let corpus_len = self.corpus.len();
        
        // Pairs of indices to perform crossover on
        let mut crossover_pairs: Vec<(usize, usize)> = Vec::new();

        // This loop iterates over all i, j in {0,...,corpus_len} such that i < j effectively collecting 
        // all unique ordered pairs to be formed irrespective of ordering in the components
        for i in 0..corpus_len {
            for j in i + 1..corpus_len {
                if rng.gen::<f32>() < message_sequence_crossover_rate {
                    crossover_pairs.push((i, j));
                }
            }
        }

        // Here we replace the parents in the corpus with their two offspring
        for (idx1, idx2) in crossover_pairs {
        	let mut parent1 = self.corpus[idx1].clone();
        	let parent2 = self.corpus[idx2].clone();

            let (offspring1, offspring2) = parent1.crossover_message_sequences(&parent2, message_crossover_rate);
            self.corpus[idx1] = offspring1;
            self.corpus[idx2] = offspring2;
        }
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