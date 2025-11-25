# How to generate flecs app manifest rust code from schema
`git` needs to be available in `$PATH` as git is used during generation to determine commit hashes.

1. Edit the list of references and module names at the bottom of `src/generate.rs`
2. Add the new modules to `src/generated/mod.rs`
3. Execute the generate binary and pass the directory you want to code to be generated in e.g. `cargo run --bin generate -- flecs_app_manifest/src/generated/` if you do not pass any argument the code will be generated at the current workdir
