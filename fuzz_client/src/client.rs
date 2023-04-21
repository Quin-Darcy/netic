#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::str;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::net::{TcpStream, Shutdown};
use std::io::{self, BufRead, BufReader, Write};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use crate::Protocol; 
use crate::Message;
use crate::MessageSequence;
use crate::protocols::GreetingProtocol;


type StateModel = HashMap<ServerState, Vec<StateTransition>>;

pub struct Client<P: Protocol> {
	server_address: String,
	protocol: P,
	corpus: Vec<MessageSequence<P>>,
	state_model: StateModel,
	message_pool: Vec<Message<P>>, 
}

impl<P: Protocol + Clone> Client<P> {
    // Initialize new client with random corpus and message_pool
    pub fn new(server_address: String, protocol: P) -> Self {
        const MESSAGE_SEQUENCE_LENGTH: usize = 10;
        const MESSAGE_POOL_LENGTH: usize = 10;
        const INITIAL_CORPUS_LENGTH: usize = 10;

        let mut corpus = Vec::new();
        for _ in 0..INITIAL_CORPUS_LENGTH {
            corpus.push(MessageSequence::random_message_sequence(protocol.clone(), MESSAGE_SEQUENCE_LENGTH));
        }

        let mut message_pool = Vec::new();
        for _ in 0..MESSAGE_POOL_LENGTH {
            message_pool.push(Message::random_message(protocol.clone()));
        }

        Self {
            server_address,
            protocol,
            corpus,
            state_model: HashMap::new(),
            message_pool,
        }
    }
}