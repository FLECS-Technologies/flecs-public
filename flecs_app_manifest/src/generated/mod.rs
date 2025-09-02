pub mod manifest_2_0_0;
pub mod manifest_3_0_0;
#[allow(clippy::large_enum_variant)]
pub mod manifest_3_1_0;
#[allow(clippy::large_enum_variant)]
#[allow(non_snake_case)]
mod manifest_FLX_1085_rbac;

pub mod manifest_3_2_0 {
    pub use super::manifest_FLX_1085_rbac::*;
}
