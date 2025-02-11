pub use super::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct PortRange {
    start: u16,
    end: u16,
}

impl Display for PortRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
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

    pub fn new(range: RangeInclusive<u16>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
        }
    }

    pub fn range(&self) -> RangeInclusive<u16> {
        self.start..=self.end
    }

    pub fn contains(&self, port: u16) -> bool {
        self.range().contains(&port)
    }

    pub fn overlaps(&self, range: &PortRange) -> bool {
        self.start <= range.end && range.start <= self.end
    }
}

impl From<RangeInclusive<u16>> for PortRange {
    fn from(range: RangeInclusive<u16>) -> Self {
        Self::new(range)
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

impl Display for PortMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(host_port, container_port) => write!(f, "{host_port}:{container_port}"),
            Self::Range { from, to } => write!(f, "{from}:{to}"),
        }
    }
}

impl PortMapping {
    pub fn do_host_ports_overlap(&self, other: &PortMapping) -> bool {
        match (self, other) {
            (PortMapping::Single(host_port, _), PortMapping::Single(other_host_port, _)) => {
                host_port == other_host_port
            }
            (PortMapping::Single(host_port, _), PortMapping::Range { from, .. })
            | (PortMapping::Range { from, .. }, PortMapping::Single(host_port, _)) => {
                from.contains(*host_port)
            }
            (
                PortMapping::Range { from, .. },
                PortMapping::Range {
                    from: other_from, ..
                },
            ) => from.overlaps(other_from),
        }
    }

    pub fn are_host_ports_equal(&self, other: &PortMapping) -> bool {
        match (self, other) {
            (PortMapping::Single(host_port, _), PortMapping::Single(other_host_port, _)) => {
                host_port == other_host_port
            }
            (PortMapping::Single(host_port, _), PortMapping::Range { from, .. })
            | (PortMapping::Range { from, .. }, PortMapping::Single(host_port, _)) => {
                *host_port == from.start && *host_port == from.end
            }
            (
                PortMapping::Range { from, .. },
                PortMapping::Range {
                    from: other_from, ..
                },
            ) => from == other_from,
        }
    }

    pub fn normalize(self) -> Self {
        match self {
            Self::Range { from, to } if from.range().len() == 1 => {
                Self::Single(from.start, to.start)
            }
            x => x,
        }
    }
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
    fn port_range_new() {
        let range = PortRange::new(100..=200);
        assert_eq!(
            range,
            PortRange {
                start: 100,
                end: 200,
            }
        )
    }

    #[test]
    fn port_range_from_range() {
        let range = PortRange::from(100..=200);
        assert_eq!(
            range,
            PortRange {
                start: 100,
                end: 200,
            }
        )
    }

    #[test]
    fn port_range_range() {
        let range = PortRange {
            start: 100,
            end: 200,
        };
        assert_eq!(range.range(), 100..=200)
    }

    #[test]
    fn port_range_contains() {
        let range = PortRange::new(100..=200);
        assert!(!range.contains(1));
        assert!(!range.contains(99));
        assert!(range.contains(100));
        assert!(range.contains(150));
        assert!(range.contains(200));
        assert!(!range.contains(201));
        assert!(!range.contains(2000));
    }

    #[test]
    fn port_range_overlaps() {
        let range = PortRange::new(100..=200);
        assert!(!range.overlaps(&PortRange::new(10..=20)));
        assert!(!range.overlaps(&PortRange::new(40..=99)));
        assert!(range.overlaps(&PortRange::new(50..=100)));
        assert!(range.overlaps(&PortRange::new(100..=120)));
        assert!(range.overlaps(&PortRange::new(101..=120)));
        assert!(range.overlaps(&PortRange::new(150..=160)));
        assert!(range.overlaps(&PortRange::new(160..=199)));
        assert!(range.overlaps(&PortRange::new(170..=200)));
        assert!(range.overlaps(&PortRange::new(180..=201)));
        assert!(range.overlaps(&PortRange::new(199..=220)));
        assert!(range.overlaps(&PortRange::new(200..=250)));
        assert!(!range.overlaps(&PortRange::new(201..=260)));
        assert!(!range.overlaps(&PortRange::new(2000..=3000)));
        assert!(range.overlaps(&PortRange::new(10..=1000)));
    }

    #[test]
    fn port_range_display() {
        let range = PortRange::new(1000..=2000);
        assert_eq!(format!("{}", range), "1000-2000");
    }

    #[test]
    fn port_mapping_display() {
        assert_eq!(format!("{}", PortMapping::Single(10, 100)), "10:100");
        assert_eq!(
            format!(
                "{}",
                PortMapping::Range {
                    from: PortRange::new(10..=100),
                    to: PortRange::new(400..=490),
                }
            ),
            "10-100:400-490"
        );
    }

    #[test]
    fn port_mapping_host_overlap_single_single() {
        let one = PortMapping::Single(10, 100);
        let two = PortMapping::Single(10, 200);
        assert!(one.do_host_ports_overlap(&two));
        assert!(two.do_host_ports_overlap(&one));
        let one = PortMapping::Single(25, 100);
        let two = PortMapping::Single(30, 200);
        assert!(!one.do_host_ports_overlap(&two));
        assert!(!two.do_host_ports_overlap(&one));
    }

    #[test]
    fn port_mapping_host_overlap_range_range() {
        let one = PortMapping::Range {
            from: PortRange::new(10..=100),
            to: PortRange::new(400..=490),
        };
        let two = PortMapping::Range {
            from: PortRange::new(50..=100),
            to: PortRange::new(400..=450),
        };
        assert!(one.do_host_ports_overlap(&two));
        assert!(two.do_host_ports_overlap(&one));
        let one = PortMapping::Range {
            from: PortRange::new(100..=1000),
            to: PortRange::new(1100..=2000),
        };
        let two = PortMapping::Range {
            from: PortRange::new(50..=99),
            to: PortRange::new(400..=449),
        };
        assert!(!one.do_host_ports_overlap(&two));
        assert!(!two.do_host_ports_overlap(&one));
    }

    #[test]
    fn port_mapping_host_overlap_range_single() {
        let one = PortMapping::Range {
            from: PortRange::new(10..=100),
            to: PortRange::new(400..=490),
        };
        let two = PortMapping::Single(10, 200);
        assert!(one.do_host_ports_overlap(&two));
        assert!(two.do_host_ports_overlap(&one));
        let one = PortMapping::Range {
            from: PortRange::new(10..=100),
            to: PortRange::new(400..=490),
        };
        let two = PortMapping::Single(7, 200);
        assert!(!one.do_host_ports_overlap(&two));
        assert!(!two.do_host_ports_overlap(&one));
    }

    #[test]
    fn port_mapping_host_equal_single_single() {
        let one = PortMapping::Single(10, 60);
        let two = PortMapping::Single(10, 70);
        assert!(one.are_host_ports_equal(&two));
        assert!(two.are_host_ports_equal(&one));
        let one = PortMapping::Single(10, 200);
        let two = PortMapping::Single(50, 200);
        assert!(!one.are_host_ports_equal(&two));
        assert!(!two.are_host_ports_equal(&one));
    }

    #[test]
    fn port_mapping_host_equal_range_range() {
        let one = PortMapping::Range {
            from: PortRange::new(100..=200),
            to: PortRange::new(400..=500),
        };
        let two = PortMapping::Range {
            from: PortRange::new(100..=200),
            to: PortRange::new(700..=800),
        };
        assert!(one.are_host_ports_equal(&two));
        assert!(two.are_host_ports_equal(&one));
        let one = PortMapping::Range {
            from: PortRange::new(10..=20),
            to: PortRange::new(80..=90),
        };
        let two = PortMapping::Range {
            from: PortRange::new(15..=25),
            to: PortRange::new(80..=90),
        };
        assert!(!one.are_host_ports_equal(&two));
        assert!(!two.are_host_ports_equal(&one));
    }

    #[test]
    fn port_mapping_host_equal_range_single() {
        let one = PortMapping::Range {
            from: PortRange::new(10..=10),
            to: PortRange::new(20..=20),
        };
        let two = PortMapping::Single(10, 200);
        assert!(one.are_host_ports_equal(&two));
        assert!(two.are_host_ports_equal(&one));
        let one = PortMapping::Range {
            from: PortRange::new(10..=100),
            to: PortRange::new(400..=490),
        };
        let two = PortMapping::Single(10, 200);
        assert!(!one.are_host_ports_equal(&two));
        assert!(!two.are_host_ports_equal(&one));
    }

    #[test]
    fn port_mapping_normalize() {
        let mapping = PortMapping::Range {
            from: PortRange::new(10..=20),
            to: PortRange::new(30..=40),
        };
        assert_eq!(mapping.clone().normalize(), mapping);
        let mapping = PortMapping::Single(10, 60);
        assert_eq!(mapping.clone().normalize(), mapping);
        let mapping = PortMapping::Range {
            from: PortRange::new(10..=10),
            to: PortRange::new(20..=20),
        };
        assert_eq!(mapping.normalize(), PortMapping::Single(10, 20));
    }
}
