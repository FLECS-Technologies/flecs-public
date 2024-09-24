pub mod pouch;

use crate::vault::pouch::{Pouch, Secrets};
use pouch::{AppPouch, ManifestPouch, SecretPouch, VaultPouch};
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub enum Error {
    Single(String),
    Multiple(Vec<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Single(s) => write!(f, "{s}"),
            Error::Multiple(m) => write!(f, "{}", m.join("\n")),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

pub type Result<T> = std::result::Result<T, Error>;
impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(value: T) -> Self {
        Self::Single(value.to_string())
    }
}

impl Error {
    pub fn append(self, other: Self) -> Self {
        match (self, other) {
            (Self::Single(s1), Self::Single(s2)) => Self::Multiple(vec![s1, s2]),
            (Self::Single(s), Self::Multiple(mut m)) | (Self::Multiple(mut m), Self::Single(s)) => {
                m.push(s);
                Self::Multiple(m)
            }
            (Self::Multiple(mut m1), Self::Multiple(mut m2)) => {
                m1.append(&mut m2);
                Self::Multiple(m1)
            }
        }
    }

    pub fn from_strings(strings: Vec<String>) -> Self {
        Self::Multiple(strings)
    }
}

/// Contains all the information for constructing a [Vault].
/// # Examples
/// ```
/// use flecs_core::vault::VaultConfig;
/// use std::path::PathBuf;
///
/// let config = VaultConfig {
///     path: PathBuf::from("/flecs-tests/vault/"),
/// };
/// ```
pub struct VaultConfig {
    pub path: PathBuf,
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            path: Path::new(crate::lore::BASE_PATH).to_path_buf(),
        }
    }
}

/// A [Vault] contains data that corresponds to files on disk. It is split into multiple pouches
/// which contain different kinds of information. A [ManifestPouch] for example contains app
/// manifests.
/// For accessing the content of a [Vault] see [Vault::reservation()].
pub struct Vault {
    app_pouch: RwLock<AppPouch>,
    manifest_pouch: RwLock<ManifestPouch>,
    secret_pouch: RwLock<SecretPouch>,
}

impl Vault {
    /// Creates a new [Vault] from a given [VaultConfig]. If data should be read from disk, a
    /// separate call to [Vault::open()] is necessary.
    /// # Example
    /// ```
    /// use flecs_core::vault::{Vault, VaultConfig};
    /// use std::path::PathBuf;
    ///
    /// let config = VaultConfig {
    ///     path: PathBuf::from("/flecs-tests/vault/"),
    /// };
    /// let vault = Vault::new(config);
    /// vault.open();
    /// ```
    pub fn new(config: VaultConfig) -> Self {
        Self {
            app_pouch: RwLock::new(AppPouch::new(&config.path.join("apps"))),
            manifest_pouch: RwLock::new(ManifestPouch::new(&config.path.join("manifests"))),
            secret_pouch: RwLock::new(SecretPouch::new(&config.path.join("device"))),
        }
    }

    /// Creates an empty [Reservation] for this [Vault], which can be used to reserve and access
    /// its Pouches. See [Reservation::grab()] for details.
    /// # Examples
    /// ```
    /// use flecs_core::vault::{Vault, VaultConfig};
    /// use std::path::Path;
    ///
    /// # tokio_test::block_on(
    /// async {
    ///     let vault = Vault::new(VaultConfig {
    ///         path: Path::new("/tmp/vault/").to_path_buf(),
    ///     });
    ///     let reservation = vault
    ///         .reservation()
    ///         .reserve_app_pouch()
    ///         .reserve_manifest_pouch()
    ///         .reserve_secret_pouch_mut();
    ///     let pouches = reservation.grab().await;
    ///     assert!(pouches.app_pouch.is_some());
    ///     assert!(pouches.app_pouch_mut.is_none());
    ///     assert!(pouches.manifest_pouch.is_some());
    ///     assert!(pouches.manifest_pouch_mut.is_none());
    ///     assert!(pouches.secret_pouch.is_none());
    ///     assert!(pouches.secret_pouch_mut.is_some());
    /// }
    /// # )
    /// ```
    /// More concise variant
    /// ```
    /// use flecs_core::vault::{GrabbedPouches, Vault, VaultConfig};
    /// use std::path::Path;
    ///
    /// # tokio_test::block_on(
    /// async {
    ///     let vault = Vault::new(VaultConfig {
    ///         path: Path::new("/tmp/vault/").to_path_buf(),
    ///     });
    ///     if let GrabbedPouches {
    ///         app_pouch: Some(apps),
    ///         manifest_pouch: Some(manifests),
    ///         secret_pouch_mut: Some(secrets),
    ///         ..
    ///     } = &vault
    ///         .reservation()
    ///         .reserve_app_pouch()
    ///         .reserve_manifest_pouch()
    ///         .reserve_secret_pouch_mut()
    ///         .grab()
    ///         .await
    ///     {
    ///         // use pouches
    ///     } else {
    ///         panic!("This branch is unreachable if the correct pouches are reserved and matched");
    ///     };
    /// }
    /// # )
    /// ```
    pub fn reservation(&self) -> Reservation {
        Reservation::new(self)
    }

    /// Replaces the content of all pouches with data from disk. See [AppPouch::open()],
    /// [ManifestPouch::open()] and [SecretPouch::open()] for details.
    pub async fn open(&self) {
        let mut grabbed_pouches = self
            .reservation()
            .reserve_app_pouch_mut()
            .reserve_manifest_pouch_mut()
            .reserve_secret_pouch_mut()
            .grab()
            .await;
        if let GrabbedPouches {
            app_pouch_mut: Some(ref mut app_pouch_mut),
            manifest_pouch_mut: Some(ref mut manifest_pouch_mut),
            secret_pouch_mut: Some(ref mut secret_pouch_mut),
            ..
        } = grabbed_pouches
        {
            app_pouch_mut
                .open()
                .unwrap_or_else(|e| eprintln!("Could not open AppPouch: {e}"));
            secret_pouch_mut
                .open()
                .unwrap_or_else(|e| eprintln!("Could not open SecretPouch: {e}"));
            manifest_pouch_mut
                .open()
                .unwrap_or_else(|e| eprintln!("Could not open ManifestPouch: {e}"));
        } else {
            panic!("Could not reserve pouches for filling");
        }
    }

    /// Saves the content of all contained pouches. Calling this function is generally not necessary
    /// as the pouches are implicitly saved after accessing them mutably via [Self::reservation()].
    pub async fn close(&self) {
        // Dropping 'GrabbedPouches' closes all contained pouches
        let _ = self
            .reservation()
            .reserve_app_pouch_mut()
            .reserve_manifest_pouch_mut()
            .reserve_secret_pouch_mut()
            .grab()
            .await;
    }

    /// Creates a copy of the [Secrets] contained in the [SecretPouch] of this vault.
    /// <div class="warning">Do not use this function if any other pouches of the vault are needed.
    /// Using this method while access to the secret pouch was granted via reservation will lead to
    /// a deadlock!</div>
    pub async fn get_secrets(&self) -> Secrets {
        self.reservation()
            .reserve_secret_pouch()
            .grab()
            .await
            .secret_pouch
            .as_ref()
            .unwrap()
            .gems()
            .clone()
    }
}

enum ReserveKind {
    None,
    Read,
    Write,
}

/// Contains information which pouches a user wants to reserve for read-only or read and write
/// purposes. This struct is created by calling [Vault::reservation()] on a [Vault]. See
/// [Vault::reservation()] for usage examples.
pub struct Reservation<'a> {
    vault: &'a Vault,
    app_pouch_reserved: ReserveKind,
    manifest_pouch_reserved: ReserveKind,
    secret_pouch_reserved: ReserveKind,
}

/// Contains references to pouches behind RwLockGuards for thread-safe access. This struct is
/// created by calling [Reservation::grab()] on a [Reservation]. See [Vault::reservation()] for
/// usage examples.
pub struct GrabbedPouches<'a> {
    pub app_pouch: Option<RwLockReadGuard<'a, AppPouch>>,
    pub secret_pouch: Option<RwLockReadGuard<'a, SecretPouch>>,
    pub manifest_pouch: Option<RwLockReadGuard<'a, ManifestPouch>>,
    pub app_pouch_mut: Option<RwLockWriteGuard<'a, AppPouch>>,
    pub secret_pouch_mut: Option<RwLockWriteGuard<'a, SecretPouch>>,
    pub manifest_pouch_mut: Option<RwLockWriteGuard<'a, ManifestPouch>>,
}

impl<'a> Reservation<'a> {
    fn new(vault: &'a Vault) -> Self {
        Self {
            vault,
            app_pouch_reserved: ReserveKind::None,
            manifest_pouch_reserved: ReserveKind::None,
            secret_pouch_reserved: ReserveKind::None,
        }
    }

    /// Marks the app pouch as immutably reserved. See [Vault::reservation()] for general usage
    /// examples. Calling [Self::reserve_app_pouch()] overwrites the reservation as mutable.
    pub fn reserve_app_pouch(mut self) -> Self {
        self.app_pouch_reserved = ReserveKind::Read;
        self
    }

    /// Marks the secret pouch as immutably reserved. See [Vault::reservation()] for general usage
    /// examples. Calling [Self::reserve_secret_pouch_mut()] overwrites the reservation as mutable.
    pub fn reserve_secret_pouch(mut self) -> Self {
        self.secret_pouch_reserved = ReserveKind::Read;
        self
    }

    /// Marks the manifest pouch as immutably reserved. See [Vault::reservation()] for general usage
    /// examples. Calling [Self::reserve_manifest_pouch_mut()] overwrites the reservation as mutable.
    pub fn reserve_manifest_pouch(mut self) -> Self {
        self.manifest_pouch_reserved = ReserveKind::Read;
        self
    }

    /// Marks the app pouch as mutably reserved. See [Vault::reservation()] for general usage
    /// examples. Calling [Self::reserve_app_pouch()] overwrites the reservation as immutable.
    pub fn reserve_app_pouch_mut(mut self) -> Self {
        self.app_pouch_reserved = ReserveKind::Write;
        self
    }

    /// Marks the secret pouch as mutably reserved. See [Vault::reservation()] for general usage
    /// examples. Calling [Self::reserve_secret_pouch()] overwrites the reservation as immutable.
    pub fn reserve_secret_pouch_mut(mut self) -> Self {
        self.secret_pouch_reserved = ReserveKind::Write;
        self
    }

    /// Marks the manifest pouch as mutably reserved. See [Vault::reservation()] for general usage
    /// examples. Calling [Self::reserve_manifest_pouch()] overwrites the reservation as immutable.
    pub fn reserve_manifest_pouch_mut(mut self) -> Self {
        self.manifest_pouch_reserved = ReserveKind::Write;
        self
    }

    async fn create_reservation_guards<T>(
        reserve_kind: ReserveKind,
        lock: &RwLock<T>,
    ) -> (Option<RwLockReadGuard<T>>, Option<RwLockWriteGuard<T>>) {
        match reserve_kind {
            ReserveKind::None => (None, None),
            ReserveKind::Read => (Some(lock.read().await), None),
            ReserveKind::Write => (None, Some(lock.write().await)),
        }
    }

    /// Converts the [Reservation] into [GrabbedPouches] which allows accessing the previously
    /// reserved pouches. This function blocks until all reserved pouches are available.
    /// For usage examples see [Vault::reservation()].
    pub async fn grab(self) -> GrabbedPouches<'a> {
        let (app_pouch, app_pouch_mut) =
            Self::create_reservation_guards(self.app_pouch_reserved, &self.vault.app_pouch).await;
        let (secret_pouch, secret_pouch_mut) =
            Self::create_reservation_guards(self.secret_pouch_reserved, &self.vault.secret_pouch)
                .await;
        let (manifest_pouch, manifest_pouch_mut) = Self::create_reservation_guards(
            self.manifest_pouch_reserved,
            &self.vault.manifest_pouch,
        )
        .await;
        GrabbedPouches {
            app_pouch,
            secret_pouch,
            manifest_pouch,
            app_pouch_mut,
            secret_pouch_mut,
            manifest_pouch_mut,
        }
    }
}

impl Drop for GrabbedPouches<'_> {
    fn drop(&mut self) {
        // TODO: Close manifest and app, if the C++ core does not access the corresponding files anymore
        if let Some(secret_pouch) = &mut self.secret_pouch_mut {
            secret_pouch
                .close()
                .unwrap_or_else(|e| eprintln!("Error saving SecretPouch: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flecs_console_client::models::SessionId;
    use ntest::timeout;

    #[tokio::test]
    #[timeout(100)]
    async fn grab_multiple() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let grab = vault
            .reservation()
            .reserve_manifest_pouch()
            .reserve_app_pouch()
            .reserve_secret_pouch()
            .grab()
            .await;
        assert!(grab.manifest_pouch.is_some());
        assert!(grab.app_pouch.is_some());
        assert!(grab.secret_pouch.is_some());
        assert!(grab.manifest_pouch_mut.is_none());
        assert!(grab.app_pouch_mut.is_none());
        assert!(grab.secret_pouch_mut.is_none());
    }

    #[tokio::test]
    #[timeout(100)]
    #[should_panic]
    async fn double_grab_mutable_mutable() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let _grab1 = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let _grab2 = vault.reservation().reserve_secret_pouch_mut().grab().await;
    }

    #[tokio::test]
    #[timeout(100)]
    #[should_panic]
    async fn double_grab_mutable_immutable() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let _grab1 = vault.reservation().reserve_secret_pouch_mut().grab().await;
        let _grab2 = vault.reservation().reserve_secret_pouch().grab().await;
    }

    #[tokio::test]
    #[timeout(100)]
    async fn double_grab_immutable_immutable() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let grab1 = vault.reservation().reserve_secret_pouch().grab().await;
        assert!(grab1.secret_pouch.is_some());
        let grab2 = vault.reservation().reserve_secret_pouch().grab().await;
        assert!(grab2.secret_pouch.is_some());
    }

    #[tokio::test]
    #[timeout(100)]
    #[should_panic]
    async fn double_grab_immutable_mutable() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let _grab1 = vault.reservation().reserve_secret_pouch().grab().await;
        let _grab2 = vault.reservation().reserve_secret_pouch_mut().grab().await;
    }

    #[tokio::test]
    #[timeout(100)]
    async fn reserving_one_pouch_leaves_other_pouches_mut() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let grab_secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
        assert!(grab_secrets.secret_pouch_mut.is_some());
        let grab_apps = vault.reservation().reserve_app_pouch_mut().grab().await;
        assert!(grab_apps.app_pouch_mut.is_some());
        let grab_manifests = vault
            .reservation()
            .reserve_manifest_pouch_mut()
            .grab()
            .await;
        assert!(grab_manifests.manifest_pouch_mut.is_some());
    }

    #[tokio::test]
    #[timeout(100)]
    async fn get_secrets() {
        let vault = Vault::new(VaultConfig {
            path: Path::new("/tmp/flecs-tests/vault/").to_path_buf(),
        });
        let expected_secrets = Secrets::new(
            Some("9876-TZUI-VBNM-4567".to_string()),
            SessionId {
                id: Some("80ef3f12-2334-47a8-a4cd-4a8a048f9040".to_string()),
                timestamp: Some(1724662594123u64),
            },
            None,
        );
        {
            let mut grab_secrets = vault.reservation().reserve_secret_pouch_mut().grab().await;
            let secrets = grab_secrets.secret_pouch_mut.as_mut().unwrap().gems_mut();
            secrets.set_session_id(expected_secrets.get_session_id());
            secrets.authentication = expected_secrets.authentication.clone();
            secrets.license_key = expected_secrets.license_key.clone();
        }
        assert_eq!(expected_secrets, vault.get_secrets().await);
    }
}
