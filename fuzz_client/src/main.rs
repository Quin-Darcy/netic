#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::num;
use std::str::from_utf8;
use std::io::{self, Write};

use fuzz_client::Client;
use fuzz_client::GreetingProtocol;
use fuzz_client::SMTP;
use fuzz_client::FuzzConfig;
use fuzz_client::MessageSequence;
use fuzz_client::TransportProtocol;
use fuzz_client::Protocol;
use fuzz_client::Swarm;



fn main() {
    // User-provided server address and transport protocol
    let server_address = String::from("10.0.0.92:8025");
    let transport_protocol: TransportProtocol = TransportProtocol::TCP;
    let target_protocol: SMTP = SMTP {};

    let pcap_file = String::from("../resources/smtp.pcap");
    let pcap_corpus = target_protocol.parse_pcap(pcap_file.as_str(), server_address.as_str());

    // Create instance of Client
    let mut client = Client::new(server_address, transport_protocol, target_protocol);
    client.corpus = pcap_corpus;

    // Create instance of Swarm
    let num_particles = 10;
    let generations = 10;
    let message_pool_size = 50;
    let pso_iterations = 13;
    let inertial_weight = 1.0;
    let cognitive_weight = 1.0;
    let social_weight = 1.0;
    let regularization_strength = 0.25;
    let vmax = 0.5;

    let mut swarm = Swarm::new(
        num_particles,
        pso_iterations, 
        generations, 
        message_pool_size,
        inertial_weight,
        cognitive_weight,
        social_weight,
        regularization_strength,
        vmax,
    );

    // Run swarm and get set configs to swarm's global best
    swarm.run_swarm(&mut client);

    let mut pso_optimized_configs = swarm.global_best_position;

    // Run fuzzing with configs from swarm
    let generations = 20;
    pso_optimized_configs.generations = generations;

    print!("\nPRESS ENTER TO RUN FUZZER ... \n");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    client.fuzz(pso_optimized_configs, true);
}
