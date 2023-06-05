//! The environment module contains the Environment struct, which is the main
//! struct of the library. It contains the `processes`, `events`, and `stores` of the
//! simulation. The [`Environment`] struct is responsible for running the simulation
//! and storing the results.

use rand::SeedableRng;

use crate::distribution::Distribution;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

/// The type of process accepted by aika. Processes are generators that yields a value of type `T` and returns `()`.
pub type Process<T> = Box<dyn Generator<State<T>, Yield = T, Return = ()> + Unpin>;

/// The type of function describing the event time delta for a given process. It can be constant, deterministic, or stochastic.
pub enum ProcessExecution {
    /// Constant process execution. The process will execute at a constant time delta.
    Constant(u64),
    /// Deterministic process execution. The process will execute at a time delta given by the function.
    Deterministic(fn(u64) -> u64),
    /// Stochastic process execution. The process will execute at a time delta given by the distribution.
    Stochastic(Box<dyn Distribution>),
}

/// The type of process duration. It can be standard, infinite, or finite.
pub enum ProcessDuration {
    /// Standard process duration. The process will run from the first event time until the simulation is complete.
    Standard,
    /// Infinite process duration. The process will run from the given event time until the simulation is complete.
    Infinite(u64),
    /// Finite process duration. The process will run from the given start event time until the given end event time.
    Finite(u64, u64),
}

/// The full process discription for the environment. It contains the process, the time delta, and the process duration.
pub struct SimProcess<T> {
    process: Process<T>,
    time_delta: ProcessExecution,
    process_duration: ProcessDuration,
}

impl<T> SimProcess<T> {
    fn new(
        process: Process<T>,
        time_delta: ProcessExecution,
        process_duration: ProcessDuration,
    ) -> Self {
        SimProcess {
            process: process,
            time_delta: time_delta,
            process_duration: process_duration,
        }
    }
}

/// Event struct. Contains information on which process to execute and when.
pub struct Event {
    /// The time at which the event occurs in the chain.
    pub time: u64,
    /// The id of the process to execute.
    pub process_id: usize,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Event {}

#[derive(Clone)]
pub struct State<T> {
    pub state: T,
    pub time: u64,
}

/// The main struct of the library. It contains the `processes`, `events`, and `stores` of the
/// simulation and keeps track of the current event time.
pub struct Environment<T: Clone> {
    /// The events to be executed.
    pub events: BinaryHeap<Reverse<Event>>,
    /// The processes and their id.
    pub processes: HashMap<usize, SimProcess<T>>,
    /// The stores of the simulation yield.
    pub stores: Vec<(u64, T)>,
    /// The current state of the simulation.
    pub state: State<T>,
    /// The current event time.
    pub time: u64,
    /// The maximum event time.
    pub stop: u64,
    /// Seeded random number generator for optional randomness.
    pub rng: rand::rngs::StdRng,
}

/// Implementation of the Environment struct. Contains public methods `new`, `add_process`, `run`.
impl<T: Clone> Environment<T> {
    pub fn new(stop: u64, seed: u64, initial_state: T) -> Self {
        Environment {
            events: BinaryHeap::new(),
            processes: HashMap::new(),
            state: State {
                state: initial_state,
                time: 0,
            },
            time: 0,
            stop: stop,
            stores: Vec::new(),
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    /// Add a new process to the simulation environment.
    pub fn add_process(
        &mut self,
        process: Box<dyn Generator<State<T>, Yield = T, Return = ()> + Unpin>,
        time_delta: ProcessExecution,
        process_duration: ProcessDuration,
    ) {
        let id = self.processes.len();
        let process = SimProcess::new(process, time_delta, process_duration);
        self.processes.insert(id, process);
        self.init_process(id);
    }

    /// Initialize a process by adding its first event to the event queue. Private function to be called in [`run`].
    fn init_process(&mut self, id: usize) {
        let process = self.processes.get(&id).unwrap();
        match process.process_duration {
            ProcessDuration::Standard => {
                self.add_events(id, 0);
            }
            ProcessDuration::Infinite(start) => {
                self.add_events(id, start);
            }
            ProcessDuration::Finite(start, _end) => {
                self.add_events(id, start);
            }
        }
    }

    /// Execute the next event in the event queue the store the yield in stores.
    fn step(&mut self) {
        let event = self.events.pop().unwrap().0;
        let process_id = event.process_id;
        self.time = event.time;
        let sim_process = self.processes.get_mut(&process_id).unwrap();
        match sim_process.process_duration {
            ProcessDuration::Finite(_start, end) => {
                if self.time >= end {
                    return;
                }
            }
            _ => {}
        }
        let process = Pin::new(&mut sim_process.process);
        let time_delta: u64;
        match &sim_process.time_delta {
            ProcessExecution::Constant(delta) => {
                time_delta = *delta;
            }
            ProcessExecution::Deterministic(events_path) => {
                time_delta = events_path(self.time);
            }
            ProcessExecution::Stochastic(distribution_sample) => {
                time_delta = distribution_sample.sample(&mut self.rng).round() as u64;
            }
        }
        match process.resume(self.state.clone()) {
            GeneratorState::Yielded(val) => {
                self.add_events(process_id, time_delta);
                self.stores.push((self.time, val.clone()));
                self.state = State {
                    state: val.clone(),
                    time: self.time,
                };
                self.time += 1;
            }
            GeneratorState::Complete(_output) => {}
        }
    }

    /// Run the simulation until the maximum event time is reached.
    pub fn run(&mut self) {
        if self.time < self.stop {
            while !self.events.is_empty() {
                self.step();
            }
        } else {
            return;
        }
    }

    /// Add an event to the event queue.
    fn add_events(&mut self, id: usize, time_delta: u64) {
        if self.time + time_delta > self.stop {
            return;
        }
        self.events.push(Reverse(Event {
            time: self.time + time_delta,
            process_id: id,
        }));
    }
}
