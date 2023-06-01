//! Distribution module. Contains the `Distribution` trait which allows for the creation of custom distributions to be used in the `ProcessExecution::Stochastic` variant.
//! Distributions must enforce a sampling of only positive real numbers, as this describes a time delta moving forward.

use rand::Rng;
use rand_distr::{Gamma as GammaDistribution, Poisson as PoissonDistribution};

/// The `Distribution` trait allows for the creation of custom distributions to be used in the `ProcessExecution::Stochastic` variant.
pub trait Distribution {
    /// Sample the distribution for time delta value.
    fn sample(&self, rng: &mut rand::rngs::StdRng) -> f64;
}

/// The `Poisson` struct implements the `Distribution` trait for the Poisson distribution.
pub struct Poisson {
    pub distribution: PoissonDistribution<f64>,
}

impl Poisson {
    pub fn new(lambda: f64) -> Poisson {
        Self {
            distribution: PoissonDistribution::new(lambda).unwrap(),
        }
    }
}

impl Distribution for Poisson {
    fn sample(&self, rng: &mut rand::rngs::StdRng) -> f64 {
        rng.sample(self.distribution)
    }
}

/// The `Gamma` struct implements the `Distribution` trait for the Gamma distribution.
pub struct Gamma {
    pub distribution: GammaDistribution<f64>,
}

impl Gamma {
    pub fn new(shape: f64, scale: f64) -> Gamma {
        Self {
            distribution: GammaDistribution::new(shape, scale).unwrap(),
        }
    }
}

impl Distribution for Gamma {
    fn sample(&self, rng: &mut rand::rngs::StdRng) -> f64 {
        rng.sample(self.distribution)
    }
}
