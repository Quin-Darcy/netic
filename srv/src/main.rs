#![allow(unused_imports)]

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};


pub enum ServerState {
    Idle,
    Greeted,
    Questioned,
    Terminated,
}

pub enum ServerError {
    InvalidPayloadLength, 
    UnrecognizedHeader,
    UnrecognizedPayload,
    HeaderMismatch,
    NonUTF8Sequence,
    InvalidStateTransition,
}

fn receive_message(listener: &TcpListener) -> String {
    // Establish connection to the remote peer by calling accept() 
    let (mut stream, _) = listener.accept().unwrap();

    // Create the buffer which will store the bytes read from the stream
    let mut buffer = [0; 1024];

    // Read the bytes from the stream into the buffer
    let bytes_read = stream.read(&mut buffer).unwrap();

    // Convert the bytes read from the stream into an owned string
    // If the byte sequence contains any invalid UTF-8 sequences, 
    // they get replaced with the Unicode (U+FFFD) character.
    String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
}

fn main () {
    // Start listener on server to await incoming connections
    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();

    loop {
        let message = receive_message(&listener);
    }
}
