#![feature(generators, generator_trait)]

pub mod distribution;
pub mod environment;
pub mod manager;
pub mod resources;

#[cfg(test)]
mod test {
    use super::distribution::*;
    use super::environment::*;

    #[test]
    fn setup_simple_des() {
        let seed = 223u64;

        impl EventYield for f64 {
            fn output(&self) -> Yield {
                Yield::Pause
            }
            fn set(&mut self, output: Yield) {
            }
        }

        let mut env = Environment::new(100, seed);

        env.create_containers(1000f64, 300f64);
        let process_random = Box::new(move |_| {
            loop {
                let i = env.containers[0].get(1.0).unwrap();
                yield i;
            }
        });
        let process = Box::new(move |_| {
            loop {
                env.containers[0].put(1.0);
                yield 1.0;
            }
        });
        // Execution Distribution
        let gamma = Gamma::new(7.0, 1.0);
    }
}
