use rand::Rng;
use rand_distr::{Gamma as GammaDistribution, Poisson as PoissonDistribution};

pub trait Distribution {
    fn sample(&self, rng: &mut rand::rngs::StdRng) -> f64;
}

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
