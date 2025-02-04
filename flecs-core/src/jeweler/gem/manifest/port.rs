pub use super::{Error, Result};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct PortRange {
    start: u16,
    end: u16,
}

impl PortRange {
    pub(crate) fn len(&self) -> usize {
        (1 + self.end - self.start) as usize
    }

    pub fn try_new(start: u16, end: u16) -> Result<Self> {
        anyhow::ensure!(
            start <= end,
            "The start of a port range can not be smaller than the end"
        );
        Ok(Self { start, end })
    }

    pub fn range(&self) -> RangeInclusive<u16> {
        self.start..=self.end
    }
}

impl FromStr for PortRange {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let splits = s.split('-').collect::<Vec<_>>();
        anyhow::ensure!(
            splits.len() == 2,
            "Expected two values separated by '-', received {} parts: {s}",
            splits.len()
        );
        let start = u16::from_str(splits[0])?;
        let end = u16::from_str(splits[1])?;
        PortRange::try_new(start, end)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum PortMapping {
    Single(u16, u16),
    Range { from: PortRange, to: PortRange },
}

impl From<&PortMapping> for flecsd_axum_server::models::InstanceDetailPort {
    fn from(value: &PortMapping) -> Self {
        match value {
            PortMapping::Single(host_port, container_port) => Self {
                container: container_port.to_string(),
                host: host_port.to_string(),
            },
            PortMapping::Range { from, to } => Self {
                host: format!("{}-{}", from.start, from.end),
                container: format!("{}-{}", to.start, to.end),
            },
        }
    }
}

impl FromStr for PortMapping {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<_>>();
        let (from, to) = match splits.len() {
            1 => (splits[0], splits[0]),
            2 => (splits[0], splits[1]),
            x => anyhow::bail!(
                "Expected two port ranges separated by ':', received {x} elements separated by ':'"
            ),
        };
        if from.contains('-') {
            let from = PortRange::from_str(from)?;
            let to = PortRange::from_str(to)?;
            anyhow::ensure!(
                from.len() == to.len(),
                "Only port ranges of equal size can be mapped: 'from' contains {} ports while 'to' contains {}",
                from.len(),
                to.len()
            );
            Ok(Self::Range { from, to })
        } else {
            Ok(Self::Single(u16::from_str(from)?, u16::from_str(to)?))
        }
    }
}

impl TryFrom<&flecs_app_manifest::generated::manifest_3_1_0::PortsItem> for PortMapping {
    type Error = Error;

    fn try_from(
        value: &flecs_app_manifest::generated::manifest_3_1_0::PortsItem,
    ) -> std::result::Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_port_range_ok() {
        assert_eq!(
            PortRange {
                start: 100,
                end: 200
            },
            PortRange::try_new(100, 200).unwrap()
        )
    }

    #[test]
    fn try_new_port_range_err() {
        assert!(PortRange::try_new(200, 100).is_err())
    }

    #[test]
    fn port_range_len() {
        assert_eq!(
            PortRange {
                start: 100,
                end: 200
            }
            .len(),
            101
        );
        assert_eq!(PortRange { start: 1, end: 1 }.len(), 1);
    }

    #[test]
    fn range_from_str_ok() {
        assert_eq!(
            PortRange {
                start: 100,
                end: 1000
            },
            PortRange::from_str("100-1000").unwrap()
        )
    }

    #[test]
    fn range_from_str_too_many_parts() {
        assert!(PortRange::from_str("100-1000-10000").is_err())
    }

    #[test]
    fn range_from_str_too_little_parts() {
        assert!(PortRange::from_str("100").is_err())
    }

    #[test]
    fn range_from_str_invalid_start() {
        assert!(PortRange::from_str("100k-1000").is_err())
    }

    #[test]
    fn range_from_str_invalid_end() {
        assert!(PortRange::from_str("10-1000µ").is_err())
    }

    #[test]
    fn port_mapping_from_str_single_ok() {
        assert_eq!(
            PortMapping::Single(10, 100),
            PortMapping::from_str("10:100").unwrap()
        )
    }

    #[test]
    fn port_mapping_from_str_single_eq_ok() {
        assert_eq!(
            PortMapping::Single(100, 100),
            PortMapping::from_str("100").unwrap()
        )
    }

    #[test]
    fn port_mapping_from_str_range_ok() {
        assert_eq!(
            PortMapping::Range {
                from: PortRange {
                    start: 10,
                    end: 100
                },
                to: PortRange {
                    start: 200,
                    end: 290
                }
            },
            PortMapping::from_str("10-100:200-290").unwrap()
        )
    }

    #[test]
    fn port_mapping_from_str_range_eq_ok() {
        assert_eq!(
            PortMapping::Range {
                from: PortRange {
                    start: 10,
                    end: 100
                },
                to: PortRange {
                    start: 10,
                    end: 100
                }
            },
            PortMapping::from_str("10-100").unwrap()
        )
    }

    #[test]
    fn port_mapping_from_str_range_different_size() {
        assert!(PortMapping::from_str("100-200:300-500").is_err())
    }

    #[test]
    fn port_mapping_from_str_too_many_parts() {
        assert!(PortMapping::from_str("10:100:200").is_err())
    }

    #[test]
    fn try_port_mapping_from_ports_item_ok() {
        let item =
            flecs_app_manifest::generated::manifest_3_1_0::PortsItem::from_str("50-60:150-160")
                .unwrap();
        assert_eq!(
            PortMapping::Range {
                from: PortRange { start: 50, end: 60 },
                to: PortRange {
                    start: 150,
                    end: 160
                }
            },
            PortMapping::try_from(&item).unwrap()
        )
    }

    #[test]
    fn try_port_mapping_from_ports_item_err() {
        let item =
            flecs_app_manifest::generated::manifest_3_1_0::PortsItem::from_str("50-60:150-260")
                .unwrap();
        assert!(PortMapping::try_from(&item).is_err())
    }

    #[test]
    fn port_mapping_to_instance_detail_port_single() {
        let port_mapping = &PortMapping::Single(1, 2);
        let result: flecsd_axum_server::models::InstanceDetailPort = port_mapping.into();
        assert_eq!(
            result,
            flecsd_axum_server::models::InstanceDetailPort {
                host: "1".to_string(),
                container: "2".to_string(),
            }
        );
    }

    #[test]
    fn port_mapping_to_instance_detail_port_range() {
        let port_mapping = &PortMapping::Range {
            from: PortRange {
                start: 100,
                end: 200,
            },
            to: PortRange {
                start: 800,
                end: 900,
            },
        };
        let result: flecsd_axum_server::models::InstanceDetailPort = port_mapping.into();
        assert_eq!(
            result,
            flecsd_axum_server::models::InstanceDetailPort {
                host: "100-200".to_string(),
                container: "800-900".to_string(),
            }
        );
    }

    #[test]
    fn port_range_range() {
        let range = PortRange {
            start: 100,
            end: 200,
        };
        assert_eq!(range.range(), 100..=200)
    }
}
