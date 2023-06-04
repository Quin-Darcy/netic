use std::io::{self, Write};

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
    personal_best_fitness: f32,
}

#[derive(Debug)]
pub struct Swarm {
    particles: Vec<Particle>,
    pub global_best_position: FuzzConfig,
    global_best_fitness: f32,
    num_particles: usize,
    pso_iterations: usize, 
    inertial_weight: f32,
    cognitive_weight: f32,
    social_weight: f32,
    regularization_strength: f32,
}

impl Particle {
    fn random(generations: usize, message_pool_size: usize) -> Particle {
        let mut rng = rand::thread_rng();

        let velocity = FuzzConfig {
            generations: 0,
            selection_pressure: rng.gen_range(-0.1..0.1),
            sequence_mutation_rate: rng.gen_range(-0.1..0.1),
            sequence_crossover_rate: rng.gen_range(-0.1..0.1),
            message_mutation_rate: rng.gen_range(-0.1..0.1),
            message_crossover_rate: rng.gen_range(-0.1..0.1),
            message_pool_size: 0,
            pool_update_rate: rng.gen_range(-0.1..0.1),
            state_rarity_threshold: rng.gen_range(-0.1..0.1),
            state_coverage_weight: rng.gen_range(-0.1..0.1),
            response_time_weight: rng.gen_range(-0.1..0.1),
            state_roc_weight: rng.gen_range(-0.1..0.1),
            state_rarity_weight: rng.gen_range(-0.1..0.1),
        };

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

    fn evaluate_fitness<P: Protocol+PartialEq>(&mut self, client: &mut Client<P>, regularization_strength: f32) -> f32 {
        client.fuzz(self.position.clone(), false);

        let slope_of_best_fit_line = client.evaluate();

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
        
        fitness
    }

    fn update_particle<P: Protocol+PartialEq>(
        &mut self, 
        client: &mut Client<P>,
        inertial_weight: f32,
        cognitive_weight: f32,
        social_weight: f32,
        global_best_position: &FuzzConfig,
        regularization_strength: f32,
    ) {
        let fitness = self.evaluate_fitness(client, regularization_strength);

        if fitness > self.personal_best_fitness {
            self.personal_best_fitness = fitness;
            self.personal_best_position = self.position.clone();
        }

        // Update velocity and position
        let mut rng = rand::thread_rng();
        
        self.velocity.selection_pressure = inertial_weight * self.velocity.selection_pressure
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.selection_pressure - self.position.selection_pressure)
        + social_weight * rng.gen::<f32>() * (global_best_position.selection_pressure - self.position.selection_pressure);

        self.velocity.sequence_mutation_rate = inertial_weight * self.velocity.sequence_mutation_rate
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.sequence_mutation_rate - self.position.sequence_mutation_rate)
        + social_weight * rng.gen::<f32>() * (global_best_position.sequence_mutation_rate - self.position.sequence_mutation_rate);

        self.velocity.sequence_crossover_rate = inertial_weight * self.velocity.sequence_crossover_rate
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.sequence_crossover_rate - self.position.sequence_crossover_rate)
        + social_weight * rng.gen::<f32>() * (global_best_position.sequence_crossover_rate - self.position.sequence_crossover_rate);

        self.velocity.message_mutation_rate = inertial_weight * self.velocity.message_mutation_rate
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.message_mutation_rate - self.position.message_mutation_rate)
        + social_weight * rng.gen::<f32>() * (global_best_position.message_mutation_rate - self.position.message_mutation_rate);

        self.velocity.message_crossover_rate = inertial_weight * self.velocity.message_crossover_rate
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.message_crossover_rate - self.position.message_crossover_rate)
        + social_weight * rng.gen::<f32>() * (global_best_position.message_crossover_rate - self.position.message_crossover_rate);

        self.velocity.pool_update_rate = inertial_weight * self.velocity.pool_update_rate
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.pool_update_rate - self.position.pool_update_rate)
        + social_weight * rng.gen::<f32>() * (global_best_position.pool_update_rate - self.position.pool_update_rate);

        self.velocity.state_rarity_threshold = inertial_weight * self.velocity.state_rarity_threshold
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.state_rarity_threshold - self.position.state_rarity_threshold)
        + social_weight * rng.gen::<f32>() * (global_best_position.state_rarity_threshold - self.position.state_rarity_threshold);

        self.velocity.state_coverage_weight = inertial_weight * self.velocity.state_coverage_weight
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.state_coverage_weight - self.position.state_coverage_weight)
        + social_weight * rng.gen::<f32>() * (global_best_position.state_coverage_weight - self.position.state_coverage_weight);

        self.velocity.response_time_weight = inertial_weight * self.velocity.response_time_weight
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.response_time_weight - self.position.response_time_weight)
        + social_weight * rng.gen::<f32>() * (global_best_position.response_time_weight - self.position.response_time_weight);

        self.velocity.state_roc_weight = inertial_weight * self.velocity.state_roc_weight
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.state_roc_weight - self.position.state_roc_weight)
        + social_weight * rng.gen::<f32>() * (global_best_position.state_roc_weight - self.position.state_roc_weight);

        self.velocity.state_rarity_weight = inertial_weight * self.velocity.state_rarity_weight
        + cognitive_weight * rng.gen::<f32>() * (self.personal_best_position.state_rarity_weight - self.position.state_rarity_weight)
        + social_weight * rng.gen::<f32>() * (global_best_position.state_rarity_weight - self.position.state_rarity_weight);

        // If the velocity is zero, give it a random value
        if self.velocity.selection_pressure.abs() < 1e-9 {
            self.velocity.selection_pressure = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.sequence_mutation_rate.abs() < 1e-9 {
            self.velocity.sequence_mutation_rate = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.sequence_crossover_rate.abs() < 1e-9 {
            self.velocity.sequence_crossover_rate = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.message_mutation_rate.abs() < 1e-9 {
            self.velocity.message_mutation_rate = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.message_crossover_rate.abs() < 1e-9 {
            self.velocity.message_crossover_rate = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.pool_update_rate.abs() < 1e-9 {
            self.velocity.pool_update_rate = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.state_rarity_threshold.abs() < 1e-9 {
            self.velocity.state_rarity_threshold = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.state_coverage_weight.abs() < 1e-9 {
            self.velocity.state_coverage_weight = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.response_time_weight.abs() < 1e-9 {
            self.velocity.response_time_weight = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.state_roc_weight.abs() < 1e-9 {
            self.velocity.state_roc_weight = rng.gen::<f32>() * 2.0 - 1.0;
        }
        if self.velocity.state_rarity_weight.abs() < 1e-9 {
            self.velocity.state_rarity_weight = rng.gen::<f32>() * 2.0 - 1.0;
        }

        // Check velocity bounds - abs max velocity is 10% of the range of the parameter
        self.velocity.selection_pressure = self.velocity.selection_pressure.max(-0.1).min(0.1);
        self.velocity.sequence_mutation_rate = self.velocity.sequence_mutation_rate.max(-0.1).min(0.1);
        self.velocity.sequence_crossover_rate = self.velocity.sequence_crossover_rate.max(-0.1).min(0.1);
        self.velocity.message_mutation_rate = self.velocity.message_mutation_rate.max(-0.1).min(0.1);
        self.velocity.message_crossover_rate = self.velocity.message_crossover_rate.max(-0.1).min(0.1);
        self.velocity.pool_update_rate = self.velocity.pool_update_rate.max(-0.1).min(0.1);
        self.velocity.state_rarity_threshold = self.velocity.state_rarity_threshold.max(-0.1).min(0.1);
        self.velocity.state_coverage_weight = self.velocity.state_coverage_weight.max(-0.1).min(0.1);
        self.velocity.response_time_weight = self.velocity.response_time_weight.max(-0.1).min(0.1);
        self.velocity.state_roc_weight = self.velocity.state_roc_weight.max(-0.1).min(0.1);
        self.velocity.state_rarity_weight = self.velocity.state_rarity_weight.max(-0.1).min(0.1);

        // Update the position of the particle
        self.position.selection_pressure += self.velocity.selection_pressure;
        self.position.sequence_mutation_rate += self.velocity.sequence_mutation_rate;
        self.position.sequence_crossover_rate += self.velocity.sequence_crossover_rate;
        self.position.message_mutation_rate += self.velocity.message_mutation_rate;
        self.position.message_crossover_rate += self.velocity.message_crossover_rate;
        self.position.pool_update_rate += self.velocity.pool_update_rate;
        self.position.state_rarity_threshold += self.velocity.state_rarity_threshold;
        self.position.state_coverage_weight += self.velocity.state_coverage_weight;
        self.position.response_time_weight += self.velocity.response_time_weight;
        self.position.state_roc_weight += self.velocity.state_roc_weight;
        self.position.state_rarity_weight += self.velocity.state_rarity_weight;

        // Check position bounds
        self.position.selection_pressure = self.position.selection_pressure.max(0.0).min(1.0);
        self.position.sequence_mutation_rate = self.position.sequence_mutation_rate.max(0.0).min(1.0);
        self.position.sequence_crossover_rate = self.position.sequence_crossover_rate.max(0.0).min(1.0);
        self.position.message_mutation_rate = self.position.message_mutation_rate.max(0.0).min(1.0);
        self.position.message_crossover_rate = self.position.message_crossover_rate.max(0.0).min(1.0);
        self.position.pool_update_rate = self.position.pool_update_rate.max(0.0).min(1.0);
        self.position.state_rarity_threshold = self.position.state_rarity_threshold.max(0.0).min(1.0);
        self.position.state_coverage_weight = self.position.state_coverage_weight.max(0.0).min(1.0);
        self.position.response_time_weight = self.position.response_time_weight.max(0.0).min(1.0);
        self.position.state_roc_weight = self.position.state_roc_weight.max(0.0).min(1.0);
        self.position.state_rarity_weight = self.position.state_rarity_weight.max(0.0).min(1.0);
    }
}

impl Swarm {
    pub fn new(
        num_particles: usize,
        pso_iterations: usize, 
        generations: usize, 
        message_pool_size: usize,
        inertial_weight: f32,
        cognitive_weight: f32,
        social_weight: f32,
        regularization_strength: f32,
    ) -> Swarm {
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
            num_particles: num_particles,
            pso_iterations: pso_iterations,
            inertial_weight: inertial_weight,
            cognitive_weight: cognitive_weight,
            social_weight: social_weight,
            regularization_strength: regularization_strength,
        }
    }

    pub fn run_swarm<P: Protocol+PartialEq>(&mut self, client: &mut Client<P>) {
        let mut count: usize = 0;
        let total = self.pso_iterations * self.num_particles;

        let initial_inertia = self.inertial_weight;
        let final_inertia = self.inertial_weight * 0.2;

        println!("\n");

        for i in 0..self.pso_iterations {
            // Perform contraction on the inertial weight so that global exploration is favored at the beginning of the search
            self.inertial_weight = initial_inertia - (initial_inertia - final_inertia) * (i as f32 / self.pso_iterations as f32);
            self.inertial_weight = self.inertial_weight.max(final_inertia);

            for particle in &mut self.particles {
                print!("\rRunning Particle Swarm Optimization on fuzz_client hyper-parameters ... {:.2}%", (count as f64 / total as f64) * 100.0);
                io::stdout().flush().unwrap();
                
                // Update the particle's position and velocity
                particle.update_particle(
                    client, 
                    self.inertial_weight, 
                    self.cognitive_weight, 
                    self.social_weight, 
                    &self.global_best_position, 
                    self.regularization_strength
                );
    
                // Update the global best position if necessary
                if particle.personal_best_fitness > self.global_best_fitness {
                    self.global_best_fitness = particle.personal_best_fitness;
                    self.global_best_position = particle.personal_best_position.clone();
                }

                count += 1;
            }
        }
        println!("\nPSO Complete!");
        print_position(&self.global_best_position);
    }
}

fn print_particle(particle: &Particle) {
    println!("        Position: ({:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2})", 
        particle.position.generations,
        particle.position.selection_pressure,
        particle.position.sequence_mutation_rate,
        particle.position.sequence_crossover_rate,
        particle.position.message_mutation_rate,
        particle.position.message_crossover_rate,
        particle.position.message_pool_size,
        particle.position.pool_update_rate,
        particle.position.state_rarity_threshold,
        particle.position.state_coverage_weight,
        particle.position.response_time_weight,
        particle.position.state_roc_weight,
        particle.position.state_rarity_weight,
    );

    println!("        Velocity: ({:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2})\n",
        particle.velocity.generations,
        particle.velocity.selection_pressure,
        particle.velocity.sequence_mutation_rate,
        particle.velocity.sequence_crossover_rate,
        particle.velocity.message_mutation_rate,
        particle.velocity.message_crossover_rate,
        particle.velocity.message_pool_size,
        particle.velocity.pool_update_rate,
        particle.velocity.state_rarity_threshold,
        particle.velocity.state_coverage_weight,
        particle.velocity.response_time_weight,
        particle.velocity.state_roc_weight,
        particle.velocity.state_rarity_weight,
    ); 
}

fn print_position(position: &FuzzConfig) {
    println!("    Position: ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n", 
        position.generations,
        position.selection_pressure,
        position.sequence_mutation_rate,
        position.sequence_crossover_rate,
        position.message_mutation_rate,
        position.message_crossover_rate,
        position.message_pool_size,
        position.pool_update_rate,
        position.state_rarity_threshold,
        position.state_coverage_weight,
        position.response_time_weight,
        position.state_roc_weight,
        position.state_rarity_weight
    );
}

