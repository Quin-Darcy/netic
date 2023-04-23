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

struct TransitionRule {
    header: u32,
    payload: &'static str,
    from_state: ServerState,
    response: &'static str,
    to_state: ServerState,
}

fn transition_rules() -> Vec<TransitionRule> {
    vec![
        TransitionRule {
            header: 0x48454c4f,
            payload: "Hello!\n",
            from_state: ServerState::Initial,
            response: "200;OK;Hello, client!\n",
            to_state: ServerState::Greeted,
        },
        TransitionRule {
            header: 0x5449455f,
            payload: "What time is it?\n",
            from_state: ServerState::Greeted,
            response: "201;OK;",
            to_state: ServerState::TimeRequested,
        },
        TransitionRule {
            header: 0x4259455f,
            payload: "Goodbye!\n",
            from_state: ServerState::TimeRequested,
            response: "202;OK;Goodbye, client!\n",
            to_state: ServerState::Goodbye,
        },
    ]
}

fn apply_rules(header: u32, payload: &str, state: &ServerState, rules: &[TransitionRule]) -> (String, ServerState) {
    for rule in rules {
        if header == rule.header && payload == rule.payload && state == &rule.from_state {
            let mut response = String::from(rule.response);
            if rule.response.ends_with(";") {
                let time = format!("{}\n", Local::now().format("%H:%M:%S"));
                response.push_str(time.as_str());
            }
            return (response, rule.to_state.clone());
        }
    }

    // If no rules match, return an error response
    (String::from("400;ERROR;Invalid request received\n"), state.clone())
}

fn extract_header_and_payload(buffer: &[u8]) -> (u32, String) {
    let header = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let payload_length = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;
    let payload = String::from_utf8(buffer[8..(8 + payload_length)].to_vec()).unwrap();

    (header, payload)
}

fn process_request(header: u32, payload: &str, state: &ServerState) -> (String, ServerState) {
    println!("CLIENT: {:?}", payload);

    let rules = transition_rules();
    let (response, new_state) = apply_rules(header, payload, state, &rules);

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
        let bytes_read: usize = reader.read_until(b'\n', &mut buffer).expect("Could not read into buffer");

        if bytes_read != 0 {
            // Extract header and payload from the message
            let (header, payload) = extract_header_and_payload(&buffer);

            let (response, new_state) = process_request(header, &payload, &server_state);
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
