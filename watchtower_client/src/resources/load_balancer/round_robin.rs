use rand::Rng;
use crate::resources::load_balancer::LoadBalancer;

pub struct RoundRobinLoadBalancer {
    current_index: usize,
    array_size: usize
}

impl RoundRobinLoadBalancer {
    pub fn new(array_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let current_index = rng.gen_range(0..array_size);
        RoundRobinLoadBalancer {
            current_index,
            array_size
        }
    }
}

impl LoadBalancer for RoundRobinLoadBalancer {
    fn get_next_index(&mut self) -> usize {
        self.current_index = (self.current_index + 1) % self.array_size;
        self.current_index        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::load_balancer::LoadBalancer;

    #[test]
    fn test_get_next_index() {
        let size = 3;

        let mut load_balancer = RoundRobinLoadBalancer::new(size);
        let start_index = load_balancer.get_next_index();

        assert_eq!((start_index + 1) % size, load_balancer.get_next_index());
        assert_eq!((start_index + 2) % size, load_balancer.get_next_index());
        assert_eq!((start_index + 3) % size, load_balancer.get_next_index());
    }
}