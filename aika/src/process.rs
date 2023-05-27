use std::cell::RefCell;

use crate::environment::{Environment, Event};

/// Trait for a process. A process is a function that is executed at a given time in the simulation.
pub trait Process {
    fn run(&mut self, events: &mut Environment);
}

/// Enumerator for the type of determinism of a process's execution.
#[derive(Clone)]
pub enum ProcessType {
    /// Executes the process deterministically in simulation time
    Deterministic,
    /// Executes the process stochastically in simulation time
    Stochastic,
}

/// Enumerator for the duration of a process's execution.
#[derive(Clone)]
pub enum ProcessDuration {
    /// Executes the process for a finite amount of time
    Finite,
    /// Executes the process for an infinite amount of time
    Infinite,
}

/// Struct for deterministic infinite process. This process will run forever until the simulation truncates. It is the default process type.
pub struct StandardProcess {
    /// The ID of the process
    pub process_id: u16,
    /// The name of the process
    pub process_name: String,
    /// The type of the process's execution
    pub process_type: ProcessType,
    /// The duration type of the process's execution
    pub process_duration: ProcessDuration,
    /// The function that the environment will execute at the specified event time
    pub process: fn(),
}

impl StandardProcess {
    pub fn new(process_id: u16, process_name: String, process: fn()) -> Self {
        StandardProcess {
            process_id: process_id,
            process_name: process_name,
            process_type: ProcessType::Deterministic,
            process_duration: ProcessDuration::Infinite,
            process: process,
        }
    }

    pub fn add_new_event(&mut self, env: &mut Environment) {
        let process = Box::new(Self {
            process_id: self.process_id,
            process_name: self.process_name.clone(),
            process_type: self.process_type.clone(),
            process_duration: self.process_duration.clone(),
            process: self.process,
        });
        let new_event = Event {
            time: env.curr_event + 1u64,
            process: RefCell::new(process),
        };
        env.add_event(new_event);
    }
}

impl Process for StandardProcess {
    fn run(&mut self, env: &mut Environment) {
        (self.process)();
        self.add_new_event(env);
    }
}

/// Struct for deterministic processes executed during a fixed simulation window
pub struct FixedProcess {
    pub process_id: u16,
    pub process_name: String,
    pub process_type: ProcessType,
    pub process_duration: ProcessDuration,
    pub process: fn(),
    pub process_start_event: u64,
    pub process_end_event: u64,
}

impl FixedProcess {
    pub fn add_new_event(&mut self, env: &mut Environment) {
        let process = Box::new(Self {
            process_id: self.process_id,
            process_name: self.process_name.clone(),
            process_type: self.process_type.clone(),
            process_duration: self.process_duration.clone(),
            process: self.process,
            process_end_event: self.process_end_event,
            process_start_event: self.process_start_event,
        });
        let new_event = Event {
            time: env.curr_event + 1u64,
            process: RefCell::new(process),
        };
        if env.curr_event < self.process_end_event && env.curr_event >= self.process_start_event {
            env.add_event(new_event);
        }
    }
}

impl Process for FixedProcess {
    fn run(&mut self, env: &mut Environment) {
        (self.process)();
        self.add_new_event(env);
    }
}

pub trait Distribution {
    fn sample(&self, rng: &mut rand::rngs::ThreadRng) -> f64;
}

/// Struct for stochastic infinite processes. This process will run forever until the simulation truncates.
pub struct StandardRandom {
    pub process_id: u16,
    pub process_name: String,
    pub process_type: ProcessType,
    pub process_duration: ProcessDuration,
    pub process: fn(),
    pub execution_distribution: Box<dyn Distribution>,
}

impl StandardRandom {
    pub fn new(
        process_id: u16,
        process_name: String,
        process: fn(),
        execution_distribution: Box<dyn Distribution>,
    ) -> Self {
        StandardRandom {
            process_id: process_id,
            process_name: process_name,
            process_type: ProcessType::Stochastic,
            process_duration: ProcessDuration::Infinite,
            process: process,
            execution_distribution: execution_distribution,
        }
    }

    pub fn add_new_event(&mut self, env: &mut Environment) {
        let rng = rand::thread_rng();
        let time_delta = self.execution_distribution.sample(&mut rng).round() as u64;

        let process = Box::new(Self {
            process_id: self.process_id,
            process_name: self.process_name.clone(),
            process_type: self.process_type.clone(),
            process_duration: self.process_duration.clone(),
            process: self.process,
            execution_distribution: self.execution_distribution.clone(),
        });
        let new_event = Event {
            time: env.curr_event + time_delta,
            process: RefCell::new(process),
        };
        env.add_event(new_event);
    }
}

impl Process for StandardRandom {
    fn run(&mut self, env: &mut Environment) {
        (self.process)();
        self.add_new_event(env);
    }
}

/// Struct for stochastic processes executed during a fixed simulation window
pub struct FixedRandom {
    pub process_id: u16,
    pub process_name: String,
    pub process_type: ProcessType,
    pub process_duration: ProcessDuration,
    pub process: fn(),
    pub execution_distribution: Box<dyn Distribution>,
    pub process_start_event: u64,
    pub process_end_event: u64,
}

impl FixedRandom {
    pub fn new(
        process_id: u16,
        process_name: String,
        process: fn(),
        execution_distribution: Box<dyn Distribution>,
        process_start_event: u64,
        process_end_event: u64,
    ) -> Self {
        FixedRandom {
            process_id: process_id,
            process_name: process_name,
            process_type: ProcessType::Stochastic,
            process_duration: ProcessDuration::Infinite,
            process: process,
            execution_distribution: execution_distribution,
            process_start_event: process_start_event,
            process_end_event: process_end_event,
        }
    }

    pub fn add_new_event(&mut self, env: &mut Environment) {
        let rng = rand::thread_rng();
        let time_delta = self.execution_distribution.sample(&mut rng).round() as u64;

        let process = Box::new(Self {
            process_id: self.process_id,
            process_name: self.process_name.clone(),
            process_type: self.process_type.clone(),
            process_duration: self.process_duration.clone(),
            process: self.process,
            execution_distribution: self.execution_distribution.clone(),
            process_start_event: self.process_start_event,
            process_end_event: self.process_end_event,
        });
        let new_event = Event {
            time: env.curr_event + time_delta,
            process: RefCell::new(process),
        };
        if env.curr_event < self.process_end_event && env.curr_event >= self.process_start_event {
            env.add_event(new_event);
        }
    }
}

impl Process for FixedRandom {
    fn run(&mut self, env: &mut Environment) {
        (self.process)();
        self.add_new_event(env);
    }
}
