#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


use std::str::from_utf8;

use fuzz_client::Client;
use fuzz_client::GreetingProtocol;
use fuzz_client::SMTP;
use fuzz_client::FuzzConfig;
use fuzz_client::MessageSequence;
use fuzz_client::TransportProtocol;
use fuzz_client::Protocol;


fn main() {
    // User-provided server address and transport protocol
    let server_address = String::from("10.0.0.92:8025");
    let transport_protocol: TransportProtocol = TransportProtocol::TCP;
    let target_protocol: SMTP = SMTP {};

    let pcap_file = String::from("/home/arbegla/projects/rust/binaries/netic/resources/new_smtp.pcap");
    let pcap_corpus = target_protocol.parse_pcap(pcap_file.as_str(), server_address.as_str());

    // Create instance of Client
    let mut client = Client::new(server_address, transport_protocol, target_protocol);
    client.corpus = pcap_corpus;

    let config = FuzzConfig {
        generations: 45,
        selection_pressure: 0.75,
        sequence_mutation_rate: 0.4,
        sequence_crossover_rate: 0.7,
        message_mutation_rate: 0.3,
        message_crossover_rate: 0.5,
        message_pool_size: 50,
        pool_update_rate: 0.3,
        state_rarity_threshold: 0.2,
        state_coverage_weight: 0.6,
        response_time_weight: 0.8,
        state_roc_weight: 0.8,
        state_rarity_weight: 1.0,
    };

    client.fuzz(config);
}
