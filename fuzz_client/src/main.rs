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
use fuzz_client::BayesianOptimizer;


fn optimize_hyperparameters<P: Protocol+PartialEq>(client: &mut Client<P>) -> FuzzConfig {
    // Create instance of Swarm
    let pso_swarm_size = 20;
    let pso_fuzzer_generations = 10;
    let pso_iterations = 100;
    let message_pool_size = 50;
    let inertial_weight = 1.0;
    let cognitive_weight = 1.0;
    let social_weight = 1.0;
    let regularization_strength = 0.25;
    let vmax = 0.25;

    let mut swarm = Swarm::new(
        pso_swarm_size,
        pso_iterations, 
        pso_fuzzer_generations, 
        message_pool_size,
        inertial_weight,
        cognitive_weight,
        social_weight,
        regularization_strength,
        vmax,
    );

    // Run swarm and get set configs to swarm's global best
    swarm.run_swarm(client);
    let mut pso_optimized_configs = swarm.global_best_position;

    // Create instance of BayesianOptimizer
    let bayesian_iterations = 50;
    let fuzzer_generations = 10;

    pso_optimized_configs.generations = fuzzer_generations;
    
    let mut bayesian_optimizer = BayesianOptimizer::new(
        &pso_optimized_configs,
        bayesian_iterations,
        pso_swarm_size,
        pso_iterations,
        pso_fuzzer_generations,
    );

    // Run Bayesian optimization and get set configs to optimizer's global best
    bayesian_optimizer.run_optimization(client);

    // Return the optimized configs
    bayesian_optimizer.get_optimized_hyperparameters()
}

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

    let mut optimized_configs = optimize_hyperparameters(&mut client);

    // Run fuzzing with configs from swarm
    let generations = 20;
    optimized_configs.generations = generations;

    print!("\nPRESS ENTER TO RUN FUZZER ... \n");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    client.fuzz(optimized_configs, true);
}
