#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::hash::Hash;
use std::cmp::PartialEq;
use std::fmt::Debug;
use rand::Rng;
use strum_macros::EnumIter;

use crate::Message;
use crate::Response;
use crate::MessageSequence;


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
        let rand_message_type = message_types[index].clone();

        let sections: HashMap<SMTPMessageSectionsKey

        /*

        let possible_payloads: [&str; 3] = ["Hello!\n", "What time is it?\n", "Goodbye!\n"];
    
        let payload = possible_payloads.choose(&mut rng).unwrap();
        let (message_type, header) = match *payload {
            "Hello!\n" => (GreetingMessageType::Hello, [0x48, 0x45, 0x4C, 0x4F]),
            "What time is it?\n" => (GreetingMessageType::TimeRequest, [0x54, 0x49, 0x4D, 0x45]),
            "Goodbye!\n" => (GreetingMessageType::Goodbye, [0x42, 0x59, 0x45, 0x5F]),
            _ => unreachable!(),
        };

        let response_time: f32 = 0.0;

        let mut sections = HashMap::new();
        sections.insert(
            GreetingMessageSectionsKey::Header,
            GreetingMessageSectionsValue { header, ..Default::default() },
        );
        sections.insert(
            GreetingMessageSectionsKey::Length,
            GreetingMessageSectionsValue { length: payload.len() as u64, ..Default::default() },
        );
        sections.insert(
            GreetingMessageSectionsKey::Payload,
            GreetingMessageSectionsValue { payload: payload.as_bytes().to_vec(), ..Default::default() },
        );

        let data = [&header[..], &(payload.len() as u64).to_be_bytes(), &payload.as_bytes()].concat();

        Message {
            protocol: GreetingProtocol,
            data,
            message_type,
            response_time,
            sections,
        }
        */
    }

    fn build_message(&self, message_bytes: &[u8]) -> Message<Self> {
        // Build a message for your protocol from the given byte slice and return it.
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

    fn parse_pcap(&self, pcap_file: &str) -> Vec<MessageSequence<Self>> {
        todo!();
    }
}

// Define your protocol-specific types below.

#[derive(PartialEq, Clone, Debug)]
pub struct SMTPMessageType {
    HELO,
    EHLO,
    MAIL_FROM,
    RCPT_TO,
    DATA,
    QUIT,
    RSET,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum SMTPMessageSectionsKey {
    Domain,
    EmailAddress,
    PlainText,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SMTPMessageSectionsValue {
    DomainValue(String),
    EmailAddressValue {
        address: String,
        angle_brackets: bool,
    },
    PlainTextValue {
        text: String,
        newline_period: bool,
    },
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct SMTPServerState {
    // Add any required fields for your protocol server state here.
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
