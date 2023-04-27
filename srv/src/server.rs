use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

use crate::StateMachine;
use crate::ServerState;
use crate::StateTransitionRules;
use crate::Message;
use crate::Response;


pub struct Server {
    listener: TcpListener,
    state_machine: StateMachine,
    state_transition_rules: StateTransitionRules,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).unwrap();
        let state_machine = StateMachine::new();
        let state_transition_rules = StateTransitionRules::new();

        Self {
            listener,
            state_machine,
            state_transition_rules,
        }
    }

    pub fn run(&mut self) {
        loop {
            let message = self.receive_message();
            let response = self.state_machine.handle_message(&message, &self.state_transition_rules);
            self.send_response(response);
        }
    }

    fn receive_message(&mut self) -> Message {
        // Establish connection to the remote peer by calling accept() 
        let (mut stream, _) = self.listener.accept().unwrap();

        // Create the buffer which will store the bytes read from the stream
        let mut buffer = [0; 1024];

        // Read the bytes from the stream into the buffer
        let bytes_read = stream.read(&mut buffer).unwrap();

        // Return a new instance of Message from the bytes read from the stream
        return Message::new(&buffer[..bytes_read]);
    }

    fn send_response(&self, response: Response) {
        todo!();
    }
}
