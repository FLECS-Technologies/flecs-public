use crate::relic::network::Ipv4Network;
use std::ffi::OsString;
use std::net::{AddrParseError, Ipv4Addr};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Environment variable {0} contains non unicode characters: {1:?}")]
    NotUnicode(&'static str, OsString),
    #[error("Invalid integer in {1} from {0}: {2}")]
    InvalidInteger(&'static str, String, ParseIntError),
    #[error("Invalid uri {1} from {0}: {2}")]
    InvalidUri(&'static str, String, http::uri::InvalidUri),
    #[error("Invalid network {1} from {0}: {2}")]
    InvalidIpv4Network(&'static str, String, anyhow::Error),
    #[error("Invalid ipv4 network {1} from {0}: {2}")]
    InvalidIpv4(&'static str, String, AddrParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
pub trait VarReader {
    fn read_os_var(&self, key: &'static str) -> Option<OsString>;
    fn read_var(&self, key: &'static str) -> Result<Option<String>> {
        match self.read_os_var(key) {
            None => Ok(None),
            Some(val) => Ok(Some(
                val.into_string()
                    .map_err(|val| Error::NotUnicode(key, val))?,
            )),
        }
    }
    fn read_path(&self, key: &'static str) -> Option<PathBuf> {
        self.read_os_var(key).map(PathBuf::from)
    }

    fn read_secs(&self, key: &'static str) -> Result<Option<Duration>> {
        self.read_var(key)?
            .map(|val| {
                u64::from_str(&val)
                    .map(Duration::from_secs)
                    .map_err(|e| Error::InvalidInteger(key, val, e))
            })
            .transpose()
    }

    fn read_uri(&self, key: &'static str) -> Result<Option<http::Uri>> {
        self.read_var(key)?
            .map(|val| http::Uri::from_str(&val).map_err(|e| Error::InvalidUri(key, val, e)))
            .transpose()
    }

    fn read_network(&self, key: &'static str) -> Result<Option<Ipv4Network>> {
        self.read_var(key)?
            .map(|val| {
                Ipv4Network::from_str(&val).map_err(|e| Error::InvalidIpv4Network(key, val, e))
            })
            .transpose()
    }

    fn read_ipv4(&self, key: &'static str) -> Result<Option<Ipv4Addr>> {
        self.read_var(key)?
            .map(|val| Ipv4Addr::from_str(&val).map_err(|e| Error::InvalidIpv4(key, val, e)))
            .transpose()
    }
}

pub struct EnvReader;

impl VarReader for EnvReader {
    fn read_os_var(&self, key: &'static str) -> Option<OsString> {
        std::env::var_os(key)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::collections::HashMap;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    #[derive(Default)]
    pub struct MockVarReader(HashMap<&'static str, OsString>);

    impl VarReader for MockVarReader {
        fn read_os_var(&self, key: &'static str) -> Option<OsString> {
            self.0.get(key).cloned()
        }
    }

    impl MockVarReader {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn from_var((key, value): (&'static str, &str)) -> Self {
            let mut data = Self::new();
            data.set_var(key, value);
            data
        }

        pub fn from_vars(vars: &[(&'static str, &str)]) -> Self {
            let mut data = Self::new();
            for (key, value) in vars {
                data.set_var(key, value);
            }
            data
        }

        pub fn remove_var(&mut self, key: &str) {
            self.0.remove(key);
        }

        pub fn set_var(&mut self, key: &'static str, value: &str) {
            self.0.insert(key, OsString::from(value));
        }
    }

    #[test]
    fn read_os_var_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "test value 123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(reader.read_os_var(KEY), Some(OsString::from(VALUE)));
    }

    #[test]
    fn read_os_var_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "test value 123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_os_var("OTHER_KEY").is_none());
    }

    #[test]
    fn read_var_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "test value 123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(reader.read_var(KEY).unwrap(), Some(String::from(VALUE)));
    }

    #[test]
    fn read_var_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "test value 123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_var("OTHER_KEY").unwrap().is_none());
    }

    #[test]
    fn read_var_err() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &[u8] = b"abc\xFFdef";
        let mut reader = MockVarReader::new();
        let value = OsStr::from_bytes(VALUE).to_os_string();
        reader.0.insert(KEY, value.clone());
        assert!(matches!(
            reader.read_var(KEY),
            Err(Error::NotUnicode(KEY, v)) if v == value
        ));
    }

    #[test]
    fn read_path_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "/some/path";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(reader.read_path(KEY), Some(PathBuf::from(VALUE)));
    }

    #[test]
    fn read_path_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "test value 123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_var("OTHER_KEY").unwrap().is_none());
    }

    #[test]
    fn read_secs_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(
            reader.read_secs(KEY).unwrap(),
            Some(Duration::from_secs(123))
        );
    }

    #[test]
    fn read_secs_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_secs("OTHER_KEY").unwrap().is_none());
    }

    #[test]
    fn read_secs_err() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "abc123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(matches!(
            reader.read_secs(KEY),
            Err(Error::InvalidInteger(KEY, v, ParseIntError{..})) if v == VALUE
        ));
    }

    #[test]
    fn read_uri_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "https://some.uri/123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(
            reader.read_uri(KEY).unwrap(),
            Some(http::Uri::from_str(VALUE).unwrap())
        );
    }

    #[test]
    fn read_uri_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "https://some.uri/123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_uri("OTHER_KEY").unwrap().is_none());
    }

    #[test]
    fn read_uri_err() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "httpp::///abc123";
        let reader = MockVarReader::from_var((KEY, VALUE));
        let result = reader.read_uri(KEY);
        assert!(
            matches!(
                result,
                Err(Error::InvalidUri(KEY, ref v, http::uri::InvalidUri{..})) if v == VALUE
            ),
            "{:?}",
            result
        );
    }

    #[test]
    fn read_network_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "123.123.0.0/16";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(
            reader.read_network(KEY).unwrap(),
            Some(Ipv4Network::try_new(Ipv4Addr::new(123, 123, 0, 0), 16).unwrap())
        );
    }

    #[test]
    fn read_network_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "123.123.0.0/16";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_network("OTHER_KEY").unwrap().is_none());
    }

    #[test]
    fn read_network_err() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "abc";
        let reader = MockVarReader::from_var((KEY, VALUE));
        let result = reader.read_network(KEY);
        assert!(
            matches!(
                result,
                Err(Error::InvalidIpv4Network(KEY, ref v, _)) if v == VALUE
            ),
            "{:?}",
            result
        );
    }

    #[test]
    fn read_ipv4_some() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "123.123.0.0";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert_eq!(
            reader.read_ipv4(KEY).unwrap(),
            Some(Ipv4Addr::new(123, 123, 0, 0))
        );
    }

    #[test]
    fn read_ipv4_none() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "123.123.0.0";
        let reader = MockVarReader::from_var((KEY, VALUE));
        assert!(reader.read_ipv4("OTHER_KEY").unwrap().is_none());
    }

    #[test]
    fn read_ipv4_err() {
        const KEY: &str = "TEST_KEY";
        const VALUE: &str = "abc";
        let reader = MockVarReader::from_var((KEY, VALUE));
        let result = reader.read_ipv4(KEY);
        assert!(
            matches!(
                result,
                Err(Error::InvalidIpv4(KEY, ref v, _)) if v == VALUE
            ),
            "{:?}",
            result
        );
    }
}
