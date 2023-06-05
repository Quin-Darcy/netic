use rand::Rng;
use rand;

use rand_distr::{Normal, Distribution};

use crate::FuzzConfig;
use crate::Client;
use crate::Protocol;


pub struct BayesianOptimizer {
    hyperparameters: Vec<f32>,
    variances: Vec<f32>,
    iterations: usize,
    observed_fitnesses: Vec<f32>,
}

impl BayesianOptimizer {
    // Constructs a new BayesianOptimizer with initial values
    pub fn new(
        pso_optimized_configs: &FuzzConfig, 
        iterations: usize,
        pso_swarm_size: usize, 
        pso_iterations: usize, 
        pso_fuzzer_generations: usize,
    ) -> Self {
        // Initialize hyperparameters with values from PSO optimized configs
        let hyperparameters = vec![
            pso_optimized_configs.generations as f32,
            pso_optimized_configs.selection_pressure,
            pso_optimized_configs.sequence_mutation_rate,
            pso_optimized_configs.sequence_crossover_rate,
            pso_optimized_configs.message_mutation_rate,
            pso_optimized_configs.message_crossover_rate,
            pso_optimized_configs.message_pool_size as f32,
            pso_optimized_configs.pool_update_rate,
            pso_optimized_configs.state_rarity_threshold,
            pso_optimized_configs.state_coverage_weight,
            pso_optimized_configs.response_time_weight,
            pso_optimized_configs.state_roc_weight,
            pso_optimized_configs.state_rarity_weight,
        ];

        // Calculate initial variance
        let initial_variance = calculate_initial_variance(pso_swarm_size, pso_iterations, pso_fuzzer_generations);
        let variances = vec![initial_variance; hyperparameters.len()];

        Self {
            hyperparameters,
            variances,
            iterations,
            observed_fitnesses: Vec::new(),
        }
    }
    
    // Runs the Bayesian optimization for a specified number of iterations
    pub fn run_optimization<P: Protocol+PartialEq>(&mut self, client: &mut Client<P>) {
        println!("\n");
        for i in 0..self.iterations {
            println!("Bayesian Optimization Iteration: {}", i);
            if i == 0 {
                print_info(&self.hyperparameters, &self.variances, &0.0);
            } else {
                print_info(&self.hyperparameters, &self.variances, &self.observed_fitnesses[i-1]);
            }

            self.iterate(client);
        }
    }
    
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
    
    // Calculates the new hyperparameters based on the current ones and their variances
    fn calculate_new_hyperparameters(&mut self) {
        for i in 0..self.hyperparameters.len() {
            // Skip hyperparameters that are not being optimized (generations and message pool size)
            if i == 0 || i == 6 {
                continue;
            }

            // Create a normal distribution with the current hyperparameter as the mean and its variance as the standard deviation
            let normal = Normal::new(self.hyperparameters[i] as f64, self.variances[i] as f64).unwrap();
            let sample = normal.sample(&mut rand::thread_rng()) as f32;

            // Clip the sample to [0, 1] to ensure it stays within the bounds of the hyperparameter
            let clipped_sample = sample.max(0.0).min(1.0);

            self.hyperparameters[i] = clipped_sample;
        }
    }

    // Update the variances based on the fitness scores of the hyperparameters
    fn update_variances(&mut self, predicted_fitness: f32, observed_fitness: f32) {
        // Set the min and max slope to normalize the difference term
        let min_slope = -5.0;
        let max_slope = 5.0;

        for i in 0..self.variances.len() {
            // Skip hyperparameters that are not being optimized (generations and message pool size)
            if i == 0 || i == 6 {
                continue;
            }
    
            // Compute the absolute difference between the observed fitness and the predicted fitness and normalize it
            let mut difference = (observed_fitness - predicted_fitness).abs() / (max_slope - min_slope);

            // Add slight noise to the difference to encourage exploration
            let noise = rand::thread_rng().gen_range(-0.025..0.025);
            difference = difference + noise;

            // Clip the difference to [0, 1] to ensure it stays within the bounds of the variance
            let clipped_difference = difference.max(0.01).min(1.0);
    
            // Update the variance
            // Note: This is a simple approach. You may want to normalize the difference or apply a more sophisticated method.
            self.variances[i] = clipped_difference;
        }
    }

    // This method predicts the fitness value by using exponential smoothing, where the prediction is a weighted average
    // of past observations, with the weights decaying exponentially as the observations get older.
    fn predict_fitness(&self) -> f32 {
        // Set the smoothing factor
        let alpha = 0.65;

        let n = self.observed_fitnesses.len();
        if n == 0 {
            return 0.00;
        }
    
        let mut smoothed_fitness = self.observed_fitnesses[0];
        for i in 1..n {
            smoothed_fitness = alpha * self.observed_fitnesses[i] + (1.0 - alpha) * smoothed_fitness;
        }
    
        smoothed_fitness
    }

    // This function returns the hyperparameters that were found to be the best after the optimization process
    pub fn get_optimized_hyperparameters(&self) -> FuzzConfig {
        FuzzConfig {
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
        }
    }
}

// This function calculates the initial variance based off of the number of particles, iterations, and generations
// The bigger any one of these parameters are, the smaller the initial variance will be since if the swarm is bigger
// (number of particles) or the number of pso iterations is bigger, then the swarm will have more time or opportunities
// to converge to a more optimal solution. If the number of fuzzer generations is bigger, then the fitness scores will
// be based off of more data points and thus be more reliable. This means that the variance should be smaller since
// the fitness scores are more reliable.
fn calculate_initial_variance(pso_swarm_size: usize, pso_iterations: usize, pso_fuzzer_generations: usize) -> f32 {
    let inverse_sum = 1.0 / pso_swarm_size as f32 + 1.0 / pso_iterations as f32 + 1.0 / pso_fuzzer_generations as f32;
    inverse_sum / 3.0
} 

fn print_info(position: &Vec<f32>, variances: &Vec<f32>, fitness: &f32) {
    println!("        Position:  ({:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2})", 
        position[0] as usize,
        position[1],
        position[2],
        position[3],
        position[4],
        position[5],
        position[6] as usize,
        position[7],
        position[8],
        position[9],
        position[10],
        position[11],
        position[12],
    );

    println!("        Variances: ({:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2})", 
        variances[0],
        variances[1],
        variances[2],
        variances[3],
        variances[4],
        variances[5],
        variances[6],
        variances[7],
        variances[8],
        variances[9],
        variances[10],
        variances[11],
        variances[12],
    );

    println!("        Fitness:    {:.4}\n", fitness);
}
