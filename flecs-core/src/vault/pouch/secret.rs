use super::Result;
use super::{combine_results, Pouch, VaultPouch};
use flecs_console_client::models::SessionId;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Secrets {
    pub license_key: Option<String>,
    pub session_id: SessionId,
}

const SESSION_FILE_NAME: &str = ".session_id";
const LICENSE_FILE_NAME: &str = ".license";
pub struct SecretPouch {
    secrets: Secrets,
    path: PathBuf,
}

impl Pouch for SecretPouch {
    type Gems = Secrets;

    fn gems(&self) -> &Self::Gems {
        &self.secrets
    }

    fn gems_mut(&mut self) -> &mut Self::Gems {
        &mut self.secrets
    }
}

impl VaultPouch for SecretPouch {
    fn close(&mut self) -> crate::vault::Result<()> {
        fs::create_dir_all(&self.path)?;
        combine_results(self.save_session(), self.save_license())
    }

    fn open(&mut self) -> crate::vault::Result<()> {
        combine_results(self.read_session(), self.read_license())
    }
}

impl SecretPouch {
    pub fn new(path: &Path) -> Self {
        Self {
            secrets: Secrets::default(),
            path: path.to_path_buf(),
        }
    }
    fn save_license(&self) -> crate::vault::Result<()> {
        fs::write(
            self.path.join(LICENSE_FILE_NAME),
            self.secrets.license_key.as_ref().unwrap_or(&String::new()),
        )?;
        Ok(())
    }

    fn save_session(&self) -> crate::vault::Result<()> {
        let content = match &self.secrets.session_id {
            SessionId {
                id: Some(ref id),
                timestamp: Some(ref timestamp),
            } => format!("{id}\n{timestamp}"),
            SessionId {
                id: Some(ref id),
                timestamp: None,
            } => id.clone(),
            _ => String::new(),
        }
        .to_string();
        fs::write(self.path.join(SESSION_FILE_NAME), content)?;
        Ok(())
    }
    fn read_session(&mut self) -> Result<()> {
        let session_file = fs::read_to_string(self.path.join(SESSION_FILE_NAME))?;
        let mut lines = session_file.lines();
        self.secrets.session_id.id = lines.next().map(str::to_string);
        self.secrets.session_id.timestamp = lines.next().and_then(|s| s.parse().ok());
        Ok(())
    }
    fn read_license(&mut self) -> Result<()> {
        let license_file = fs::read_to_string(self.path.join(LICENSE_FILE_NAME))?;
        self.secrets.license_key = license_file.lines().next().map(str::to_string);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_PATH: &str = "/tmp/flecs-tests/pouch";

    #[test]
    fn open_complete_secret_pouch() {
        let test_path = Path::new(TEST_PATH).join("open_complete_secret_pouch");
        fs::create_dir_all(&test_path).unwrap();
        let id = "510292de-e745-4dd0-bf04-75630ba52187";
        let timestamp = 1723534992;
        let license = "1234-ABCD-EFGH-5678-IJKL";
        fs::write(
            test_path.join(SESSION_FILE_NAME),
            format!("{id}\n{timestamp}"),
        )
        .unwrap();
        fs::write(test_path.join(LICENSE_FILE_NAME), license).unwrap();
        let mut secrets = SecretPouch::new(&test_path);
        secrets.open().unwrap();
        assert_eq!(secrets.secrets.session_id.timestamp, Some(timestamp));
        assert_eq!(secrets.secrets.session_id.id, Some(id.to_string()));
        assert_eq!(secrets.secrets.license_key, Some(license.to_string()));
    }

    #[test]
    fn close_complete_secret_pouch() {
        let test_path = Path::new(TEST_PATH).join("close_complete_secret_pouch");
        fs::create_dir_all(&test_path).unwrap();
        let id = "510292de-e745-4dd0-bf04-75630ba52187";
        let timestamp = 1723534992;
        let license = "1234-ABCD-EFGH-5678-IJKL";
        if test_path.join(SESSION_FILE_NAME).exists() {
            fs::remove_file(test_path.join(SESSION_FILE_NAME)).unwrap();
        }
        if test_path.join(LICENSE_FILE_NAME).exists() {
            fs::remove_file(test_path.join(LICENSE_FILE_NAME)).unwrap();
        }
        let mut secrets = SecretPouch {
            path: test_path.to_path_buf(),
            secrets: Secrets {
                license_key: Some(license.to_string()),
                session_id: SessionId {
                    id: Some(id.to_string()),
                    timestamp: Some(timestamp),
                },
            },
        };
        secrets.close().unwrap();
        assert_eq!(
            fs::read_to_string(test_path.join(LICENSE_FILE_NAME)).unwrap(),
            license.to_string()
        );
        assert_eq!(
            fs::read_to_string(test_path.join(SESSION_FILE_NAME)).unwrap(),
            format!("{id}\n{timestamp}")
        );
    }

    #[test]
    fn open_secret_pouch_without_timestamp() {
        let test_path = Path::new(TEST_PATH).join("open_secret_pouch_without_timestamp");
        fs::create_dir_all(&test_path).unwrap();
        let id = "510292de-e745-4dd0-bf04-75630ba52187";
        let license = "1234-ABCD-EFGH-5678-IJKL";
        fs::write(test_path.join(SESSION_FILE_NAME), id).unwrap();
        fs::write(test_path.join(LICENSE_FILE_NAME), license).unwrap();
        let mut secrets = SecretPouch::new(&test_path);
        secrets.open().unwrap();
        assert!(secrets.secrets.session_id.timestamp.is_none());
        assert_eq!(secrets.secrets.session_id.id, Some(id.to_string()));
        assert_eq!(secrets.secrets.license_key, Some(license.to_string()));
    }

    #[test]
    fn close_secret_pouch_without_timestamp() {
        let test_path = Path::new(TEST_PATH).join("close_secret_pouch_without_timestamp");
        fs::create_dir_all(&test_path).unwrap();
        let id = "510292de-e745-4dd0-bf04-75630ba52187";
        let license = "1234-ABCD-EFGH-5678-IJKL";
        if test_path.join(SESSION_FILE_NAME).exists() {
            fs::remove_file(test_path.join(SESSION_FILE_NAME)).unwrap();
        }
        if test_path.join(LICENSE_FILE_NAME).exists() {
            fs::remove_file(test_path.join(LICENSE_FILE_NAME)).unwrap();
        }
        let mut secrets = SecretPouch {
            path: test_path.to_path_buf(),
            secrets: Secrets {
                license_key: Some(license.to_string()),
                session_id: SessionId {
                    id: Some(id.to_string()),
                    timestamp: None,
                },
            },
        };
        secrets.close().unwrap();
        assert_eq!(
            fs::read_to_string(test_path.join(LICENSE_FILE_NAME)).unwrap(),
            license.to_string()
        );
        assert_eq!(
            fs::read_to_string(test_path.join(SESSION_FILE_NAME)).unwrap(),
            format!("{id}")
        );
    }

    #[test]
    fn open_empty_secret_pouch() {
        let test_path = Path::new(TEST_PATH).join("open_empty_secret_pouch");
        fs::create_dir_all(&test_path).unwrap();
        if test_path.join(SESSION_FILE_NAME).exists() {
            fs::remove_file(test_path.join(SESSION_FILE_NAME)).unwrap();
        }
        if test_path.join(LICENSE_FILE_NAME).exists() {
            fs::remove_file(test_path.join(LICENSE_FILE_NAME)).unwrap();
        }
        let mut secrets = SecretPouch::new(&test_path);
        assert!(secrets.open().is_err());
        assert!(secrets.secrets.session_id.id.is_none());
        assert!(secrets.secrets.session_id.timestamp.is_none());
        assert!(secrets.secrets.license_key.is_none());
    }

    #[test]
    fn close_empty_secret_pouch() {
        let test_path = Path::new(TEST_PATH).join("close_empty_secret_pouch");
        fs::create_dir_all(&test_path).unwrap();
        if test_path.join(SESSION_FILE_NAME).exists() {
            fs::remove_file(test_path.join(SESSION_FILE_NAME)).unwrap();
        }
        if test_path.join(LICENSE_FILE_NAME).exists() {
            fs::remove_file(test_path.join(LICENSE_FILE_NAME)).unwrap();
        }
        let mut secrets = SecretPouch {
            path: test_path.to_path_buf(),
            secrets: Secrets {
                license_key: None,
                session_id: SessionId {
                    id: None,
                    timestamp: None,
                },
            },
        };
        secrets.close().unwrap();
        assert!(fs::read_to_string(test_path.join(LICENSE_FILE_NAME))
            .unwrap()
            .is_empty());
        assert!(fs::read_to_string(test_path.join(SESSION_FILE_NAME))
            .unwrap()
            .is_empty());
    }
}
