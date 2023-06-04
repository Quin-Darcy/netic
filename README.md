# Netic: Genetic Network Protocol Fuzzer

This Rust program is a genetic fuzzer for testing protocol implementations. The fuzzer utilizes an evolutionary algorithm, Particle Swarm Optimization (PSO) for hyper-parameter optimization, and heuristics to generate protocol-specific message sequences that can potentially uncover bugs in the target protocol implementation.

### Overview

**Initialization**

In the beginning, the fuzzer initializes a `Client` with the user provided server address and transport protocol. The `Client` is also configured with the target protocol (e.g., SMTP), and an initial corpus of message sequences is created by parsing a PCAP file.

The program performs Particle Swarm Optimization (PSO) to optimize the fuzzing paramters. These optimized parameters are then used for running the fuzzing process.

```rust
fn main() {
    // Initialization
    let server_address = String::from("10.0.0.92:8025");
    let transport_protocol: TransportProtocol = TransportProtocol::TCP;
    let target_protocol: SMTP = SMTP {};

    let pcap_file = String::from("../resources/new_smtp.pcap");
    let pcap_corpus = target_protocol.parse_pcap(pcap_file.as_str(), server_address.as_str());

    let mut client = Client::new(server_address, transport_protocol, target_protocol);
    client.corpus = pcap_corpus;

    // PSO
    let mut swarm = Swarm::new(...);
    swarm.run_swarm(&mut client);

    let mut pso_optimized_configs = swarm.global_best_position;
    println!("Optimized configs found: {:?}", pso_optimized_configs);

    // Fuzzing
    client.fuzz(pso_optimized_configs, true);
}
```

**Execution**

The fuzzer iteratively executes the following steps for a specified number of generations:

1. Run message sequences from the corpus and record their interaction history.

2. Process the interaction history to update the state model and identify unique and rare server states.

3. Evaluate the fitness of each message sequence in the corpus based on coverage, rate of change, server response time, and presence of rare states.

4. Evolve the generation by applying selection, crossover, and mutation operations to create a new generation of message sequences.

After the specified number of generations have been created and tested, the final state model is converted into a digraph which then can be converted to PNG or SVG

**Extending Protocol Support**

This program is designed to be easily extended with support for new network protocols. This is accomplished by implementing the `Protocol` trait for the new protocol in a new Rust file in the `protocols` folder. The `Protocol` trait provides a common interface for defining shared behavior across multiple network protocol implementations.

For example, the support for SMTP and a made-up GreetingProtocol has been built out using the `Protocol` trait in the files `smtp.rs` and `greeting_protocol.rs` in the `protocols` folder. A new protocol can be added by starting with the template `protocols/new_protocol_template.rs` and filling out the relevant parts for the new protocol.

Here's an example of what a new protocol implementation might look like:

```rust
pub struct YourProtocol {
// Add any required fields and states for your protocol here.
}

impl Protocol for YourProtocol {
    type MessageType = YourProtocolMessageType;
    type MessageSectionsKey = YourProtocolMessageSectionsKey;
    type MessageSectionsValue = YourProtocolMessageSectionsValue;
    type ServerState = YourProtocolServerState;

    fn random_message(&self) -> Message<Self> {
    // Generate a random message for your protocol and return it.
        ...
    }
    ...
    // Similarly, implement other required methods for your protocol.
}
...
```

Each protocol implementation should provide the associated types and methods required by the `Protocol` trait. This includes types for messages, message sections, server states, as well as methods for creating, partins, mutating, and handling crossover of messages.

With the protocol-specific `Protocol` implementation, the `Client` and `Fuzzer` can be configured to use the new protocol, and the fuzzer can start creating and testing message sequences for that protocol.

### Notable Features

- **Genetic Algorithm:** The fuzzer uses a genetic algorithm to evolve message sequences, applying selection, crossover, and mutation operations. This helps explore diverse and potentially interesting test cases.

- **PSO Hyper-parameter Optimization**: The fuzzer uses Particle Swarm Optimization (PSO) for tuning key parameters of the genetic algorithm, such as selection pressure, mutation rate, message pool update rate, etc. This enables more effective exploration of the search space.

- **State Model:** The program builds and updates a state model of the server based on the server's responses to message sequences. This helps guide the fuzzer towards new and unexplored states.

- **Fitness Evaluation:** The fuzzer evaluates the fitness of message sequences based on various criteria, such as state coverage, state rarity, rate of change per sequence, and server response time. This allows the fuzzer to prioritize promising test cases.

- **Flexible Transport Layer Support**: The user can specify the transport layer protocol (TCP or UDP) for their target protocol. The `Transport` struct in the code provides the appropriate send/receive methods based on this specification.

- **Configuration:** The fuzzer provides a configurable framework with parameters for controlling the fuzzing process, such as selection pressure, mutation rate, crossover rate, message pool size, and state rarity threshold.

### Code Design

The key design aspect of this project is the use of the `Protocol` trait, which serves as a common interface for defining shared behavior across multiple network protocol implementations. By implementing the `Protocol` trait, you can easily extend the program to support new protocols.

The `Protocol` trait requires associated types for `MessageType`, `MessageSectionsKey`, `MessageSectionsValue`, and `ServerState`. Additionally, it specifies methods for creating, mutating, and parsing messages as well as handling crossover operations.

By using the `Self` type alias, the `Protocol` trait ensures that the `Message` struct and the implementing type share the same protocol. This prevents mixing different protocols and enforces a consistent implementation. The use of `Self` in the method signatures also allows for generic code that works with any type implementing the `Protocol` trait.

In summary, the flexible and extensible design is achieved through the use of Rust's trait system, making it easy to add support for new protocols by simply implementing the `Protocol` trait with the desired behavior and associated types. The majority of the code and its structure remains independent of any specific protocol, highlighting the reusability and adaptability of the design.

### Future Directions

This project is continually evolving. We are working on adding support for more protocols and enhancing the fuzzing strategies for more effective uncovering of bugs in protocol implementations. Currently the hyper-parameter optimization is being re-worked into a two-stage approach where PSO is run in the first stage and a Bayesian inference algorithm will take the paramter set converged on by PSO to fine tune it even further.