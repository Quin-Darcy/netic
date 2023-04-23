use rand::prelude::*;

use crate::Protocol;
use crate::message::Message;

pub struct MessageSequence<P: Protocol> {
    pub messages: Vec<Message<P>>,
    pub timings: Vec<f32>,
    pub fitness: f32,
}

impl<P: Protocol> MessageSequence<P> {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            timings: Vec::new(),
            fitness: 0.0,
        }
    }

    pub fn from_messages(messages: Vec<Message<P>>, timings: Vec<f32>) -> Self {
        Self {
            messages,
            timings,
            fitness: 0.0,
        }
    }

    pub fn random_message_sequence(protocol: P, sequence_length: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut messages: Vec<Message<P>> = Vec::new();
        let mut timings: Vec<f32> = Vec::new();

        for _ in 0..sequence_length {
            messages.push(Message::random_message(protocol.clone()));
        }

        for _ in 0..sequence_length - 1 {
            timings.push(rng.gen_range(1.0..2.0));
        }

        Self {
            messages,
            timings,
            fitness: 0.0,
        }
    }
}

impl<P: Protocol> PartialEq for MessageSequence<P> {
    fn eq(&self, other: &Self) -> bool {
        self.messages == other.messages
            && self.timings == other.timings
            && self.fitness == other.fitness
    }
}

impl<P: Protocol> Clone for MessageSequence<P> {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
            timings: self.timings.clone(),
            fitness: self.fitness.clone(),
        }
    }
}
