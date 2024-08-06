mod manifest;

pub use crate::manifest::download_manifest;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn download_manifest(x_session_id: &str, app: &str, version: &str) -> Result<String>;
    }
}
