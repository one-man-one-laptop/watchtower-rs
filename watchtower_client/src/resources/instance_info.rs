use std::cmp::{Ord, PartialOrd, PartialEq, Ordering};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug, Eq)]
pub struct InstanceInfo {
    pub instance_id: String,
    pub ip_addr: String,
    pub port: u16
}

impl Ord for InstanceInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.instance_id.cmp(&other.instance_id)
    }
}

impl PartialOrd for InstanceInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InstanceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.instance_id == other.instance_id
    }
}
