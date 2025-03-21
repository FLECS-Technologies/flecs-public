use std::fmt::{Display, Formatter};
use std::net::Ipv6Addr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv6Network {
    pub address: Ipv6Addr,
    pub suffix: u8,
}

impl Display for Ipv6Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.address, self.suffix)
    }
}

pub fn ipv6_to_network(ip: Ipv6Addr, subnet_mask: Ipv6Addr) -> Ipv6Network {
    let address = ip & subnet_mask;
    let subnet_mask: u128 = subnet_mask.into();
    Ipv6Network {
        address,
        suffix: subnet_mask.count_ones() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    #[test]
    fn test_ipv6_to_network() {
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("2002:0000:0000:1234:abcd:ffff:c0a8:0101").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:ffff:ffff:0000:0000:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("2002:0000:0000:1234:0000:0000:0000:0000").unwrap(),
                suffix: 64
            }
        );
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("abcd:4422:efef:0707:8888:1212:3234:1256").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:0000:0000:0000:0000:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("abcd:4422:0000:0000:0000:0000:0000:0000").unwrap(),
                suffix: 32
            }
        );
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:fff0:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1110:0000:0000").unwrap(),
                suffix: 92
            }
        );
    }
}
