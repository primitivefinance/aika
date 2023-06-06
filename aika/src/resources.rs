use std::collections::VecDeque;

use crate::environment::Event;

#[derive(Clone)]
pub struct Stores<T> {
    capacity: usize,
    gets: VecDeque<Event<T>>,
    puts: VecDeque<Event<T>>,
    state: VecDeque<Event<T>>,
}

impl<T> Stores<T> {
    pub fn new(capacity: usize) -> Self {
        Stores {
            capacity: capacity,
            gets: VecDeque::new(),
            puts: VecDeque::new(),
            state: VecDeque::new(),
        }
    }

    pub fn get(&mut self, event: Event<T>) -> Result<Event<T>, &'static str> {
        let process_id = event.process_id;
        let time = event.time;
        if let Some(mut event) = self.state.pop_front() {
            if let Some(put) = self.puts.pop_front() {
                self.state.push_back(put);
            };
            event = Event {
                time: time,
                process_id: process_id,
                state: event.state,
            };
            Ok(event)
        } else {
            self.gets.push_back(event);
            Err("Cannot get from empty store")
        }
    }

    pub fn put(&mut self, event: Event<T>) {
        if self.state.len() < self.capacity {
            self.state.push_back(event);
        } else {
            self.puts.push_back(event);
        }
        
    }
}

#[derive(Clone)]
pub struct Resources<T> {
    capacity: usize,
    left: usize,
    queue: VecDeque<Event<T>>,
}

impl<T> Resources<T> {
    pub fn new(capacity: usize) -> Self {
        Resources {
            capacity: capacity,
            left: capacity,
            queue: VecDeque::new(),
        }
    }

    pub fn request(&mut self, event: Event<T>) -> Result<Event<T>, &'static str> {
        let process_id = event.process_id;
        let time = event.time;
        if self.left > 0 {
            self.left -= 1;
            let event = Event {
                time: time,
                process_id: process_id,
                state: event.state,
            };
            Ok(event)
        } else {
            self.queue.push_back(event);
            Err("Cannot request from empty resource")
        }
    }

    pub fn release(&mut self, event: Event<T>) -> Option<Event<T>> {
        let time = event.time;
        if let Some(event) = self.queue.pop_front() {
            Some(Event {
                time: time,
                process_id: event.process_id,
                state: event.state,
            })
        } else {
            assert!(self.left < self.capacity);
            self.left += 1;
            None
        }
    }
}

#[derive(Clone)]
pub struct Containers<T> {
    container_capacity: T,
    init: T,
    chain: Vec<T>,
}

impl<T: Clone + Default + PartialOrd + Arithmetic<T>> Containers<T> {
    pub fn new(capacity: T, init: T) -> Self {
        let mut vec = Vec::new();
        vec.push(init);
        Containers {
            container_capacity: capacity,
            init: T::default(),
            chain: vec,
        }
    }

    pub fn get(&mut self, amount: T) -> Result<T, &'static str> {
        let mut total = T::default();
        if amount >= T::default() {
            if let Some(value) = self.chain.pop() {
                if value >= amount {
                    total = amount.clone();
                    let new_value = value.sub(amount);
                    self.chain.push(new_value);
                } else {
                    return Err("Cannot have negative reserves");
                }
            }
        } else {
            return Err("Cannot have negative get amount");
        }
        Ok(total.clone())
    }

    pub fn put(&mut self, amount:T) {
        if let Some(value) = self.chain.pop() {
            if value.add(amount.clone()) > self.container_capacity {
                self.chain.push(self.container_capacity.clone());
            } else {
                self.chain.push(value.add(amount.clone()));
            }
        }
    }
}

pub trait Arithmetic<T> {
    fn add(&self, other: T) -> T;
    fn sub(&self, other: T) -> T;
}

impl Arithmetic<i32> for i32 {
    fn add(&self, other: i32) -> i32 {
        self + other
    }

    fn sub(&self, other: i32) -> i32 {
        self - other
    }
}

impl Arithmetic<f64> for f64 {
    fn add(&self, other: f64) -> f64 {
        self + other
    }

    fn sub(&self, other: f64) -> f64 {
        self - other
    }
}

impl Arithmetic<f32> for f32 {
    fn add(&self, other: f32) -> f32 {
        self + other
    }

    fn sub(&self, other: f32) -> f32 {
        self - other
    }
}

impl Arithmetic<u64> for u64 {
    fn add(&self, other: u64) -> u64 {
        self + other
    }

    fn sub(&self, other: u64) -> u64 {
        self - other
    }
}

impl Arithmetic<u32> for u32 {
    fn add(&self, other: u32) -> u32 {
        self + other
    }

    fn sub(&self, other: u32) -> u32 {
        self - other
    }
}

impl Arithmetic<u16> for u16 {
    fn add(&self, other: u16) -> u16 {
        self + other
    }

    fn sub(&self, other: u16) -> u16 {
        self - other
    }
}

impl Arithmetic<u8> for u8 {
    fn add(&self, other: u8) -> u8 {
        self + other
    }

    fn sub(&self, other: u8) -> u8 {
        self - other
    }
}

impl Arithmetic<i64> for i64 {
    fn add(&self, other: i64) -> i64 {
        self + other
    }

    fn sub(&self, other: i64) -> i64 {
        self - other
    }
}

impl Arithmetic<i16> for i16 {
    fn add(&self, other: i16) -> i16 {
        self + other
    }

    fn sub(&self, other: i16) -> i16 {
        self - other
    }
}

impl Arithmetic<i8> for i8 {
    fn add(&self, other: i8) -> i8 {
        self + other
    }

    fn sub(&self, other: i8) -> i8 {
        self - other
    }
}

impl Arithmetic<usize> for usize {
    fn add(&self, other: usize) -> usize {
        self + other
    }

    fn sub(&self, other: usize) -> usize {
        self - other
    }
}

impl Arithmetic<isize> for isize {
    fn add(&self, other: isize) -> isize {
        self + other
    }

    fn sub(&self, other: isize) -> isize {
        self - other
    }
}