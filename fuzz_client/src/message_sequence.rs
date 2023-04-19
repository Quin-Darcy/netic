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
}





