//! The environment module contains the Environment struct, which is the main
//! struct of the library. It contains the `processes`, `events`, and `stores` of the
//! simulation. The [`Environment`] struct is responsible for running the simulation
//! and storing the results.

use rand::SeedableRng;

use crate::distribution::Distribution;
use crate::resources::*;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

/// The type of process accepted by aika. Processes are generators that yields a value of type `T` and returns `()`.
pub type Process<T> = Box<dyn Generator<State<T>, Yield = T, Return = ()> + Unpin>;

/// The type of function describing the event time delta for a given process. It can be constant, deterministic, or stochastic.
pub enum ProcessExecution {
    /// Process will only execute once
    Standard,
    /// Constant process execution. The process will execute at a constant time delta.
    Constant(u64, ProcessDuration),
    /// Deterministic process execution. The process will execute at a time delta given by the function.
    Deterministic(fn(u64) -> u64, ProcessDuration),
    /// Stochastic process execution. The process will execute at a time delta given by the distribution.
    Stochastic(Box<dyn Distribution>, ProcessDuration),
}

/// The type of process duration. It can be standard, infinite, or finite.
pub enum ProcessDuration {
    /// Infinite process duration. The process will run from the given event time until the simulation is complete.
    Infinite(u64),
    /// Finite process duration. The process will run from the given start event time until the given end event time.
    Finite(u64, u64),
}

/// The full process discription for the environment. It contains the process, the time delta, and the process duration.
pub struct SimProcess<T> {
    process: Process<T>,
    time_delta: ProcessExecution,
}

impl<T> SimProcess<T> {
    fn new(
        process: Process<T>,
        time_delta: ProcessExecution,
    ) -> Self {
        SimProcess {
            process: process,
            time_delta: time_delta,
        }
    }
}

pub trait EventYield {
    fn output(&self) -> Yield;
    fn set(&mut self, output: Yield);
}

#[derive(Clone)]
/// Event struct. Contains information on which process to execute and when.
pub struct Event<T> {
    /// The time at which the event occurs in the chain.
    pub time: u64,
    /// The id of the process to execute.
    pub process_id: usize,
    /// Simulation state
    pub state: T,
}

impl<T> Ord for Event<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl<T> PartialOrd for Event<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl<T> PartialEq for Event<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<T> Eq for Event<T> {}

#[derive(Clone)]
pub struct State<T> {
    pub state: T,
    pub time: u64,
}

#[derive(Copy, Clone)]
pub enum Yield {
    Timeout(u64),
    Pause,
    AddEvent {
        time_delta: u64,
        process_id: usize,
    },
    RequestResource(usize),
    ReleaseResource(usize),
    GetContainer(usize),
    PutContainer(usize),
    GetStore(usize),
    PutStore(usize),
}

impl EventYield for Yield {
    fn output(&self) -> Yield {
        *self
    }
    fn set(&mut self, output: Yield) {
        *self = output;
    }
}

/// The main struct of the library. It contains the `processes`, `events`, and `stores` of the
/// simulation and keeps track of the current event time.
pub struct Environment<T: EventYield + Clone> {
    /// The events to be executed.
    pub events: BinaryHeap<Reverse<Event<T>>>,
    /// Past events.
    pub past_events: Vec<Event<T>>,
    /// The processes and their id.
    pub processes: HashMap<usize, SimProcess<T>>,
    /// The stores of the simulation yield.
    pub stores: Vec<Stores<T>>,
    /// The containers of the simulation yield.
    pub containers: Vec<Containers<T>>,
    /// The resources of the simulation yield.
    pub resources: Vec<Resources<T>>,
    /// The current event time.
    pub time: u64,
    /// The maximum event time.
    pub stop: u64,
    /// Seeded random number generator for optional randomness.
    pub rng: rand::rngs::StdRng,
    /// Logging boolean
    pub logs: bool,
}

/// Implementation of the Environment struct. Contains public methods `new`, `add_process`, `run`.
impl<T: EventYield + Clone + Default + PartialOrd + Arithmetic<T>> Environment<T> {
    pub fn new(stop: u64, seed: u64) -> Self {
        Environment {
            events: BinaryHeap::new(),
            past_events: Vec::new(),
            processes: HashMap::new(),
            stores: Vec::new(),
            containers: Vec::new(),
            resources: Vec::new(),
            time: 0,
            stop: stop,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            logs: false,
        }
    }

    /// Add a new process to the simulation environment.
    pub fn add_process(
        &mut self,
        process: Box<dyn Generator<State<T>, Yield = T, Return = ()> + Unpin>,
        time_delta: ProcessExecution,
    ) {
        let id = self.processes.len();
        let process = SimProcess::new(process, time_delta);
        self.processes.insert(id, process);
    }

    pub fn create_stores(&mut self, capacity: usize) {
        self.stores.push(Stores::new(capacity));
    }

    pub fn create_containers(&mut self, capacity: T, init: T) {
        self.containers.push(Containers::new(capacity, init));
    }

    pub fn create_resources(&mut self, capacity: usize) {
        self.resources.push(Resources::new(capacity));
    }

    /// Execute the next event in the event queue the store the yield in stores.
    fn step(&mut self) {
        let event = self.events.pop().unwrap().0;
        let process_id = event.process_id;
        self.time = event.time;
        let sim_process = self.processes.get_mut(&process_id).unwrap();
        let process = Pin::new(&mut sim_process.process);
        let time_delta: u64;
        match &sim_process.time_delta {
            ProcessExecution::Standard => {
                time_delta = 0;
            }
            ProcessExecution::Constant(delta, duration) => {
                match duration {
                    ProcessDuration::Infinite(_) => {
                        time_delta = *delta;
                    }
                    ProcessDuration::Finite(_start, end) => {
                        if end < &self.time {
                            return;
                        }
                        time_delta = *delta;
                    }
                }
            }
            ProcessExecution::Deterministic(events_path, duration) => {
                match duration {
                    ProcessDuration::Infinite(_) => {
                        time_delta = events_path(self.time);
                    }
                    ProcessDuration::Finite(_start, end) => {
                        if end < &self.time {
                            return;
                        }
                        time_delta = events_path(self.time);
                    }
                }
            }
            ProcessExecution::Stochastic(distribution_sample, duration) => {
                match duration {
                    ProcessDuration::Infinite(_) => {
                        time_delta = distribution_sample.sample(&mut self.rng).round() as u64;
                    }
                    ProcessDuration::Finite(_start, end) => {
                        if end < &self.time {
                            return;
                        }
                        time_delta = distribution_sample.sample(&mut self.rng).round() as u64;
                    }
                }
            }
        }
        match process.resume(State {
            state: event.state,
            time: self.time,
        }) {
            GeneratorState::Yielded(val) => {
                match val.output() {
                    Yield::Timeout(delta) => {
                        self.add_events(process_id, delta as u64, val.clone());
                    },
                    Yield::Pause => {},
                    Yield::AddEvent { time_delta, process_id } => {
                        self.add_events(process_id, time_delta, val.clone());
                    },
                    Yield::RequestResource(r) => {
                        let resource = self.resources.get_mut(r).unwrap();
                        resource.request(Event {
                            time: self.time,
                            process_id: process_id,
                            state: val.clone(),
                        }).unwrap();
                    },
                    Yield::ReleaseResource(r) => {
                        let resource = self.resources.get_mut(r).unwrap();
                        resource.release(Event {
                            time: self.time,
                            process_id: process_id,
                            state: val.clone(),
                        }).unwrap();
                    },
                    Yield::GetContainer(c) => {
                        let container = self.containers.get_mut(c).unwrap();
                        container.get(val.clone()).unwrap();
                    },
                    Yield::PutContainer(c) => {
                        let container = self.containers.get_mut(c).unwrap();
                        container.put(val.clone());
                    },
                    Yield::GetStore(s) => {
                        let store = self.stores.get_mut(s).unwrap();
                        store.get(Event {
                            time: self.time,
                            process_id: process_id,
                            state: val.clone(),
                        }).unwrap();
                    },
                    Yield::PutStore(s) => {
                        let store = self.stores.get_mut(s).unwrap();
                        store.put(Event {
                            time: self.time,
                            process_id: process_id,
                            state: val.clone(),
                        });
                    },

                }
                self.add_events(process_id, time_delta, val)
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
    pub fn add_events(&mut self, id: usize, time_delta: u64, state: T) {
        if self.time + time_delta > self.stop {
            return;
        } else if time_delta == 0 {
            return;
        }
        self.events.push(Reverse(Event {
            time: self.time + time_delta,
            process_id: id,
            state: state,
        }));
    }

    pub fn set_logs(&mut self, logs: bool) {
        self.logs = logs;
    }
}
