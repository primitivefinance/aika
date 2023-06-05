#![feature(generators, generator_trait)]

pub mod distribution;
pub mod environment;
pub mod manager;

#[cfg(test)]
mod test {
    use super::distribution::*;
    use super::environment::*;

    #[test]
    fn setup_simple_des() {
        let seed = 223u64;

        let mut env = Environment::new(100, seed, 0);
        let process_random = Box::new(move |state: State<i32>| {
            loop {
                let mut i = state.state;
                i += 3;
                yield i;
            }
        });
        let process = Box::new(move |state: State<i32>| {
            loop {
                let mut i = state.state;
                i -= 1;
                yield i;
            }
        });
        // Execution Distribution
        let gamma = Gamma::new(7.0, 1.0);

        env.add_process(
            process_random,
            ProcessExecution::Stochastic(Box::new(gamma)),
            ProcessDuration::Finite(30, 60),
        );
        env.add_process(
            process,
            ProcessExecution::Constant(3),
            ProcessDuration::Standard,
        );
        env.run();
        println!("{:?}", env.stores);
    }
}
