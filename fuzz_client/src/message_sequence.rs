#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]

use rand::prelude::*;
use rand::Rng;

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

    pub fn mutate_message_sequence(&mut self, protocol: P, message_mutation_rate: f32, message_pool: &[Message<P>]) {
        // Randomly select which mutation type the MessageSequence will undergo
        let mut rng = rand::thread_rng();
        let mutation_type = rng.gen_range(0..5);

        match mutation_type {
            0 => {
                // Deletion of random message
                if !self.messages.is_empty() {
                    let message_index = rng.gen_range(0..self.messages.len());
                    self.messages.remove(message_index);
                }
            }
            1 => {
                // Addition of random message from random "stock" message 
                // or recent message from message_pool
                let message_index = rng.gen_range(0..=self.messages.len());
                let message_to_add = if rng.gen_bool(0.5) {
                    Message::random_message(protocol)
                } else if !message_pool.is_empty() {
                    message_pool.choose(&mut rng).unwrap().clone()
                } else {
                    Message::random_message(protocol)
                };

                self.messages.insert(message_index, message_to_add);
            }
            2 => {
                // Shuffle message order in sequence
                self.messages.shuffle(&mut rng);
            }
            3 => {
                // Substitute random message with stock message or one from pool
                if !self.messages.is_empty() {
                    let message_index = rng.gen_range(0..self.messages.len());
                    let message_to_add = if rng.gen_bool(0.5) {
                        Message::random_message(protocol)
                    } else if !message_pool.is_empty() {
                        message_pool.choose(&mut rng).unwrap().clone()
                    } else {
                        Message::random_message(protocol)
                    };

                    self.messages[message_index] = message_to_add;
                }
            }
            4 => {
                // Change random timing value by making it smaller or bigger
                if !self.timings.is_empty() {
                    let timing_index = rng.gen_range(0..self.timings.len());
                    let min_delta = -self.timings[timing_index] + 0.1;
                    self.timings[timing_index] += rng.gen_range(min_delta..1.0);
                }
            }
            _ => {}
        }

        // Run through each message in the sequence and determine if it gets mutated
        for i in 0..self.messages.len() {
            if rng.gen_range(0.0..1.0) < message_mutation_rate {
                self.messages[i].mutate_message();
            }
        }
    }

    pub fn crossover_message_sequences(&mut self, other: &MessageSequence<P>, message_crossover_rate: f32) -> (MessageSequence<P>, MessageSequence<P>) {
        // Two-point crossover method
        let mut rng = rand::thread_rng();

        let (small_parent, big_parent) = if self.messages.len() < other.messages.len() {
            (self.clone(), other.clone())
        } else {
            (other.clone(), self.clone())
        };

        let min_len = small_parent.messages.len();
        let max_len = big_parent.messages.len();

        let crossover_point1 = rng.gen_range(0..min_len);
        let crossover_point2 = rng.gen_range(crossover_point1..min_len);

        let mut small_offspring = small_parent.clone();
        let mut big_offspring = big_parent.clone();

        // This loop cross transplants the section of messages and timings 
        // in each sequence defined by the crossover points
        for i in crossover_point1..=crossover_point2 {
            small_offspring.messages[i] = big_parent.messages[i].clone();
            big_offspring.messages[i] = small_parent.messages[i].clone();

            // Since the timings vector is always one less than the length of the MessageSequence
            if i < std::cmp::min(small_parent.timings.len(), big_parent.timings.len()) {
                small_offspring.timings[i] = (small_parent.timings[i] + big_parent.timings[i]) / 2.0;
                big_offspring.timings[i] = (small_parent.timings[i] + big_parent.timings[i]) / 2.0;
            }
        }

        // Perform crossover on individual messages based on the message_crossover_rate
        for i in crossover_point1..=crossover_point2 {
            if rng.gen::<f32>() < message_crossover_rate {
                let (new_small_msg, new_big_msg) = small_offspring.messages[i].crossover_messages(&big_offspring.messages[i]);
                small_offspring.messages[i] = new_small_msg;
                big_offspring.messages[i] = new_big_msg;
            }
        }

        // Reset the offspring's fitnesses
        small_offspring.fitness = 0.0;
        big_offspring.fitness = 0.0;

        (small_offspring, big_offspring)
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
