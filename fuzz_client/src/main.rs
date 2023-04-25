#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use fuzz_client::Client;
use fuzz_client::GreetingProtocol;
use fuzz_client::FuzzConfig;
use fuzz_client::MessageSequence;


fn main() {
    let server_address: String = String::from("127.0.0.1:8888");

    // Create instance of Client
    let mut client = Client::new(server_address, GreetingProtocol);

    let config = FuzzConfig {
        generations: 10,
        selection_pressure: 0.8,
        sequence_mutation_rate: 0.4,
        sequence_crossover_rate: 0.7,
        message_mutation_rate: 0.3,
        message_crossover_rate: 0.4,
        message_pool_size: 50,
        pool_update_rate: 0.2,
        state_rarity_threshold: 0.2,
        state_coverage_weight: 0.6,
        state_roc_weight: 0.5,
        state_rarity_weight: 0.7,
    };

    client.fuzz(config);
}
