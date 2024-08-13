mod app;
mod manifest;
mod secret;

use super::Result;
pub use app::*;
pub use manifest::*;
pub use secret::*;
use serde::{Deserialize, Serialize};

pub(super) trait VaultPouch: Pouch {
    fn close(&mut self) -> Result<()>;
    fn open(&mut self) -> Result<()>;
}

/// Structs that implement the Pouch trait have to provide an object of the chosen type [Self::Gems]
/// with the functions [Self::gems()] and [Self::gems_mut()].
/// # Example
/// ```
/// use flecs_core::vault::pouch::Pouch;
/// #[derive(Debug, Eq, PartialEq)]
/// enum Arrow {
///     Standard,
///     Fire,
///     Ice,
/// }
///
/// struct ArrowPouch {
///     arrows: Vec<Arrow>,
/// }
///
/// impl Pouch for ArrowPouch {
///     type Gems = Vec<Arrow>;
///
///     fn gems(&self) -> &Self::Gems {
///         &self.arrows
///     }
///
///     fn gems_mut(&mut self) -> &mut Self::Gems {
///         &mut self.arrows
///     }
///
/// }
///
/// let mut arrow_pouch = ArrowPouch {
///     arrows: Vec::new(),
/// };
///
/// arrow_pouch.gems_mut().push(Arrow::Fire);
/// arrow_pouch.gems_mut().push(Arrow::Ice);
/// assert_eq!(arrow_pouch.gems(), &vec![Arrow::Fire, Arrow::Ice])
/// ```
pub trait Pouch {
    type Gems;
    fn gems(&self) -> &Self::Gems;
    fn gems_mut(&mut self) -> &mut Self::Gems;
}

fn combine_results(left: Result<()>, right: Result<()>) -> Result<()> {
    match (left, right) {
        (Err(e1), Err(e2)) => Err(e1.append(e2)),
        (Err(e), _) | (_, Err(e)) => Err(e),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Ord, PartialOrd, Eq, Serialize, Deserialize)]
pub struct AppKey {
    name: String,
    version: String,
}
