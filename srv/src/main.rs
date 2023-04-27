use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

use srv::StateMachine;
use srv::Message;


fn receive_message(listener: &TcpListener) -> Message {
    // Establish connection to the remote peer by calling accept() 
    let (mut stream, _) = listener.accept().unwrap();

    // Create the buffer which will store the bytes read from the stream
    let mut buffer = [0; 1024];

    // Read the bytes from the stream into the buffer
    let bytes_read = stream.read(&mut buffer).unwrap();

    // Return a new instance of Message from the bytes read from the stream
    return Message::new(&buffer[..bytes_read]);
}

fn main () {
    // Start listener on server to await incoming connections
    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();

    // Create an instance of a StateMachine
    let stae_machine = StateMachine::new();

    loop {
        let message: Message = receive_message(&listener);
    }
}
