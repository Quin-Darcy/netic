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
    let pso_swarm_size = 10;
    let pso_fuzzer_generations = 10;
    let pso_iterations = 10;
    let message_pool_size = 50;
    let inertial_weight = 1.0;
    let cognitive_weight = 1.0;
    let social_weight = 1.0;
    let regularization_strength = 0.1;
    let vmax = 0.1;

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
    let bayesian_iterations = 20;
    let fuzzer_generations = 20;

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

    // Optimized hyperparameters on smaller PCAP file to save time
    let pcap_file = String::from("../resources/smtp.pcap");
    let pcap_corpus = target_protocol.parse_pcap(pcap_file.as_str(), server_address.as_str().clone());

    // Create instance of Client
    let mut client = Client::new(server_address.clone(), transport_protocol, target_protocol);
    client.corpus = pcap_corpus;

    let mut optimized_configs = optimize_hyperparameters(&mut client);

    // Run fuzzing with configs from swarm
    let generations = 30;
    optimized_configs.generations = generations;

    // Reset client's corpus to "official" corpus by parsing bigger PCAP
    let pcap_file = String::from("../resources/new_smtp.pcap");
    let pcap_corpus = target_protocol.clone().parse_pcap(pcap_file.as_str(), server_address.as_str().clone());

    client.corpus = pcap_corpus;

    println!("Optimized Hyperparameters: ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n", 
        optimized_configs.generations,
        optimized_configs.selection_pressure,
        optimized_configs.sequence_mutation_rate,
        optimized_configs.sequence_crossover_rate,
        optimized_configs.message_mutation_rate,
        optimized_configs.message_crossover_rate,
        optimized_configs.message_pool_size,
        optimized_configs.pool_update_rate,
        optimized_configs.state_rarity_threshold,
        optimized_configs.state_coverage_weight,
        optimized_configs.response_time_weight,
        optimized_configs.state_roc_weight,
        optimized_configs.state_rarity_weight
    );

    print!("\nPRESS ENTER TO RUN FUZZER ... \n");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // (20, 0.4208, 0.4129, 0.2749, 0.6024, 0.3268, 50, 0.4425, 0.4724, 0.3726, 0.7373, 0.2529, 0.5072)

    client.fuzz(optimized_configs, true);
}
