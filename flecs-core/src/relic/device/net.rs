#[cfg(test)]
use mockall::{automock, predicate::*};
use std::fs;

#[cfg(not(test))]
const NET_PATH: &str = "/sys/class/net/";
#[cfg(test)]
const NET_PATH: &str = "/tmp/flecs-tests/sys/class/net/";

#[cfg_attr(test, automock)]
pub trait NetDeviceReader: Sync + Send {
    fn get_net_property(&self, net_adapter: &str, property_name: &str) -> crate::Result<String>;
}

#[derive(Default)]
pub struct NetDeviceReaderImpl;
impl NetDeviceReader for NetDeviceReaderImpl {
    fn get_net_property(&self, net_adapter: &str, property_name: &str) -> anyhow::Result<String> {
        let path = format!("{NET_PATH}{net_adapter}/{property_name}");
        Ok(fs::read_to_string(path)?.trim_end().to_string())
    }
}

pub trait NetDeviceReaderExt {
    fn is_connected(&self, net_adapter: &str) -> bool;
}

impl<T: ?Sized + NetDeviceReader> NetDeviceReaderExt for T {
    fn is_connected(&self, net_adapter: &str) -> bool {
        matches!(self.get_net_property(net_adapter, "carrier"), Ok(value) if value == "1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn prepare_net_device_test_path(test_name: &str) -> PathBuf {
        let path = Path::new(NET_PATH).join(test_name);
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(&path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(&path).unwrap();
        assert!(path.try_exists().unwrap());
        path
    }

    #[test]
    fn net_device_reader_impl_read_property_ok() {
        let net_adapter = "net_device_reader_impl_read_property_ok";
        let path = prepare_net_device_test_path(net_adapter);
        fs::write(path.join("test_property"), b"some property value \t\n").unwrap();
        assert_eq!(
            NetDeviceReaderImpl
                .get_net_property(net_adapter, "test_property")
                .unwrap(),
            String::from("some property value")
        );
    }

    #[test]
    fn net_device_reader_impl_read_property_err() {
        let net_adapter = "net_device_reader_impl_read_property_err";
        prepare_net_device_test_path(net_adapter);
        assert!(NetDeviceReaderImpl
            .get_net_property(net_adapter, "test_property")
            .is_err());
    }

    #[test]
    fn net_device_reader_ext_is_connected_true() {
        let net_adapter = "TestAdapter";
        let mut mock_reader = MockNetDeviceReader::new();
        mock_reader
            .expect_get_net_property()
            .once()
            .with(eq(net_adapter), eq("carrier"))
            .returning(|_, _| Ok("1".to_owned()));
        assert!(mock_reader.is_connected(net_adapter));
    }

    #[test]
    fn net_device_reader_ext_is_connected_false() {
        let net_adapter = "TestAdapter";
        let mut mock_reader = MockNetDeviceReader::new();
        mock_reader
            .expect_get_net_property()
            .once()
            .with(eq(net_adapter), eq("carrier"))
            .returning(|_, _| Ok("0".to_owned()));
        assert!(!mock_reader.is_connected(net_adapter));
    }

    #[test]
    fn net_device_reader_ext_is_connected_err() {
        let net_adapter = "TestAdapter";
        let mut mock_reader = MockNetDeviceReader::new();
        mock_reader
            .expect_get_net_property()
            .once()
            .with(eq(net_adapter), eq("carrier"))
            .returning(|_, _| Err(anyhow::anyhow!("TestError")));
        assert!(!mock_reader.is_connected(net_adapter));
    }
}
