use crate::Protocol;
use crate::message::Message;

pub struct MessageSequence<P: Protocol> {
    pub messages: Vec<Message<P>>,
    pub timings: Vec<f32>,
    pub fitness: f32,
}

impl<P: Protocol + Clone> MessageSequence<P> {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            timings: Vec::new(),
            fitness: 0.0,
        }
    }

    pub fn random_message_sequence(protocol: P, num_messages: usize) -> Self {
        let mut messages: Vec<Message<P>> = Vec::new();

        for _ in 0..num_messages {
            messages.push(Message::random_message(protocol.clone()));
        }

        let timings: Vec<f32> = vec![1.0; num_messages-1];

        Self {
            messages,
            timings,
            fitness: 0.0
        }
    }

    pub fn from_messages(messages: Vec<Message<P>>, timings: Vec<f32>) -> Self {
        Self {
            messages,
            timings,
            fitness: 0.0,
        }
    }
}





