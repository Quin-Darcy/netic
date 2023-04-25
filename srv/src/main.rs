#![allow(unused_imports)]

use std::str;
use std::thread;
use std::io::{Error, ErrorKind};
use std::string::FromUtf8Error;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, BufRead, BufReader, Read, Write};

use chrono::Local;

#[derive(Debug)]
enum ServerError {
    InvalidBufferSize,
    NonAsciiPayload,
    Utf8Error(FromUtf8Error),
}

#[derive(Debug, PartialEq, Clone)]
enum ServerState {
    Initial,
    Greeted,
    TimeRequested,
    Goodbye,
    NonAsciiPayload, // New state
    InvalidState, // New state
    InvalidBufferSize, // New state
    UTFError, // New stae
    Terminated // New state
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
            header: 0x54494d45, 
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
        // New rules
        TransitionRule {
            header: 0x54494d45,
            payload: "What time is it?\n",
            from_state: ServerState::NonAsciiPayload,
            response: "400;ERROR;Cannot process time request after non-ASCII payload\n",
            to_state: ServerState::InvalidState,
        },
        TransitionRule {
            header: 0x4259455f,
            payload: "Goodbye!\n",
            from_state: ServerState::InvalidState,
            response: "400;ERROR;Cannot process goodbye in invalid state\n",
            to_state: ServerState::InvalidState,
        },
        // If a non-ASCII payload is encountered, keep server state as NonAsciiPayload
        TransitionRule {
            header: 0x54494d45,
            payload: "What time is it?\n",
            from_state: ServerState::NonAsciiPayload,
            response: "400;ERROR;Cannot process time request after non-ASCII payload\n",
            to_state: ServerState::NonAsciiPayload,
        },
        TransitionRule {
            header: 0x4259455f,
            payload: "Goodbye!\n",
            from_state: ServerState::NonAsciiPayload,
            response: "400;ERROR;Cannot process goodbye after non-ASCII payload\n",
            to_state: ServerState::NonAsciiPayload,
        },
    ]
}

fn apply_rules(header: u32, payload: Option<&str>, state: &ServerState, rules: &[TransitionRule]) -> (String, ServerState) {
    println!("apply_rules");
    for rule in rules {
        if header == rule.header && payload.as_ref().map(|s| *s) == Some(rule.payload) && state == &rule.from_state {
            let mut response = String::from(rule.response);
            if rule.response.ends_with(";") {
                let time = format!("{}\n", Local::now().format("%H:%M:%S"));
                response.push_str(time.as_str());
            }
            return (response, rule.to_state.clone());
        }
    }

    // If no rules match, return a specific error response based on the current state
    match state {
        ServerState::NonAsciiPayload => (String::from("400;ERROR;Invalid request after non-ASCII payload\n"), state.clone()),
        ServerState::InvalidState => (String::from("400;ERROR;Invalid request in invalid state\n"), state.clone()),
        ServerState::InvalidBufferSize => (String::from("400;ERROR;Invalid request with invalid buffer size\n"), state.clone()),
        _ => (String::from("400;ERROR;Invalid request received\n"), state.clone()),
    }
}

fn process_request(header: u32, payload: Option<&str>, state: &ServerState) -> (String, ServerState) {
    println!("process_request");
    println!("CLIENT: {:?}", payload.unwrap());

    let rules = transition_rules();
    let (response, new_state) = apply_rules(header, payload, state, &rules);

    println!("SERVER: {:?}", response.as_str());
    return (response, new_state);
}

fn extract_header_and_payload(buffer: &[u8]) -> Result<(u32, String), ServerError> {
    println!("extract_header_and_payload");
    if buffer.len() < 12 {
        return Err(ServerError::InvalidBufferSize);
    }

    let header = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let payload_length = u64::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10], buffer[11]]) as usize;

    // Check if the payload length is within a reasonable range (e.g., less than the buffer size)
    if payload_length + 12 > buffer.len() || payload_length > buffer.len() - 12 {
        return Err(ServerError::InvalidBufferSize);
    }

    let payload = String::from_utf8_lossy(&buffer[12..(12 + payload_length)]).to_string();

    // Check if there are any non-ASCII characters in the payload
    if payload.chars().any(|c| c.is_ascii() == false) {
        return Err(ServerError::NonAsciiPayload);
    }

    Ok((header, payload))
}



fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    println!("handle client");
    let mut buffer = [0; 1024];
    let mut server_state = ServerState::Initial;

    loop {
        let bytes_read = stream.read(&mut buffer).expect("Unable to read from stream");

        if bytes_read == 0 {
            stream.shutdown(Shutdown::Both)?;
            return Ok(());
        }

        let (header, payload) = match extract_header_and_payload(&buffer[0..bytes_read]) {
            Ok((header, payload)) => (header, Some(payload)),
            Err(ServerError::InvalidBufferSize) => {
                server_state = ServerState::InvalidBufferSize;
                let response = "500;ERROR;Invalid buffer size received\n";
                stream.write(response.as_bytes())?;
                continue;
            }
            Err(ServerError::NonAsciiPayload) => {
                server_state = ServerState::NonAsciiPayload;
                let response = "501;ERROR;Invalid characters in payload";
                stream.write(response.as_bytes())?;
                continue;
            }
            Err(ServerError::Utf8Error(_)) => {
                server_state = ServerState::UTFError;
                let response = "502;ERROR;UTF error";
                stream.write(response.as_bytes())?;
                continue;
            }
        };

        let (response, new_state) = process_request(header, payload.as_deref(), &server_state);

        stream.write(response.as_bytes())?;

        if new_state == ServerState::Terminated {
            stream.shutdown(Shutdown::Both)?;
            return Ok(());
        } else {
            server_state = new_state;
        }

        // Clear the buffer and reset bytes_read before the next iteration
        buffer = [0; 1024];
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
