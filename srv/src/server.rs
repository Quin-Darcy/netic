#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

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
            let (stream, addr) = self.listener.accept().unwrap();
            println!("New client connected: {:?}", addr);

            {
                let mut client_stream = stream;
                loop {
                    let message_result = self.receive_message(&mut client_stream);

                    match message_result {
                        Ok(Some(message)) => {
                            println!("CLIENT: {}", String::from_utf8_lossy(&message.data));

                            let response = self.state_machine.handle_message(&message, &self.state_transition_rules);

                            println!("SERVER: {}", &response.response_string);

                            if let Err(e) = self.send_response(&mut client_stream, &response) {
                                eprintln!("Failed to write to client: {}", e);
                                println!("Breaking due to an error while reading from the client.");
                                break;
                            }
                        },
                        Ok(None) => {
                            // Client disconnected
                            println!("Client disconnected");
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to read from client: {}", e);
                            println!("Closing connection due to error");
                            break;
                        }
                    }
                }
                println!("Connection closed, waiting for new clients.");
            }
        }
    }
       

    fn receive_message(&mut self, mut stream: &TcpStream) -> Result<Option<Message>, std::io::Error> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            // Client disconnected
            return Ok(None);
        }

        Ok(Some(Message::new(&buffer[..bytes_read])))
    }

    fn send_response(&self, mut stream: &TcpStream, response: &Response) -> Result<(), std::io::Error> {
        let response_string = response.response_string.clone();
        stream.write(response_string.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

/*
    pub fn run() {
        loop {
            let (stream, addr) = self.listener.accept().unwrap();
            println!("New client connected: {:?}", addr);

            {
                let mut client_stream = stream;
                loop {
                    let message_result = self.receive_message(&mut client_stream);

                    match message_result {
                        Ok(Some(message)) => {
                            println!("CLIENT: {}", String::from_utf8_lossy(&message.data));

                            let response = self.state_machine.handle_message(&message, &self.state_transition_rules);

                            println!("SERVER: {}", &response.response_string);

                            if let Err(e) = self.send_response(&mut client_stream, &response) {
                                eprintln!("Failed to write to client: {}", e);
                                println!("Breaking due to an error while reading from the client.");
                                break;
                            }
                        },
                        Ok(None) => {
                            // Client disconnected
                            println!("Client disconnected");
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to read from client: {}", e);
                            println!("Closing connection due to error");
                            break;
                        }
                    }
                }
                println!("Connection closed, waiting for new clients.");
            }
        }
    }
    */
