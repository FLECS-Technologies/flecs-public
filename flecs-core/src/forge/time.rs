use std::time::{SystemTime, UNIX_EPOCH};

pub trait SystemTimeExt {
    fn unix_millis(&self) -> u128;
}

impl SystemTimeExt for SystemTime {
    fn unix_millis(&self) -> u128 {
        self.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }
}
