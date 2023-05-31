#![feature(generators, generator_trait)]

pub mod distribution;
pub mod environment;

#[cfg(test)]
mod test {
    use super::distribution::*;
    use super::environment::*;

    #[test]
    fn setup_simple_des() {
        let seed = 223u64;

        let mut env = Environment::new(100, seed);
        let process_random = Box::new(move || {
            let mut i = 0;
            loop {
                yield i;
                i -= 1;
            }
        });
        // Execution Distribution
        let poisson = Poisson::new(11.0);

        env.add_process(
            process_random,
            ProcessExecution::Stochastic(Box::new(poisson)),
            ProcessDuration::Standard,
        );
        env.run();
        println!("{:?}", env.state_chain);
    }
}
