#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]

use std::hash::Hash;
use std::collections::HashMap;
use std::cmp::PartialEq;
use std::fmt::Debug;
use rand::Rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::fmt::Formatter;
use std::fmt;

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
                    SMTPMessageSectionsValue::DomainValue(String::from(" example.com\r\n")),
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
                    SMTPMessageSectionsValue::DomainValue(String::from(" example.com\r\n")),
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
                    SMTPMessageSectionsValue::EmailAddressValue(String::from(":<user@example.com>\r\n")),
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
                    SMTPMessageSectionsValue::EmailAddressValue(String::from(":<user@example.com>\r\n")),
                );
            }
            SMTPMessageType::DATA => {
                sections.insert(
                    SMTPMessageSectionsKey::Command,
                    SMTPMessageSectionsValue::CommandValue(String::from("DATA\r\n")),
                );
            }
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

#[derive(EnumIter, PartialEq, Clone, Debug)]
pub enum SMTPMessageType {
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
