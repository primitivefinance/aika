use std::collections::BTreeMap;

use crate::environment::Environment;

/// The `Manager` struct is responsible for running a series of simulations and storing the results.
pub struct Manager<T: Clone> {
    /// The number of simulations to run.
    pub simulations: Vec<Environment<T>>,
    /// The storage of simulation data
    pub stores: Vec<BTreeMap<u64, T>>,
}

impl<T: Clone> Manager<T> {
    /// Create a new `Manager` struct.
    pub fn new() -> Self {
        Manager {
            simulations: Vec::new(),
            stores: Vec::new(),
        }
    }

    /// Add a simulation to the `Manager` struct.
    pub fn add_simulation(&mut self, simulation: Environment<T>) {
        self.simulations.push(simulation);
    }

    /// Run all simulations in the `Manager` struct.
    pub fn run(&mut self) {
        for simulation in &mut self.simulations {
            simulation.run();
            self.stores.push(simulation.stores.clone());
        }
    }
}
