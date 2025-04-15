use super::Result;
use super::{combine_results, Pouch};
use flecs_console_client::models::SessionId;
use flecsd_axum_server::models::AuthResponseData;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Secrets {
    pub license_key: Option<String>,
    session_id: SessionId,
    pub authentication: Option<AuthResponseData>,
}

impl Secrets {
    pub fn get_session_id(&self) -> SessionId {
        self.session_id.clone()
    }
    pub fn set_session_id(&mut self, session_id: SessionId) {
        match (
            &self.session_id.id,
            session_id.timestamp,
            &self.session_id.timestamp,
        ) {
            (.., None) | (None, ..) => self.session_id = session_id,
            (Some(_), Some(new), Some(current)) if new >= *current => self.session_id = session_id,
            _ => {}
        }
    }

    pub fn new(
        license_key: Option<String>,
        session_id: SessionId,
        authentication: Option<AuthResponseData>,
    ) -> Self {
        Self {
            license_key,
            session_id,
            authentication,
        }
    }
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

impl SecretPouch {
    pub(in super::super) fn close(&mut self) -> crate::vault::Result<()> {
        fs::create_dir_all(&self.path)?;
        combine_results(self.save_session(), self.save_license())
    }

    pub(in super::super) fn open(&mut self) -> crate::vault::Result<()> {
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
                id: Some(id),
                timestamp: Some(timestamp),
            } => format!("{id}\n{timestamp}"),
            SessionId {
                id: Some(id),
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
pub mod tests {
    use super::*;
    use testdir::testdir;
    const TEST_PATH: &str = "/tmp/flecs-tests/pouch";

    pub fn test_secret_pouch() -> SecretPouch {
        SecretPouch {
            path: testdir!().join("secrets"),
            secrets: Secrets {
                license_key: None,
                session_id: Default::default(),
                authentication: None,
            },
        }
    }

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
        assert!(secrets.secrets.authentication.is_none());
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
                authentication: None,
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
        assert!(secrets.secrets.authentication.is_none());
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
                authentication: None,
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
        assert_eq!(secrets.secrets.authentication, None);
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
        assert!(secrets.secrets.authentication.is_none());
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
                authentication: None,
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

    #[test]
    fn set_session_id_newer() {
        let current = SessionId {
            id: Some("51ff9015-1d6e-4b3a-a3e0-a51ee7d5b4f3".to_string()),
            timestamp: Some(1724671774876u64),
        };
        let next = SessionId {
            id: Some("d22cdf2b-abc4-4cc1-bc01-7cbd70b5b10f".to_string()),
            timestamp: Some(1724671774900u64),
        };
        let mut secrets = Secrets {
            session_id: current.clone(),
            license_key: None,
            authentication: None,
        };
        secrets.set_session_id(next.clone());
        assert_eq!(secrets.session_id, next);
    }

    #[test]
    fn set_session_id_older() {
        let current = SessionId {
            id: Some("51ff9015-1d6e-4b3a-a3e0-a51ee7d5b4f3".to_string()),
            timestamp: Some(1724671774876u64),
        };
        let next = SessionId {
            id: Some("d22cdf2b-abc4-4cc1-bc01-7cbd70b5b10f".to_string()),
            timestamp: Some(1724671774000u64),
        };
        let mut secrets = Secrets {
            session_id: current.clone(),
            license_key: None,
            authentication: None,
        };
        secrets.set_session_id(next.clone());
        assert_eq!(secrets.session_id, current);
    }

    #[test]
    fn set_session_id_same_age() {
        let current = SessionId {
            id: Some("51ff9015-1d6e-4b3a-a3e0-a51ee7d5b4f3".to_string()),
            timestamp: Some(1724671774876u64),
        };
        let next = SessionId {
            id: Some("d22cdf2b-abc4-4cc1-bc01-7cbd70b5b10f".to_string()),
            timestamp: Some(1724671774876u64),
        };
        let mut secrets = Secrets {
            session_id: current.clone(),
            license_key: None,
            authentication: None,
        };
        secrets.set_session_id(next.clone());
        assert_eq!(secrets.session_id, next);
    }

    #[test]
    fn set_session_id_empty() {
        let current = SessionId {
            id: None,
            timestamp: Some(1724671774900u64),
        };
        let next = SessionId {
            id: Some("d22cdf2b-abc4-4cc1-bc01-7cbd70b5b10f".to_string()),
            timestamp: Some(1724671774876u64),
        };
        let mut secrets = Secrets {
            session_id: current.clone(),
            license_key: None,
            authentication: None,
        };
        secrets.set_session_id(next.clone());
        assert_eq!(secrets.session_id, next);
    }

    #[test]
    fn set_session_id_no_time() {
        let current = SessionId {
            id: Some("51ff9015-1d6e-4b3a-a3e0-a51ee7d5b4f3".to_string()),
            timestamp: None,
        };
        let next = SessionId {
            id: Some("d22cdf2b-abc4-4cc1-bc01-7cbd70b5b10f".to_string()),
            timestamp: Some(1724671774876u64),
        };
        let mut secrets = Secrets {
            session_id: current.clone(),
            license_key: None,
            authentication: None,
        };
        secrets.set_session_id(next.clone());
        assert_eq!(secrets.session_id, next);
    }
}
