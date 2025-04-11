use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId {
    pub(crate) value: u32,
}

impl InstanceId {
    pub fn to_docker_id(self) -> String {
        format!("flecs-{self}")
    }

    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn new_random() -> Self {
        Self {
            value: rand::random::<u32>(),
        }
    }
}

impl From<u32> for InstanceId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl FromStr for InstanceId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u32::from_str_radix(s, 16).map(Self::new)
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_instance_id() {
        assert_eq!("00000000", InstanceId { value: 0 }.to_string());
        assert_eq!("00000001", InstanceId { value: 1 }.to_string());
        assert_eq!("000000f0", InstanceId { value: 240 }.to_string());
        assert_eq!("0003da33", InstanceId { value: 252467 }.to_string());
    }
}
