#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

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
            // Establish connection to the remote by calling accept()
            let (stream, _) = self.listener.accept().unwrap();

            // Handle the client connection in a separate scope
            {
                let mut client_stream = stream;
                loop {
                    let message_result = self.receive_message(&mut client_stream);

                    match message_result {
                        Ok(message) => {
                            println!("CLIENT: {}", String::from_utf8_lossy(&message.data));

                            let response = self.state_machine.handle_message(&message, &self.state_transition_rules);

                            println!("SERVER: {}", &response.response_string);

                            // Check the result of send_response and break the loop if an error occurs
                            if let Err(e) = self.send_response(&mut client_stream, &response) {
                                eprintln!("Failed to write to client: {}", e);
                                break;
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to read from client: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn receive_message(&mut self, mut stream: &TcpStream) -> Result<Message, std::io::Error> {
        // Create the buffer which will store the bytes read from the stream
        let mut buffer = [0; 1024];

        // Read the bytes from the stream into the buffer
        let bytes_read = stream.read(&mut buffer).unwrap();

        // Return a new instance of Message from the bytes read from the stream
        Ok(Message::new(&buffer[..bytes_read]))
    }

    // Change the signature of the send_response method to return a Result<(), std::io::Error>
    fn send_response(&self, mut stream: &TcpStream, response: &Response) -> std::io::Result<()> {
        let response_string = response.response_string.clone();
        stream.write(response_string.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}
