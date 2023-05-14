#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]

use core::panic;
use std::hash::Hash;
use std::collections::HashMap;
use std::cmp::PartialEq;
use std::fmt::Debug;
use rand::Rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::fmt::Formatter;
use std::fmt;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv4::Ipv4;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use std::net::Ipv4Addr;
use rand;
use rand::distributions::Alphanumeric;

use crate::Protocol;
use crate::Message;
use crate::Response;
use crate::MessageSequence;
use crate::Transport;


#[derive(Clone, PartialEq)]
pub struct SMTP;

impl Protocol for SMTP {
    type MessageType = SMTPMessageType;
    type MessageSectionsKey = SMTPMessageSectionsKey;
    type MessageSectionsValue = SMTPMessageSectionsValue;

    type ServerState = SMTPServerState;

    fn random_message(&self) -> Message<Self> {
        let mut rng = rand::thread_rng();
        let message_types = SMTPMessageType::iter().collect::<Vec<_>>();
        let index = rng.gen_range(0..message_types.len());
        let selected_message_type = message_types[index].clone();

        // Generate random domains and email addresses
        let mut rng = rand::thread_rng();
        let sender_local_length = rng.gen_range(1..=100);
        let recipient_local_length = rng.gen_range(1..=100);
        let sender_domain_length = rng.gen_range(1..=100);
        let recipient_domain_length = rng.gen_range(1..=100);

        let sender_local_part: String = (0..sender_local_length)
            .map(|_| { rng.sample(Alphanumeric) as char })
            .collect();

        let recipient_local_part: String = (0..recipient_local_length)
            .map(|_| { rng.sample(Alphanumeric) as char })
            .collect();

        let sender_domain_part: String = (0..sender_domain_length)
            .map(|_| { rng.sample(Alphanumeric) as char })
            .collect();

        let recipient_domain_part: String = (0..recipient_domain_length)
            .map(|_| { rng.sample(Alphanumeric) as char })
            .collect();

        let sender_domain = format!("{}.com", sender_domain_part);
        let recipient_domain = format!("{}.com", recipient_domain_part);
        let sender_email_address = format!("{}@{}.com", sender_local_part, sender_domain_part);
        let recipient_email_address = format!("{}@{}.com", recipient_local_part, recipient_domain_part);

        let mut sections: HashMap<SMTPMessageSectionsKey, SMTPMessageSectionsValue> = HashMap::new();

        match selected_message_type {
            SMTPMessageType::HELO => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("HELO")),
                );

                 // Call to random domain generator should be made here
                sections.insert(
                    SMTPMessageSectionsKey::Domain,
                    SMTPMessageSectionsValue::DomainValue(String::from(" ")+&sender_domain+&"\r\n"),
                );
            }
            SMTPMessageType::EHLO => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("EHLO")),
                );

                 // Call to random domain generator should be made here
                sections.insert(
                    SMTPMessageSectionsKey::Domain,
                    SMTPMessageSectionsValue::DomainValue(String::from(" ")+&sender_domain+&"\r\n"),
                );
            }
            SMTPMessageType::MAIL_FROM => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("MAIL FROM")),
                );

                 // Call to random email generator should be made here
                sections.insert(
                    SMTPMessageSectionsKey::EmailAddress,
                    SMTPMessageSectionsValue::EmailAddressValue(String::from(":<")+&sender_email_address+&">\r\n"),
                );
            }
            SMTPMessageType::RCPT_TO => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("RCPT TO")),
                );

                 // Call to random email generator should be made here
                sections.insert(
                    SMTPMessageSectionsKey::EmailAddress,
                    SMTPMessageSectionsValue::EmailAddressValue(String::from(":<")+&recipient_email_address+&">\r\n"),
                );
            }
            SMTPMessageType::DATA => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("DATA\r\n")),
                );
            },
            SMTPMessageType::EMAIL_CONTENT => {
                // Generate random From, To, and Subject fields.
                let from = format!("From: <{}>\r\n", sender_email_address);
                let to = format!("To: <{}>\r\n", recipient_email_address);
                let subject = format!("Subject: {}{}\r\n", sender_domain, recipient_domain);
            
                // Generate a random body.
                let mut rng = rand::thread_rng();
                let body_length = rng.gen_range(1..=1000);
                let mut body: String = (0..body_length)
                    .map(|_| {
                        if rng.gen_bool(0.85) {
                            // 85% of the time, generate an alphanumeric character.
                            rng.sample(Alphanumeric) as char
                        } else {
                            // 15% of the time, generate a special character from the given range.
                            rng.gen_range(33_u8..=47_u8) as char
                        }
                    })
                    .collect();
                body.push_str("\r\n.\r\n");  // append the end-of-data signal
            
                // Combine the headers and body.
                let email_content = from + &to + &subject + "\r\n" + &body;
            
                sections.insert(
                    SMTPMessageSectionsKey::PlainText,
                    SMTPMessageSectionsValue::PlainTextValue(email_content),
                );
            },                     
            SMTPMessageType::QUIT => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("QUIT\r\n")),
                );
            }
            SMTPMessageType::RSET => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("RSET\r\n")),
                );
            }
        }

        let response_time = 0.0;
        let mut data: Vec<u8> = Vec::new();

        for (_, value) in sections.iter() {
            match value {
                SMTPMessageSectionsValue::CommandValue(s)
                | SMTPMessageSectionsValue::DomainValue(s)
                | SMTPMessageSectionsValue::EmailAddressValue(s)
                | SMTPMessageSectionsValue::PlainTextValue(s) => {
                    data.extend(s.as_bytes());
                }
            }
        }

        Message {
            protocol: SMTP,
            data,
            message_type: selected_message_type,
            response_time,
            sections,
        }
    }

    fn build_message(&self, message_bytes: &[u8]) -> Message<Self> {
        // Since SMTP is a text-based protocol, we can simply convert the given bytes to a string.
        let message_string = String::from_utf8_lossy(message_bytes).to_string();

        // Split the message string into lines.
        let lines = message_string.split("\r\n").collect::<Vec<&str>>();

        // message_bytes may or may not contain a command.
        let potential_command = lines[0].split(" ").collect::<Vec<&str>>()[0];

        let message_type = if ["HELO", "EHLO", "MAIL FROM", "RCPT TO", "DATA", "QUIT", "RSET"].contains(&potential_command) {
            match potential_command {
                "HELO" => SMTPMessageType::HELO,
                "EHLO" => SMTPMessageType::EHLO,
                "MAIL FROM" => SMTPMessageType::MAIL_FROM,
                "RCPT TO" => SMTPMessageType::RCPT_TO,
                "DATA" => SMTPMessageType::DATA,
                "QUIT" => SMTPMessageType::QUIT,
                "RSET" => SMTPMessageType::RSET,
                _ => panic!("Invalid message type."),
            }
        } else {
            // If we didn't find a known command, we assume it's the headers/body of an email.
            SMTPMessageType::EMAIL_CONTENT
        };

        // Create a HashMap to store the message sections.
        let mut sections: HashMap<SMTPMessageSectionsKey, SMTPMessageSectionsValue> = HashMap::new();

        // Populate the sections HashMap based on the message type.
        match message_type {
            SMTPMessageType::HELO => {
                let message = lines[0].split(" ").collect::<Vec<&str>>();
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(message[0])),
                );

                sections.insert(
                    SMTPMessageSectionsKey::Domain,
                    SMTPMessageSectionsValue::DomainValue(String::from(" ") + message[1] + "\r\n"),
                );
            },
            SMTPMessageType::EHLO => {
                let message = lines[0].split(" ").collect::<Vec<&str>>();
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(message[0])),
                );

                sections.insert(
                    SMTPMessageSectionsKey::Domain,
                    SMTPMessageSectionsValue::DomainValue(String::from(" ") + message[1] + "\r\n"),
                );
            },
            SMTPMessageType::MAIL_FROM => {
                let message = lines[0].split(":").collect::<Vec<&str>>();
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(message[0])),
                );

                sections.insert(
                    SMTPMessageSectionsKey::EmailAddress,
                    SMTPMessageSectionsValue::EmailAddressValue(String::from(":") + message[1] + "\r\n"),
                );
            },
            SMTPMessageType::RCPT_TO => {
                let message = lines[0].split(":").collect::<Vec<&str>>();
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(message[0])),
                );

                sections.insert(
                    SMTPMessageSectionsKey::EmailAddress,
                    SMTPMessageSectionsValue::EmailAddressValue(String::from(":") + message[1] + "\r\n"),
                );
            },
            SMTPMessageType::DATA => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(lines[0]) + "\r\n"),
                );
            },
            SMTPMessageType::EMAIL_CONTENT => {
                sections.insert(
                    SMTPMessageSectionsKey::PlainText,
                    SMTPMessageSectionsValue::PlainTextValue(message_string),
                );
            }
            SMTPMessageType::QUIT => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(lines[0]) + "\r\n"),
                );
            },
            SMTPMessageType::RSET => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from(lines[0]) + "\r\n"),
                );
            },
        };
        todo!();
    }

    fn mutate_message(&self, message: &Message<Self>) -> Message<Self> {
        // Mutate the given message for your protocol and return the mutated message.
        todo!();
    }

    fn crossover_messages(&self, message1: &Message<Self>, message2: &Message<Self>) -> (Message<Self>, Message<Self>) {
        // Perform crossover on the given messages for your protocol and return the resulting pair of messages.
        todo!();
    }

    fn parse_response(&self, response: &Response) -> Self::ServerState {
        // Parse the given response and return the corresponding server state for your protocol.
        todo!();
    }

    // This method takes a path to a file path to a pcap file as an argument and extracts out 
    // the SMTP messages from the pcap file, then processes the messages in order to determine
    // which seqeunces to bundle together as a MessageSequence. Finally, the collection
    // of all MessageSequences are returned as a vector.
    fn parse_pcap(&self, pcap_file: &str, server_socket: &str) -> Vec<MessageSequence<Self>> {
        
        // Get the server address and port from the given socket and convert them to Ipv4Addr and u16 types, respectively.
        let server_ip_str = server_socket.split(":").collect::<Vec<_>>()[0];
        let server_address = server_ip_str.parse::<Ipv4Addr>().unwrap();
        let server_port = server_socket.split(":").collect::<Vec<_>>()[1].parse::<u16>().unwrap();

        // HashMap to store the payloads of each request.
        let mut request_payloads: HashMap<u32, Vec<Vec<u8>>> = HashMap::new();

        let mut cap = Capture::from_file(pcap_file).unwrap();

        while let Ok(packet) = cap.next() {

            let packet_data = packet.data.to_owned();
            
            // Parse Ethernet, IP, and TCP headers to get application layer data.
            let ethernet = EthernetPacket::new(&packet_data).unwrap();
            let ip = Ipv4Packet::new(ethernet.payload()).unwrap();
            let tcp = TcpPacket::new(ip.payload()).unwrap();

            let dst_ip = ip.get_destination();
            let dst_port = tcp.get_destination();

            // Check if the packet is a request or response.
            let is_request = dst_ip == server_address && dst_port == server_port;

            // If the packet is a request, then we need to get the data from the TCP payload.
            // We want to group the TCP payloads based on the TCP sequence number.
            // After all the payloads with the same sequence number have been collected, we 
            // can then combine them into a single payload and create a Message from it by 
            // sending the combined data to the build_message method.
            //
            // We will use request_payloads HashMap to store the payloads. The key will be the sequence
            // number and the value will be a vector of the payloads with that sequence number.

            if is_request {
                let seq_num = tcp.get_sequence();
                let payload = tcp.payload().to_owned();

                // Check if the sequence number is already in the map.
                if let Some(payloads) = request_payloads.get_mut(&seq_num) {
                    // If the sequence number is already in the map, then we need to append the payload to the existing vector.
                    payloads.push(payload);
                } else {
                    // If the sequence number is not in the map, then we need to create a new vector and add it to the map.
                    let payloads = vec![payload];
                    request_payloads.insert(seq_num, payloads);
                }
            } 
        }

        // We can now combine the payload groups into a single payloads and create a Messages from it.
        let mut messages: Vec<Message<Self>> = Vec::new();

        // Iterate through the request_payloads HashMap and combine the payloads into a single payload.
        for (_, payloads) in request_payloads.iter() {
            let mut data: Vec<u8> = Vec::new();

            for payload in payloads.iter() {
                data.extend(payload);
            }

            // Create a Message from the combined payload.
            let message = self.build_message(&data);
            messages.push(message);
        }

        let mut message_sequences: Vec<MessageSequence<Self>> = Vec::new();
        message_sequences
    }
}

// Define your protocol-specific types below.

#[derive(EnumIter, PartialEq, Clone, Debug)]
pub enum SMTPMessageType {
    HELO,
    EHLO,
    MAIL_FROM,
    RCPT_TO,
    DATA,
    EMAIL_CONTENT,
    QUIT,
    RSET,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum SMTPMessageSectionsKey {
    Command,
    Domain,
    EmailAddress,
    PlainText,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SMTPMessageSectionsValue {
    CommandValue(String),
    DomainValue(String),
    EmailAddressValue(String),
    PlainTextValue(String),
}

impl Default for SMTPMessageType {
    fn default() -> Self {
        // Default SMTPMessageType
        todo!();
    }
}

impl Default for SMTPMessageSectionsKey {
    fn default() -> Self {
        // Default SMTPMessageSectionsKey, like Self::Header or Self::Payload
        todo!();
    }
}

impl Default for SMTPMessageSectionsValue {
    fn default() -> Self {
        // Default SMTPMessageSectionsValue
        todo!();
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SMTPServerState {
    // Fields which make up structure of SMTP's server responses
}

impl Debug for SMTPServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        /*
        write!(
            f,
            "{} - {}: {}",
            self.field1, self.field2, self.field3, etc
        )
        */
        todo!();
    }
}
