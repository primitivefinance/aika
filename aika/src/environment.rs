use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub type Process<T> = Box<dyn Generator<Yield = T, Return = ()> + Unpin>;

pub trait Distribution {
    fn sample(&self) -> f64;
}

pub struct Event {
    pub time: u64,
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

pub struct Environment<T> {
    pub events: BinaryHeap<Reverse<Event>>,
    pub processes: HashMap<usize, SimProcess<T>>,
    pub curr_event: u64,
    pub max_event: u64,
    pub state_chain: Vec<T>,
}

impl<T> Environment<T> {
    pub fn new(max_event: u64) -> Self {
        Environment {
            events: BinaryHeap::new(),
            processes: HashMap::new(),
            curr_event: 0,
            max_event: max_event,
            state_chain: Vec::new(),
        }
    }

    pub fn add_process(
        &mut self,
        process: Box<dyn Generator<Yield = T, Return = ()> + Unpin>,
        time_delta: ProcessExecution,
    ) {
        let id = self.processes.len();
        let process = SimProcess::new(process, time_delta);
        self.processes.insert(id, process);
    }

    pub fn run(&mut self) {
        let event = self.events.pop().unwrap().0;
        let process_id = event.process_id;
        self.curr_event = event.time;
        let sim_process = self.processes.get_mut(&process_id).unwrap();
        let process = Pin::new(&mut sim_process.process);
        let time_delta: u64;
        match sim_process.time_delta {
            ProcessExecution::Constant(delta) => {
                time_delta = delta;
            }
            ProcessExecution::Deterministic(events_path) => {
                time_delta = events_path(self.curr_event);
            }
            ProcessExecution::Stochastic(distribution_sample) => {
                time_delta = distribution_sample().round() as u64;
            }
        }
        match process.resume(()) {
            GeneratorState::Yielded(val) => {
                self.add_events(process_id, time_delta);
                self.state_chain.push(val);
                self.curr_event += 1;
            }
            GeneratorState::Complete(_output) => {}
        }
    }

    pub fn add_events(&mut self, id: usize, time_delta: u64) {
        if self.curr_event + time_delta > self.max_event {
            return;
        }
        self.events.push(Reverse(Event {
            time: self.curr_event + time_delta,
            process_id: id,
        }));
    }
}

pub enum ProcessExecution {
    Constant(u64),
    Deterministic(fn(u64) -> u64),
    Stochastic(fn() -> f64),
}

pub struct SimProcess<T> {
    process: Process<T>,
    time_delta: ProcessExecution,
}

impl<T> SimProcess<T> {
    fn new(process: Process<T>, time_delta: ProcessExecution) -> Self {
        SimProcess {
            process: process,
            time_delta: time_delta,
        }
    }
}

