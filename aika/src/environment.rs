use crate::process::Process;
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct Event {
    pub time: u64,
    pub process: RefCell<Box<dyn Process>>,
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

pub struct Environment {
    pub events: BinaryHeap<Reverse<Event>>,
    pub curr_event: u64,
    pub max_event: u64,
}

impl Environment {
    pub fn new(max_event: u64) -> Self {
        Environment {
            events: BinaryHeap::new(),
            curr_event: 0,
            max_event: max_event,
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(Reverse(event));
    }

    pub fn run(&mut self) {
        while let Some(Reverse(event)) = self.events.pop() {
            if self.curr_event >= self.max_event {
                break;
            }
            if event.time >= self.curr_event {
                self.curr_event = event.time;
                event.process.borrow_mut().run(self);
                self.curr_event += 1;
            } else {
                panic!("Event time is less than current time, events out of order");
            }
        }
    }
}
