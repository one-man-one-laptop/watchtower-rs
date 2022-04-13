use crate::{
    resources::{InstanceInfo, 
        load_balancer::{
            RoundRobinLoadBalancer,
            LoadBalancer
        }
    },
    utils::time::get_time_since_epoch,
    types::{Result, Error}
};

pub const UPDATE_INTERVAL: u64 = 30;

pub struct Service {
    pub instance_infos: Vec<InstanceInfo>,
    pub load_balancer: RoundRobinLoadBalancer,
    pub last_updated_timestamp: u64
}

impl Service {
    pub fn new(instance_infos: Vec<InstanceInfo>) -> Self {
        Service {
            load_balancer: RoundRobinLoadBalancer::new(instance_infos.len()),
            instance_infos,
            last_updated_timestamp: get_time_since_epoch().unwrap()
        }
    }

    /// Returns true if the service is expired
    pub fn is_expired(&self) -> Result<bool> {
        Ok((self.last_updated_timestamp + UPDATE_INTERVAL) <= get_time_since_epoch()?)
    }

    /// Gets the next instance for the given service
    pub fn get_next_instance(&mut self) -> Result<InstanceInfo> {
        let index = self.load_balancer.get_next_index();
        match self.instance_infos.get(index) {
            Some(instance_info) => Ok(instance_info.clone()),
            None => Err(Error::InternalError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::instance_info::InstanceInfo;

    #[test]
    fn test_is_expired() {
        let service = Service::new(vec![InstanceInfo {
            instance_id: "test".to_string(),
            ip_addr: "0.0.0.0".to_string(),
            port: 8888
        }]);
        assert_eq!(service.is_expired().unwrap(), false);
        let sleep_time = std::time::Duration::from_millis(UPDATE_INTERVAL * 1000);
        std::thread::sleep(sleep_time);
        assert_eq!(service.is_expired().unwrap(), true);
    }

    #[test]
    fn test_get_next_instance() {
        let instance_info1 = InstanceInfo {
            instance_id: "test1".to_string(),
            ip_addr: "0.0.0.0".to_string(),
            port: 8888
        };
        let instance_info2 = InstanceInfo {
            instance_id: "test2".to_string(),
            ip_addr: "0.0.0.0".to_string(),
            port: 8888
        };
        let mut service = Service::new(vec![instance_info1.clone(), instance_info2.clone()]);
        
        let ret_instance = service.get_next_instance().unwrap();
        assert!(ret_instance == instance_info1 || ret_instance == instance_info2);
        assert_ne!(service.get_next_instance().unwrap(), ret_instance);
        assert_eq!(service.get_next_instance().unwrap(), ret_instance);
    }
}
