# Netic: Genetic Network Protocol Fuzzer

This Rust program is a genetic fuzzer for testing protocol implementations. The fuzzer utilizes an evolutionary algorithm, Particle Swarm Optimization (PSO), and Bayesian optimization for two-stage hyper-parameter tuning, and heuristics to generate protocol-specific message sequences that can potentially uncover bugs in the target protocol implementation.

### Overview

**Initialization**

In the beginning, the fuzzer initializes a `Client` with the user provided server address and transport protocol. The `Client` is also configured with the target protocol (e.g., SMTP), and an initial corpus of message sequences is created by parsing a PCAP file.

The program performs two stages of hyperparameter optimization. Initially, it uses Particle Swarm Optimization (PSO) to optimize the fuzzing parameters. The second stage utilizes Bayesian Optimization to fine-tune the parameters further, enabling the fuzzing process to be more precise and effective.

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

    // Two-stage Hyperparameter Optimization
    let mut optimized_configs = optimize_hyperparameters(&mut client);

    // Fuzzing
    client.fuzz(optimized_configs, true);
}
```

**Execution**

The fuzzer iteratively executes the following steps for a specified number of generations:

1. Run message sequences from the corpus and record their interaction history.

2. Process the interaction history to update the state model and identify unique and rare server states.

3. Evaluate the fitness of each message sequence in the corpus based on coverage, rate of change, server response time, and presence of rare states.

4. Evolve the generation by applying selection, crossover, and mutation operations to create a new generation of message sequences.

![Program Diagram](resources/program_diagram.png)

After the specified number of generations have been created and tested, the final state model is converted into a digraph which then can be converted to PNG or SVG

![Program Diagram](resources/full_diagram.png)

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

- **Two-stage Hyper-parameter Optimization**: The fuzzer uses Particle Swarm Optimization (PSO) for the initial tuning of key parameters of the genetic algorithm, such as selection pressure, mutation rate, message pool update rate, etc. Then, it uses Bayesian Optimization to fine-tune these parameters further. This two-stage approach enables a more effective exploration of the search space.

- **State Model:** The program builds and updates a state model of the server based on the server's responses to message sequences. This helps guide the fuzzer towards new and unexplored states.

- **Fitness Evaluation:** The fuzzer evaluates the fitness of message sequences based on various criteria, such as state coverage, state rarity, rate of change per sequence, and server response time. This allows the fuzzer to prioritize promising test cases.

- **Flexible Transport Layer Support**: The user can specify the transport layer protocol (TCP or UDP) for their target protocol. The `Transport` struct in the code provides the appropriate send/receive methods based on this specification.

- **Configuration:** The fuzzer provides a configurable framework with parameters for controlling the fuzzing process, such as selection pressure, mutation rate, crossover rate, message pool size, and state rarity threshold.

### Code Design

The key design aspect of this project is the use of the `Protocol` trait, which serves as a common interface for defining shared behavior across multiple network protocol implementations. By implementing the `Protocol` trait, you can easily extend the program to support new protocols.

The `Protocol` trait requires associated types for `MessageType`, `MessageSectionsKey`, `MessageSectionsValue`, and `ServerState`. Additionally, it specifies methods for creating, mutating, and parsing messages as well as handling crossover operations.

By using the `Self` type alias, the `Protocol` trait ensures that the `Message` struct and the implementing type share the same protocol. This prevents mixing different protocols and enforces a consistent implementation. The use of `Self` in the method signatures also allows for generic code that works with any type implementing the `Protocol` trait.

In summary, the flexible and extensible design is achieved through the use of Rust's trait system, making it easy to add support for new protocols by simply implementing the `Protocol` trait with the desired behavior and associated types. The majority of the code and its structure remains independent of any specific protocol, highlighting the reusability and adaptability of the design.

### Hyperparameter Optimization

The program's hyperparameter optimization process is conducted in two stages. The initial phase employs Particle Swarm Optimization (PSO) to optimize the fuzzing parameters, while the secondary phase uses Bayesian Optimization to further refine these parameters, thereby increasing the precision and effectiveness of the fuzzing process.

The fitness computation for each set of hyperparameters proceeds differently for each stage. 

During the PSO stage, a designated number of fuzzer generations is defined, alongside other parameters. As the PSO iterates and updates each particle within the swarm, the current particle - an instance of the `FuzzConfig` struct - is evaluated. This is done by passing the `FuzzConfig` to the client, which then allows it to run for a specified number of generations (`pso_fuzzer_generations`). The average fitness of the population of message sequences is calculated thereafter. The slope of the best fit line, which passes through the data points (x-axis representing the generation number, y-axis representing the average fitness for that generation) is returned from the `Client::evaluate()` method. Following this, the PSO algorithm performs L2 regularization on the slope of the best fit line, with the output being the fitness of the particle. Here's the fitness calculation snippet from the PSO code:

```rust
// pso.rs
    fn evaluate_fitness<P: Protocol+PartialEq>(&mut self, client: &mut Client<P>, regularization_strength: f32) -> f32 {
        client.fuzz(self.position.clone(), false);

        let slope_of_best_fit_line = client.evaluate();

        // Add up the squares of each hyperparameter in the FuzzConfig
        let l2_norm = self.position.selection_pressure.powi(2) 
                     + self.position.sequence_mutation_rate.powi(2)
                     + self.position.sequence_crossover_rate.powi(2)
                     + self.position.message_mutation_rate.powi(2)
                     + self.position.message_crossover_rate.powi(2)
                     + self.position.pool_update_rate.powi(2)
                     + self.position.state_rarity_threshold.powi(2)
                     + self.position.state_coverage_weight.powi(2)
                     + self.position.response_time_weight.powi(2)
                     + self.position.state_roc_weight.powi(2)
                     + self.position.state_rarity_weight.powi(2);
        let regularization_term = regularization_strength * l2_norm;
        
        // Fitness is the slope of the best fit line minus the regularization term
        let fitness = slope_of_best_fit_line - regularization_term;
        
        fitness
    }
```

During the Bayesian Optimization stage, the fitness computation still uses the same fitness definition, but L2 regularization is not performed. The Bayesian optimization process performs one iteration at a time, calculating new hyperparameters and predicting fitness based on exponential smoothing in `BayesianOptimizer::predict_fitness`, where the prediction is a weighted average of past fitness observations, with the weights decaying exponentially as the observations get older. It then updates the variances based on the observed and predicted fitness values where a larger difference between the observation and prediction yeilds larger variances. The variances are then used in subsequent iterations to define the distributions from which the new hyperparameters are sampled from. Here's the relevant code snippet from the Bayesian Optimization process:

```rust
// bayesian.rs
    // Performs one iteration of the Bayesian optimization process
    fn iterate<P: Protocol+PartialEq>(&mut self, client: &mut Client<P>) {
        // Calculate new hyperparameters
        self.calculate_new_hyperparameters();

        // Predict the fitness based on the updated hyperparameters
        let predicted_fitness = self.predict_fitness();

        // Create FuzzConfig instance with new hyperparameters
        let new_configs = FuzzConfig {
            generations: self.hyperparameters[0] as usize,
            selection_pressure: self.hyperparameters[1],
            sequence_mutation_rate: self.hyperparameters[2],
            sequence_crossover_rate: self.hyperparameters[3],
            message_mutation_rate: self.hyperparameters[4],
            message_crossover_rate: self.hyperparameters[5],
            message_pool_size: self.hyperparameters[6] as usize,
            pool_update_rate: self.hyperparameters[7],
            state_rarity_threshold: self.hyperparameters[8],
            state_coverage_weight: self.hyperparameters[9],
            response_time_weight: self.hyperparameters[10],
            state_roc_weight: self.hyperparameters[11],
            state_rarity_weight: self.hyperparameters[12],
        };

        // Run the fuzzer with the new configs and get the fitness score
        client.fuzz(new_configs, false);
        let observed_fitness = client.evaluate();

        // Update the variances based on the fitness score
        self.update_variances(predicted_fitness, observed_fitness);

        // Update the observed fitnesses
        self.observed_fitnesses.push(observed_fitness);
    }
```

The Bayesian Optimization process continues to iterate in this manner for a specified number of iterations.

### Future Directions

This project is continually evolving. We are working on adding support for more protocols and enhancing the fuzzing strategies for more effective uncovering of bugs in protocol implementations. Currently the hyper-parameter optimization is being re-worked into a two-stage approach where PSO is run in the first stage and a Bayesian inference algorithm will take the paramter set converged on by PSO to fine tune it even further.