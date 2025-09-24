use std::process::Stdio;

fn main() {
    const CODENAME: &str = env!("FLECS_CODENAME");
    const VERSION: &str = env!("FLECS_VERSION");
    #[cfg(debug_assertions)]
    println!("cargo:rustc-env=FLECS_VERSION={VERSION}-{CODENAME}-dev");
    #[cfg(not(debug_assertions))]
    println!("cargo:rustc-env=FLECS_VERSION={VERSION}-{CODENAME}");
    let result = std::process::Command::new("git")
        .args([
            "-C",
            env!("CARGO_MANIFEST_DIR"),
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert!(result.status.success());
    let flecs_git_sha = String::from_utf8(result.stdout).unwrap();
    println!("cargo:rustc-env=FLECS_GIT_SHA={flecs_git_sha}");
    #[cfg(feature = "dev-auth")]
    println!(
        "cargo::warning=Feature dev-auth is enabled which will disable all authorization checks on http requests"
    );
}
