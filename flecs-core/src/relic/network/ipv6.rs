use std::fmt::{Display, Formatter};
use std::net::Ipv6Addr;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ipv6Network {
    pub address: Ipv6Addr,
    prefix_len: u8,
}

impl Ipv6Network {
    pub fn new(address: Ipv6Addr, prefix_len: u8) -> Self {
        Self {
            address,
            prefix_len,
        }
    }

    pub fn new_from_address_and_subnet_mask(address: Ipv6Addr, subnet_mask: Ipv6Addr) -> Self {
        let suffix = subnet_mask
            .octets()
            .iter()
            .fold(0, |acc, x| acc + x.count_ones());
        Self::new(address & subnet_mask, suffix as u8)
    }

    pub fn address(&self) -> &Ipv6Addr {
        &self.address
    }

    pub fn subnet_mask(&self) -> Ipv6Addr {
        Ipv6Addr::from(u128::MAX << (128 - self.prefix_len))
    }

    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }
}

impl FromStr for Ipv6Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (address, size) = s
            .split_once('/')
            .ok_or_else(|| anyhow::anyhow!("No '/' found"))?;
        Ok(Ipv6Network::new(
            Ipv6Addr::from_str(address)?,
            u8::from_str(size)?,
        ))
    }
}

impl Display for Ipv6Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

pub fn ipv6_to_network(ip: Ipv6Addr, subnet_mask: Ipv6Addr) -> Ipv6Network {
    let address = ip & subnet_mask;
    let subnet_mask: u128 = subnet_mask.into();
    Ipv6Network {
        address,
        prefix_len: subnet_mask.count_ones() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;
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
                prefix_len: 64
            }
        );
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("abcd:4422:efef:0707:8888:1212:3234:1256").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:0000:0000:0000:0000:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("abcd:4422:0000:0000:0000:0000:0000:0000").unwrap(),
                prefix_len: 32
            }
        );
        assert_eq!(
            ipv6_to_network(
                Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1111:2222:3333").unwrap(),
                Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:fff0:0000:0000").unwrap()
            ),
            Ipv6Network {
                address: Ipv6Addr::from_str("aaaa:bbbb:cccc:dddd:eeee:1110:0000:0000").unwrap(),
                prefix_len: 92
            }
        );
    }

    #[test_case("81f2:f385:4800::", 37, "81f2:f385:4800::/37")]
    #[test_case("86e5:6018:d00::", 44, "86e5:6018:d00::/44")]
    #[test_case("4761:45da:6::", 50, "4761:45da:6::/50")]
    #[test_case("b884:6129:db74:a800::", 53, "b884:6129:db74:a800::/53")]
    #[test_case("15a1:b1ac::", 33, "15a1:b1ac::/33")]
    #[test_case("3cf9:2cff::", 33, "3cf9:2cff::/33")]
    #[test_case("ffa4:aafb:9c26:3040::", 59, "ffa4:aafb:9c26:3040::/59")]
    #[test_case("b20d:a3e5:3857:b800::", 53, "b20d:a3e5:3857:b800::/53")]
    #[test_case("7519:f47a:9000::", 37, "7519:f47a:9000::/37")]
    #[test_case("d5f0:bf0f:7ec0::", 45, "d5f0:bf0f:7ec0::/45")]
    #[test_case("23f1:99b8:6000::", 35, "23f1:99b8:6000::/35")]
    #[test_case("97a7:922d:5ec0::", 46, "97a7:922d:5ec0::/46")]
    #[test_case("edd3:206b:1e8f:f6c0::", 58, "edd3:206b:1e8f:f6c0::/58")]
    #[test_case("e831:1727:7500::", 40, "e831:1727:7500::/40")]
    #[test_case("b4c9:b860:e45e:b500::", 57, "b4c9:b860:e45e:b500::/57")]
    #[test_case("71d4:f385:375a:e000::", 51, "71d4:f385:375a:e000::/51")]
    #[test_case("383e:da05:7800::", 39, "383e:da05:7800::/39")]
    #[test_case("9926:8e1a:47ee:c000::", 50, "9926:8e1a:47ee:c000::/50")]
    #[test_case("5abd:c7f5:e300::", 43, "5abd:c7f5:e300::/43")]
    fn ipv6_network_from_str_ok(ip: &str, prefix_len: u8, s: &str) {
        let expected = Ipv6Network::new(Ipv6Addr::from_str(ip).unwrap(), prefix_len);
        assert_eq!(Ipv6Network::from_str(s).unwrap(), expected);
    }

    #[test_case("5abd:c7f5:e300:://43")]
    #[test_case("5abd:c7ff5:e300::/4")]
    #[test_case("5abd:c7x5:e300::/4")]
    #[test_case("5abd:c7f5:e300::/438")]
    #[test_case("5abd:c7f5:e300::43")]
    fn ipv6_network_from_str_err(s: &str) {
        assert!(Ipv6Network::from_str(s).is_err());
    }

    #[test_case("81f2:f385:4800::", 37, "ffff:ffff:f800::")]
    #[test_case("86e5:6018:d00::", 44, "ffff:ffff:fff0::")]
    #[test_case("4761:45da:6::", 50, "ffff:ffff:ffff:c000::")]
    #[test_case("b884:6129:db74:a800::", 53, "ffff:ffff:ffff:f800::")]
    #[test_case("15a1:b1ac::", 33, "ffff:ffff:8000::")]
    #[test_case("3cf9:2cff::", 33, "ffff:ffff:8000::")]
    #[test_case("ffa4:aafb:9c26:3040::", 59, "ffff:ffff:ffff:ffe0::")]
    #[test_case("b20d:a3e5:3857:b800::", 53, "ffff:ffff:ffff:f800::")]
    #[test_case("7519:f47a:9000::", 37, "ffff:ffff:f800::")]
    #[test_case("d5f0:bf0f:7ec0::", 45, "ffff:ffff:fff8::")]
    #[test_case("23f1:99b8:6000::", 35, "ffff:ffff:e000::")]
    #[test_case("97a7:922d:5ec0::", 46, "ffff:ffff:fffc::")]
    #[test_case("edd3:206b:1e8f:f6c0::", 58, "ffff:ffff:ffff:ffc0::")]
    #[test_case("e831:1727:7500::", 40, "ffff:ffff:ff00::")]
    #[test_case("b4c9:b860:e45e:b500::", 57, "ffff:ffff:ffff:ff80::")]
    #[test_case("71d4:f385:375a:e000::", 51, "ffff:ffff:ffff:e000::")]
    #[test_case("383e:da05:7800::", 39, "ffff:ffff:fe00::")]
    #[test_case("9926:8e1a:47ee:c000::", 50, "ffff:ffff:ffff:c000::")]
    #[test_case("5abd:c7f5:e300::", 43, "ffff:ffff:ffe0::")]
    fn ipv6_getter(ip: &str, prefix_len: u8, subnet_mask: &str) {
        let network = Ipv6Network::new(Ipv6Addr::from_str(ip).unwrap(), prefix_len);
        assert_eq!(*network.address(), Ipv6Addr::from_str(ip).unwrap());
        assert_eq!(network.prefix_len(), prefix_len);
        assert_eq!(
            network.subnet_mask(),
            Ipv6Addr::from_str(subnet_mask).unwrap()
        );
    }

    #[test_case("81f2:f385:4800::", 37, "ffff:ffff:f800::")]
    #[test_case("86e5:6018:d00::", 44, "ffff:ffff:fff0::")]
    #[test_case("4761:45da:6::", 50, "ffff:ffff:ffff:c000::")]
    #[test_case("b884:6129:db74:a800::", 53, "ffff:ffff:ffff:f800::")]
    #[test_case("15a1:b1ac::", 33, "ffff:ffff:8000::")]
    #[test_case("3cf9:2cff::", 33, "ffff:ffff:8000::")]
    #[test_case("ffa4:aafb:9c26:3040::", 59, "ffff:ffff:ffff:ffe0::")]
    #[test_case("b20d:a3e5:3857:b800::", 53, "ffff:ffff:ffff:f800::")]
    #[test_case("7519:f47a:9000::", 37, "ffff:ffff:f800::")]
    #[test_case("d5f0:bf0f:7ec0::", 45, "ffff:ffff:fff8::")]
    #[test_case("23f1:99b8:6000::", 35, "ffff:ffff:e000::")]
    #[test_case("97a7:922d:5ec0::", 46, "ffff:ffff:fffc::")]
    #[test_case("edd3:206b:1e8f:f6c0::", 58, "ffff:ffff:ffff:ffc0::")]
    #[test_case("e831:1727:7500::", 40, "ffff:ffff:ff00::")]
    #[test_case("b4c9:b860:e45e:b500::", 57, "ffff:ffff:ffff:ff80::")]
    #[test_case("71d4:f385:375a:e000::", 51, "ffff:ffff:ffff:e000::")]
    #[test_case("383e:da05:7800::", 39, "ffff:ffff:fe00::")]
    #[test_case("9926:8e1a:47ee:c000::", 50, "ffff:ffff:ffff:c000::")]
    #[test_case("5abd:c7f5:e300::", 43, "ffff:ffff:ffe0::")]
    fn ipv6_from_address_and_subnet_mask(ip: &str, prefix_len: u8, subnet_mask: &str) {
        let expected = Ipv6Network::new(Ipv6Addr::from_str(ip).unwrap(), prefix_len);
        let ip = Ipv6Addr::from_str(ip).unwrap();
        let subnet_mask = Ipv6Addr::from_str(subnet_mask).unwrap();
        assert_eq!(
            Ipv6Network::new_from_address_and_subnet_mask(ip, subnet_mask),
            expected
        );
    }
}
