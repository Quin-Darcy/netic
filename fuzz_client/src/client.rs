#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]


use std::cmp::min;
use std::str;
use std::thread;
use std::hash::Hash;
use std::collections::HashSet;
use std::cmp::PartialEq;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::net::{TcpStream, Shutdown};
use std::io::{self, BufRead, BufReader, Write};
use std::error::Error;

use csv::Writer;
use csv::Reader;

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use crate::Protocol; 
use crate::Message;
use crate::MessageSequence;
use crate::StateTransition;
use crate::StateModel;
use crate::Response;
use crate::Transport;
use crate::TransportProtocol;

use crate::GreetingProtocol;
use crate::SMTP;

#[derive(Debug, Clone)]
pub struct FuzzConfig {
	pub generations: usize,
	pub selection_pressure: f32,
	pub sequence_mutation_rate: f32,
	pub sequence_crossover_rate: f32,
	pub message_mutation_rate: f32,
	pub message_crossover_rate: f32,
	pub message_pool_size: usize,
	pub pool_update_rate: f32,
	pub state_rarity_threshold: f32,
	pub state_coverage_weight: f32,
	pub response_time_weight: f32,
	pub state_roc_weight: f32,
	pub state_rarity_weight: f32,
}


pub struct Client<P: Protocol + Clone + PartialEq> {
	server_address: String,
	transport_protocol: TransportProtocol,
	protocol: P,
	pub corpus: Vec<MessageSequence<P>>,
	state_model: StateModel<P>,
	message_pool: Vec<Message<P>>, 
}


const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

impl<P: Protocol + Clone + PartialEq> Client<P> {
    // Initialize new client with random corpus and message_pool
    pub fn new(server_address: String, transport_protocol: TransportProtocol, protocol: P) -> Self {
        const MESSAGE_SEQUENCE_LENGTH: usize = 6;
        const MESSAGE_POOL_LENGTH: usize = 10;
        const INITIAL_CORPUS_LENGTH: usize = 5;

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
            transport_protocol,
            protocol,
            corpus,
            state_model: StateModel::new(),
            message_pool,
        }
    }

    // Initialize client with corpus pulled from PCAP file
    pub fn new_from_pcap(pcap_file: &str, protocol: P) -> Self {
    	todo!();
    }

	fn initialize_transport(&self) -> Result<Transport, std::io::Error> {
	    Transport::connect(self.transport_protocol.clone(), &self.server_address)
	}

	fn send_message(&mut self, transport: &mut Transport, message: &Message<P>) {
		let msg = String::from_utf8_lossy(&message.data).to_string();
	    transport.send(&message.data, &self.server_address);
	}

	fn read_response(&mut self, transport: &mut Transport) -> Result<Response, std::io::Error> {
	    transport.receive()
	}

	// A new Transport Stream is created and destroyed for each MessageSequence
	// Send every MessageSequence in the current corpus and collect the Message sent
	// with the Responses received and return this collection
	fn run_message_sequence(&mut self, message_sequence: &MessageSequence<P>) -> Vec<(Message<P>, Response)> {
	    let mut transport = self.initialize_transport().expect("Failed to initialize transport");
	    let mut message_response: Vec<(Message<P>, Response)> = Vec::new();

	    for (index, original_message) in message_sequence.messages.iter().enumerate() {
	        self.send_message(&mut transport, original_message);

	        // Begin timer to track server's response time
	        let start_time = Instant::now();

	        // Result is returned in case server crashes or hangs and reading from stream was not possible
	        let response_result: Result<Response, std::io::Error> = self.read_response(&mut transport);
	        let elapsed_time = start_time.elapsed();

	        let mut message = original_message.clone(); // Clone the message to create a mutable copy

	        match response_result {
	            Ok(response) => {
	                message.response_time = elapsed_time.as_secs_f32();
	                message_response.push((message, response));
	            }
	            Err(e) => {
	                message.response_time = 5.0;
	                message_response.push((message, Response::new(vec![])));
	            }
	        }

	        // wait message_sequence.timings[index] many seconds
	        // before sending the next message
	        if index < message_sequence.timings.len() {
	            let sleep_duration: Duration = Duration::from_secs_f32(message_sequence.timings[index]);
	            thread::sleep(sleep_duration);
	        }
	    }

		/* 
	    if let Err(e) = transport.shutdown() {
	        eprintln!("Error shutting down transport: {}", e);
	    }
		*/
	    message_response
	}

	// Take the interection history (Vec<Message<P>, Response)>) of each MessageSequnce sent and 
	// construct the resultant StateTransitions from this information and return a vector of all the 
	// StateTransitions
	fn process_trace(&mut self, corpus_trace: &[Vec<(Message<P>, Response)>]) -> (Vec<StateTransition<P::ServerState, P>>, Vec<usize>) {
	    let mut state_transitions: Vec<StateTransition<P::ServerState, P>> = Vec::new();

	    // This vector will contain a count of the unique ServerStates prompted by each MessageSequence
	    let mut unique_server_state_counts: Vec<usize> = Vec::new();


	    for interaction_history in corpus_trace {
	        // Option is used here to represent the possibility of having
	        // a server state or not since the previous state is unknown
	        // at the beginning of an interaction history.
	        let mut previous_server_state: Option<P::ServerState> = None;

	        // A HashSet is created for each interaction_history
	        let mut unique_server_states: HashSet<P::ServerState> = HashSet::new();

	        for (message, response) in interaction_history {
	            let target_state: P::ServerState = self.protocol.parse_response(&response);

	            // The HashSet automatically deduplicates so we need not worry checking for repeats
	            unique_server_states.insert(target_state.clone());
	            
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

	        // Add the count to the vector
	        unique_server_state_counts.push(unique_server_states.len());
	    }

	    return (state_transitions, unique_server_state_counts);
	}

	// Go through each StateTransition in the processed trace and use them to update state_model
    fn update_state_model(&mut self, state_transitions: Vec<StateTransition<P::ServerState, P>>) {
        for transition in state_transitions {
            self.state_model.add(transition.source_state.clone(), transition.target_state.clone(), &transition.message);
        }
    }

    // This method aims to identify "rare" ServerStates, which are the states that occur less frequently
    // in the StateModel based on the given rarity_threshold which is to denote a percentage.
	fn identify_rare_server_states(&self, rarity_threshold: f32) -> HashSet<P::ServerState> {
		// This will contain the "rare" server states identifies
	    let mut rare_server_states = HashSet::new();

	    // This will contain (key, value) pairs where key is defined as a particular ServerState
	    // and value is the number of times that ServerState has been transitioned to out of all
	    // transitions recorded in the state_model
	    let mut server_state_counts = HashMap::new();

	    // This will be the sum of all the number of occurrances of each ServerState
	    let mut total_server_state_occurrences = 0;

	    // Iterate over each vector of StateTransitions pointed to by each unique ServerState
	    for transitions in self.state_model.inner.values() {
	    	// Iterate over each StateTransition which will contain a "target_state"
	    	// If that target_state is already present in server_state_counts as a key, increment
	    	// the value it points to by 1. Otherwise, insert a new (target_state, occurrances)
	    	// (key, value) pair into the server_state_counts HashMap
	        for state_transition in transitions {
	            *server_state_counts.entry(state_transition.target_state.clone()).or_insert(0) += 1;
	            total_server_state_occurrences += 1;
	        }
	    }

	    // Iternate over all the (ServerState, occurances) pairs in server_state_count
	    for (server_state, count) in server_state_counts {
	    	// If out of the total number of all occurances of all ServerStates, this 
	    	// ServerState has occured less than rarity_threshold percent, then it is a "rare"
	    	// ServerState and will be placed into the rare_server_states HashSet
	        let proportion = count as f32 / total_server_state_occurrences as f32;
	        if proportion < rarity_threshold {
	            rare_server_states.insert(server_state);
	        }
	    }

	    rare_server_states
	}

	fn tournament_selection(&self, selection_pressure: f32) -> Vec<usize> {
		// Selection pressure determines the tournament size
		// The higher the pressure, the more biased the selection process
		// is to selecting fitter individuals

		let num_parents: usize = self.corpus.len();
		let tournament_size: usize = (selection_pressure * (self.corpus.len() as f32)) as usize;

		let mut rng = thread_rng();
    	let mut selected_indices: Vec<usize> = Vec::new();

    	// Each tournament gives back one "winner" who will be inserted into the mating pool. 
    	// The number of tournaments run will be equal to the size of the resulting mating pool.
    	for _ in 0..num_parents {
        	let mut tournament: Vec<usize> = (0..self.corpus.len()).collect();

        	// Randomize the ordering of the tournament vector which contains the indices of 
        	// the MessageSequences in the corpus. Then truncate the tournament down to tournament_size
        	tournament.shuffle(&mut rng);
        	tournament.truncate(tournament_size);

	        // Initialize a best index and best fitness
			let mut best_index: usize = 0;
			if tournament.len() != 0 {
				best_index = tournament[0];
			} 

	        let mut best_fitness: f32 = self.corpus[best_index].fitness;

	        // Loop through the indices in the tournament and compare the best
	        // index to all other indices but skip the first to avoid
	        // comparing the initialized best index to itself
	        for &index in tournament.iter().skip(1) {
	        	let fitness: f32 = self.corpus[index].fitness;

	            	if fitness > best_fitness {
	                	best_fitness = fitness;
	                	best_index = index;
	            	}
	        }

	        // Store the winner of the tournament
	        selected_indices.push(best_index);
		}

		return selected_indices;
	}

	fn evaluate_fitness(
		&mut self, 
		corpus: &mut Vec<MessageSequence<P>>, 
		corpus_trace: &Vec<Vec<(Message<P>, Response)>>,
		unique_server_states_visited: &[usize], 
		rare_server_states: &HashSet<P::ServerState>,
		state_coverage_weight: f32,
		response_time_weight: f32,
		state_roc_weight: f32,
		state_rarity_weight: f32,
		) {

		let total_unique_states = self.state_model.count_unique_server_states();

	    for (i, message_sequence) in corpus.iter_mut().enumerate() {
	    	// The proportion of unique ServerStates prompted by an individual MessageSequence out of 
			// all unique ServerStates vistted throughout the running of the program. This evaluates how well
			// the MessageSequence contributes to exploring the entire state space of the server
	        let coverage_score = unique_server_states_visited[i] as f32 / total_unique_states as f32;

	        // The proportion of unique ServerStates visited during the course of a single MessageSequence.
	        // This evaluates how well the message sequence is at trodding a productive path through the state space
	        let rate_of_change_score = unique_server_states_visited[i] as f32 / message_sequence.messages.len() as f32;

 			// Of the ServerStates prompted by the MessageSequence, the proportion of them which can be found
 			// in rare_server_states is the rarity_score. This evaluates how effective the message sequence is 
 			// at getting the server into rare states. We also get the average server_response_times
	        let mut rare_states_count = 0;
	        let mut response_time_score = 0.0;
	        for (message, response) in &corpus_trace[i] {
	        	response_time_score += message.response_time / 5.0;
	            let target_state = self.protocol.parse_response(&response);
	            if rare_server_states.contains(&target_state) {
	                rare_states_count += 1;
	            }
	        }

	        response_time_score = response_time_score / (corpus_trace[i].len() as f32);

	        let rarity_score = rare_states_count as f32 / corpus_trace[i].len() as f32;

	        // Combine the three scores with their respective weights to compute the final fitness
	        let fitness = coverage_score * state_coverage_weight
	        	+ response_time_score * response_time_weight
	            + rate_of_change_score * state_roc_weight
	            + rarity_score * state_rarity_weight;

	        message_sequence.fitness = fitness;
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
    fn crossover_corpus(&mut self, message_sequence_crossover_rate: f32, message_crossover_rate: f32) {
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

	fn create_new_generation(
		&mut self, 
		mating_pool: &[usize], 
		sequence_crossover_rate: f32, 
		sequence_mutation_rate: f32, 
		message_crossover_rate: f32, 
		message_mutation_rate: f32
		) {

	    // Update the corpus by cloning the selected individuals from the mating pool
	    self.corpus = mating_pool.iter().map(|&idx| self.corpus[idx].clone()).collect();

	    // Perform crossover on the MessageSequences within the new generation
	    self.crossover_corpus(sequence_crossover_rate, message_crossover_rate);

	    // Go through each MessageSequence within the new generation and mutate it according to the mutation rates
	    self.mutate_corpus(sequence_mutation_rate, message_mutation_rate);
	}

	fn get_fitness_stats(&mut self, print_flag: bool) -> (f32, f32, f32) {
		let mut total_fitness: f32 = 0.0;
		let mut min_fitness: f32 = f32::MAX;
		let mut max_fitness: f32 = f32::MIN;

		for individual in &self.corpus {
			total_fitness += individual.fitness;
			min_fitness = min_fitness.min(individual.fitness);
			max_fitness = max_fitness.max(individual.fitness);
		}

		let average_fitness = total_fitness / self.corpus.len() as f32;

		if print_flag {
			println!("    AVERAGE FITNESS: {:?}", average_fitness);
		}

		return (min_fitness, average_fitness, max_fitness);
	}

	pub fn evaluate(&mut self) -> f32 {
		// Calculate the slope of the best fit line which passes through the average fitness points recorded in the fitness.csv file
		let mut rdr = Reader::from_path("../resources/fitness.csv").unwrap();
		let mut x: Vec<f32> = Vec::new();
		let mut y: Vec<f32> = Vec::new();

		for result in rdr.records() {
			let record = result.unwrap();
			x.push(record[0].parse::<f32>().unwrap());
			y.push(record[2].parse::<f32>().unwrap());
		}

        let n = x.len() as f32;

        // calculate sums
        let sum_x: f32 = x.iter().sum();
        let sum_y: f32 = y.iter().sum();
        let sum_x_squared: f32 = x.iter().map(|&v| v.powi(2)).sum();
        let sum_xy: f32 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();

        // apply the formula to calculate the slope
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x.powi(2));

        slope
	}

	pub fn fuzz(&mut self, config: FuzzConfig, print_flag: bool) {
		// Initialize a CSV writer that writes into a file named "fitness.csv"
		let mut wtr = Writer::from_path("../resources/fitness.csv").unwrap();

		// Write the header
		wtr.write_record(&["generation", "min_fitness", "average_fitness", "max_fitness"]).unwrap();

		for j in 0..config.generations {
			if print_flag {
				println!("GENERATION {}", j);
			}

			// If corpus ever becomes empty, we need to repopulate it
			if self.corpus.is_empty() {
				let mut rng = rand::thread_rng();
				let num_message_sequences = rng.gen_range(2..10);

				for _ in 0..num_message_sequences {
					let sequence_len = rng.gen_range(1..10);
					let message_sequence = MessageSequence::random_message_sequence(self.protocol.clone(), sequence_len);
					self.corpus.push(message_sequence);
				}
			}

			let corpus_len: usize = self.corpus.len();
			let mut message_sequence: MessageSequence<P>;
			let mut interaction_history: Vec<(Message<P>, Response)>;
			let mut corpus_trace: Vec<Vec<(Message<P>, Response)>> = Vec::new();

			let mut rng = rand::thread_rng();

			// Iterate over the corpus and run each MessageSequence

			if print_flag {
				println!("    RUNNING CORPUS ...");
			}

			for i in 0..corpus_len {
				message_sequence = self.corpus[i].clone();
				interaction_history = self.run_message_sequence(&message_sequence);
				corpus_trace.push(interaction_history);

		        // Update message_pool with a random message from the current message_sequence at the defined rate
    			if rng.gen_range(0.0..1.0) < config.pool_update_rate {
    				if self.message_pool.len() == config.message_pool_size && config.message_pool_size > 0 {
    					let random_index = rng.gen_range(0..config.message_pool_size);
    					self.message_pool.remove(random_index);
    				}

					if message_sequence.messages.len() > 0 {
        				let random_message_idx = rng.gen_range(0..message_sequence.messages.len());
        				let random_message = message_sequence.messages[random_message_idx].clone();
        				self.message_pool.push(random_message);
					}
    			}
			}

			// Process the the corpus_trace to get the ServerStates needed to update the StateModel
			if print_flag {
				println!("    PROCESSING CORPUS TRACE AND UPDATING STATE MODEL ...");
			}
			let (state_transitions, unique_server_states_visited): (Vec<StateTransition<P::ServerState, P>>, Vec<usize>) = self.process_trace(&corpus_trace[..]);
			self.update_state_model(state_transitions);

			// Identify rare server states
        	let rare_server_states = self.identify_rare_server_states(config.state_rarity_threshold);

        	// Compute fitness of each MessageSequence in the corpus
			if print_flag {
				println!("    COMPUTING FITNESSES ...");
			}
        	let mut corpus_clone = self.corpus.clone();
        	self.evaluate_fitness(&mut corpus_clone, &corpus_trace, &unique_server_states_visited, &rare_server_states, 
        						  config.state_coverage_weight, config.response_time_weight, config.state_roc_weight, config.state_rarity_weight);
			
			self.corpus = corpus_clone.to_vec();

			// The mating_pool contains all the MessageSequences from the corpus which selected amongst
			// the tournaments run. This represents the pre-mutated and pre-crossed over new generation
			let mating_pool: Vec<usize> = self.tournament_selection(config.selection_pressure);

			// Get fitness stats and save them to CSV.
			let (min_fitness, avg_fitness, max_fitness) = self.get_fitness_stats(print_flag);
			wtr.write_record(&[
				j.to_string(),
				min_fitness.to_string(),
				avg_fitness.to_string(),
				max_fitness.to_string(),
			]).unwrap();

			// Apply crossover and mutation on the corpus to create the new generation
			if print_flag {
				println!("    CREATING NEW GENERATION ...");
			}
			self.create_new_generation(&mating_pool, config.sequence_crossover_rate, config.sequence_mutation_rate, 
									   config.message_crossover_rate, config.message_mutation_rate);
		}

		// After running the fuzzer...
		let dot_string = self.state_model.to_dot_string();
		std::fs::write("../resources/state_model.dot", dot_string).expect("Unable to write to file");

		// Flush the writer to ensure all records are written to the file.
		wtr.flush().unwrap();
	}
}