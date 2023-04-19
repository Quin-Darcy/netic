#![allow(unused_imports)]

use std::str;
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, BufRead, BufReader, Read, Write, Error};

use chrono::Local;

#[derive(Debug, PartialEq, Clone)]
enum ServerState {
    Initial,
    Greeted,
    TimeRequested,
    Goodbye,
}

fn process_request(buffer: &Vec<u8>, state: &ServerState) -> (String, ServerState) {
    let message: &str = str::from_utf8(buffer)
        .expect("Could not write to buffer as string");

    println!("CLIENT: {:?}", message);

    let response: String;
    let new_state: ServerState;

    match (message, state) {
        ("Hello!\n", ServerState::Initial) => {
            response = String::from("200 OK: HELLO - Hello, client!\n");
            new_state = ServerState::Greeted;
        }
        ("What time is it?\n", ServerState::Greeted) => {
            let time: String = format!("{}\n", Local::now().format("%H:%M:%S"));
            let mut time_response: String = String::from("201 OK: TIME - ");
            time_response.push_str(time.as_str());
            response = time_response;
            new_state = ServerState::TimeRequested;
        }
        ("Goodbye!\n", ServerState::TimeRequested) => {
            response = String::from("202 OK: GOODBYE - Goodbye, client!\n");
            new_state = ServerState::Goodbye;
        }
        _ => {
            response = String::from("400 ERROR: INVALID_REQUEST - Invalid request received\n");
            new_state = state.clone();
        }
    }

    println!("SERVER: {:?}", response.as_str());
    return (response, new_state);
}

fn handle_client(mut stream: TcpStream) -> Result<(), Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);
    let mut server_state = ServerState::Initial;
    loop {
        let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();
        let response: String;
        let bytes_read: usize  = reader.read_until(b'\n', &mut buffer)
                                    .expect("Could not read into buffer");
        if bytes_read != 0 {
            let (response, new_state) = process_request(&buffer, &server_state);
            server_state = new_state;
            stream.write(response.as_bytes())?;
            if server_state == ServerState::Goodbye {
                stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            }
        } else {
            return Ok(());
        }
    }
}

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:8888")
                                .expect("Could not bind");

    for stream in listener.incoming() {
        match stream {
            Err(e) => { eprintln!("failed: {}", e) }
            Ok(stream) => {
                // Move the ownership of stream into new thread's closure
                thread::spawn(move || {
                    handle_client(stream)
                    .unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
}
