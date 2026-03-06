use std::process::Stdio;

fn main() {
    const CODENAME: &str = env!("FLECS_CODENAME");
    const VERSION: &str = env!("FLECS_VERSION");

    let sha_output = std::process::Command::new("git")
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
    assert!(sha_output.status.success());
    let git_sha = String::from_utf8(sha_output.stdout).unwrap();
    let git_sha = git_sha.trim();
    println!("cargo:rustc-env=FLECS_GIT_SHA={git_sha}");

    let dirty_output = std::process::Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "status", "--porcelain"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert!(dirty_output.status.success());
    let dirty_suffix = if dirty_output.stdout.is_empty() {
        ""
    } else {
        "-dirty"
    };

    let is_release = std::env::var("PROFILE").unwrap_or_default() == "release";
    let is_ci = std::env::var("CI").unwrap_or_default() == "true";

    let flecs_version = if is_release {
        if is_ci {
            format!("{VERSION}-{CODENAME}-{git_sha}{dirty_suffix}")
        } else {
            format!("{VERSION}-{CODENAME}-local-{git_sha}{dirty_suffix}")
        }
    } else {
        format!("{VERSION}-{CODENAME}-next-dev-{git_sha}{dirty_suffix}")
    };
    println!("cargo:rustc-env=FLECS_VERSION={flecs_version}");

    #[cfg(feature = "dev-auth")]
    println!(
        "cargo::warning=Feature dev-auth is enabled which will disable all authorization checks on http requests"
    );
}
