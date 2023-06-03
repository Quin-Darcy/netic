use rand::Rng;
use rand;

use crate::FuzzConfig;
use crate::Client;
use crate::Protocol;


#[derive(Clone, Debug)]
struct Particle {
    position: FuzzConfig,
    velocity: FuzzConfig,
    personal_best_position: FuzzConfig,
    personal_best_fitness: f64,
}

#[derive(Debug)]
pub struct Swarm {
    particles: Vec<Particle>,
    global_best_position: FuzzConfig,
    global_best_fitness: f64,
}

impl Particle {
    fn random(generations: usize, message_pool_size: usize) -> Particle {
        let velocity = FuzzConfig {
            generations: 0,
            selection_pressure: 0.0,
            sequence_mutation_rate: 0.0,
            sequence_crossover_rate: 0.0,
            message_mutation_rate: 0.0,
            message_crossover_rate: 0.0,
            message_pool_size: 0,
            pool_update_rate: 0.0,
            state_rarity_threshold: 0.0,
            state_coverage_weight: 0.0,
            response_time_weight: 0.0,
            state_roc_weight: 0.0,
            state_rarity_weight: 0.0,
        };

        let mut rng = rand::thread_rng();

        let position = FuzzConfig {
            generations: generations,
            selection_pressure: rng.gen_range(0.0..1.0),
            sequence_mutation_rate: rng.gen_range(0.0..1.0),
            sequence_crossover_rate: rng.gen_range(0.0..1.0),
            message_mutation_rate: rng.gen_range(0.0..1.0),
            message_crossover_rate: rng.gen_range(0.0..1.0),
            message_pool_size: message_pool_size,
            pool_update_rate: rng.gen_range(0.0..1.0),
            state_rarity_threshold: rng.gen_range(0.0..0.5),
            state_coverage_weight: rng.gen_range(0.0..1.0),
            response_time_weight: rng.gen_range(0.0..1.0),
            state_roc_weight: rng.gen_range(0.0..1.0),
            state_rarity_weight: rng.gen_range(0.0..1.0),
        };

        Particle {
            position: position.clone(),
            velocity: velocity,
            personal_best_position: position.clone(),
            personal_best_fitness: 0.0,
        }
    }

    fn evaluate_fitness<P: Protocol+PartialEq>(&mut self, client: &mut Client<P>) {
        client.fuzz(self.position.clone());

        let slope_of_best_fit_line = client.evaluate();

        let regularization_strength = 0.1; // This is a hyperparameter that you'd have to choose
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
        
        let fitness = slope_of_best_fit_line - regularization_term;
        
    }
}

impl Swarm {
    pub fn new(num_particles: usize, generations: usize, message_pool_size: usize) -> Swarm {
        let mut particles = Vec::new();
        for _ in 0..num_particles {
            particles.push(Particle::random(generations, message_pool_size));
        }

        Swarm {
            particles: particles,
            global_best_position: FuzzConfig {
                generations: generations,
                selection_pressure: 0.0,
                sequence_mutation_rate: 0.0,
                sequence_crossover_rate: 0.0,
                message_mutation_rate: 0.0,
                message_crossover_rate: 0.0,
                message_pool_size: message_pool_size,
                pool_update_rate: 0.0,
                state_rarity_threshold: 0.0,
                state_coverage_weight: 0.0,
                response_time_weight: 0.0,
                state_roc_weight: 0.0,
                state_rarity_weight: 0.0,
            },
            global_best_fitness: 0.0,
        }
    }
}