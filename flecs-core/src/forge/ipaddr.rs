use std::net::{Ipv4Addr, Ipv6Addr};

pub trait BitComplementExt {
    fn complement(&self) -> Self;
}

impl BitComplementExt for Ipv4Addr {
    fn complement(&self) -> Self {
        Ipv4Addr::from(!u32::from(*self))
    }
}

impl BitComplementExt for Ipv6Addr {
    fn complement(&self) -> Self {
        Ipv6Addr::from(!u128::from(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;
    use std::str::FromStr;

    #[test_case("192.168.54.23", "63.87.201.232")]
    #[test_case("10.3.72.198", "245.252.183.57")]
    #[test_case("172.16.88.7", "83.239.167.248")]
    #[test_case("203.45.112.9", "52.210.143.246")]
    #[test_case("8.46.219.33", "247.209.36.222")]
    #[test_case("255.255.255.255", "0.0.0.0")]
    fn complement_ipv4(one: &str, expected: &str) {
        let one = Ipv4Addr::from_str(one).unwrap();
        let expected = Ipv4Addr::from_str(expected).unwrap();
        assert_eq!(one.complement(), expected);
        assert_eq!(expected.complement(), one);
    }

    #[test_case(
        "2001:0db8:85a3:0000:4f3b:9c2d:8a1e:7c44",
        "dffe:f247:7a5c:ffff:b0c4:63d2:75e1:83bb"
    )]
    #[test_case(
        "fe80:0000:0000:0000:5e2b:76ff:fe13:abcd",
        "017f:ffff:ffff:ffff:a1d4:8900:01ec:5432"
    )]
    #[test_case("fd12:3456:789a:1::1", "02ed:cba9:8765:fffe:ffff:ffff:ffff:fffe")]
    #[test_case("2607:f8b0:4005:809::200e", "d9f8:074f:bffa:f7f6:ffff:ffff:ffff:dff1")]
    #[test_case(
        "2a03:2880:2110:df07:face:b00c:0:1",
        "d5fc:d77f:deef:20f8:0531:4ff3:ffff:fffe"
    )]
    #[test_case("2001:470:1f0b:1234::2", "dffe:fb8f:e0f4:edcb:ffff:ffff:ffff:fffd")]
    fn complement_ipv6(one: &str, expected: &str) {
        let one = Ipv6Addr::from_str(one).unwrap();
        let expected = Ipv6Addr::from_str(expected).unwrap();
        assert_eq!(one.complement(), expected);
        assert_eq!(expected.complement(), one);
    }
}
