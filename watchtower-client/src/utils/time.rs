use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub fn get_time_since_epoch() -> Result<u64, SystemTimeError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(now) => Ok(now.as_secs() as u64),
        Err(error) => Err(error)
    }
}