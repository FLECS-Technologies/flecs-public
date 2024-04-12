fn main() -> miette::Result<()> {
    let _build = cxx_build::bridge("src/lib.rs");
    let path = std::path::PathBuf::from("src"); // include path
    let flecs = std::path::PathBuf::from("../"); // include path
    let mut b = autocxx_build::Builder::new("src/lib.rs", [&path, &flecs])
        .extra_clang_args(&["-std=c++20", "-DFLECS_FINAL_UNLESS_TESTED=final"])
        .build()?;
    // This assumes all your C++ bindings are in main.rs
    b.flag_if_supported("-std=c++20").compile("autocxx-demo"); // arbitrary library name, pick anything
    println!("cargo:rerun-if-changed=src/lib.rs");
    // Add instructions to link to any C++ libraries you need.
    Ok(())
}
