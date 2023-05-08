use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream, UdpSocket};
use std::time::Duration;

use crate::Response;


const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

pub enum Transport {
    TCP(TcpStream),
    UDP(UdpSocket),
}

impl Transport {
    pub fn connect(protocol: TransportProtocol, server_address: &str) -> Result<Self, std::io::Error> {
        match protocol {
            TransportProtocol::TCP => {
                let tcp_stream = TcpStream::connect(server_address).expect("TCP: Could not connect to server");
                Ok(Self::TCP(tcp_stream))
            }
            TransportProtocol::UDP => {
                let udp_socket = UdpSocket::bind(server_address).expect("UDP: Could not bind to server");
                udp_socket.connect(server_address).expect("UDP: Could not connect to server");
                Ok(Self::UDP(udp_socket))
            }
        }
    }

    pub fn send(&mut self, message: &[u8]) {
        match self {
            Self::TCP(tcp_stream) => tcp_stream.write(message).expect("TCP: Failed to write to stream"),
            Self::UDP(udp_socket) => udp_socket.send(message).expect("UDP: Failed to write to server:"),
        };
    }

    pub fn receive(&mut self) -> Result<Response, std::io::Error> {
        match self {
            Self::TCP(tcp_stream) => {
                let mut reader = BufReader::new(tcp_stream);
                match Self::read_response_tcp(&mut reader) {
                    Ok(buffer) => Ok(Response::new(buffer)),
                    Err(e) => Err(e),
                }
            },
            Self::UDP(udp_socket) => {
                udp_socket.set_read_timeout(Some(RESPONSE_TIMEOUT))
                    .expect("Failed to set read timeout");

                let mut buffer = vec![0; 1024];
                let bytes_received = udp_socket.recv(&mut buffer)?;
                buffer.truncate(bytes_received);
                Ok(Response::new(buffer))
            }
        }
    }

    fn read_response_tcp(reader: &mut BufReader<&mut TcpStream>) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer: Vec<u8> = Vec::new();
        
        // Set the read timeout
        reader.get_mut().set_read_timeout(Some(RESPONSE_TIMEOUT))
            .expect("Failed to set read timeout");
        
        let read_result = reader.read_until(b'\n', &mut buffer);
        match read_result {
            Ok(_) => Ok(buffer),
            Err(e) => Err(e),
        }
    }

    pub fn shutdown(&mut self) -> Result<(), std::io::Error> {
        match self {
            Self::TCP(tcp_stream) => tcp_stream.shutdown(Shutdown::Both),
            Self::UDP(_) => Ok(()), // No specific shutdown needed for UDP
        }
    }
}
