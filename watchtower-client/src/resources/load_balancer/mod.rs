mod round_robin;

pub trait LoadBalancer {
    fn get_next_index(&mut self) -> usize;
}

pub use round_robin::RoundRobinLoadBalancer;