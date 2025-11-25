use anyhow::Result;
use schemars::schema::{RootSchema, Schema, SchemaObject};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use typify::{TypeSpace, TypeSpaceSettings};

const GITHUB_REPO: &str = "FLECS-Technologies/app-manifest";

#[allow(dead_code)]
enum Reference {
    TagOrBranch(String),
    CommitHash(String),
    PullRequest(u16),
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Reference::TagOrBranch(tag) => write!(f, "tag/branch {tag}"),
            Reference::CommitHash(hash) => write!(f, "commit {hash}"),
            Reference::PullRequest(number) => write!(f, "pull request #{number}"),
        }
    }
}

impl Reference {
    fn determine_hash(&self) -> Result<String> {
        let reference = match self {
            Reference::TagOrBranch(tag) => tag.clone(),
            Reference::CommitHash(hash) => return Ok(hash.to_string()),
            Reference::PullRequest(number) => format!("refs/pull/{number}/head"),
        };
        let url = format!("https://github.com/{GITHUB_REPO}.git");
        let output = Command::new("git")
            .args(["ls-remote", &url, &reference])
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?
            .stdout;
        let output = String::from_utf8(output)?;
        Ok(output.split_whitespace().next().unwrap().to_string())
    }
}

fn download_app_manifest_schema(hash: &str) -> Result<String, reqwest::Error> {
    let url =
        format!("https://raw.githubusercontent.com/{GITHUB_REPO}/{hash}/manifest.schema.json");
    let content = reqwest::blocking::get(url)?.text()?;
    eprintln!("{}", content);
    Ok(content)
}

fn remove_schema_version(schema_object: &mut SchemaObject) {
    if let Some(object_validation) = &mut schema_object.object {
        object_validation.required.remove("_schemaVersion");
        object_validation.properties.remove("_schemaVersion");
    }
}

fn generate_code(reference: Reference, module_name: &str) -> Result<()> {
    let directory: PathBuf = match std::env::args().nth(1) {
        None => ".".into(),
        Some(base) => PathBuf::from(base),
    };
    let file_path = directory.join(format!("{module_name}.rs"));
    let modules_directory = directory.join(module_name);
    let test_module_path = modules_directory.join("tests.rs");
    std::fs::create_dir_all(&directory)?;
    let mut out_file = File::create(file_path)?;

    out_file.write_all(b"#![cfg_attr(any(), rustfmt::skip)]")?;
    out_file.write_all(b"#![allow(clippy::clone_on_copy)]")?;
    out_file.write_all(b"#![allow(clippy::to_string_trait_impl)]")?;
    let hash = reference.determine_hash()?;
    let schema = download_app_manifest_schema(&hash)?;
    let mut schema: RootSchema = serde_json::from_str(&schema)?;
    // The Schema version in our rust code is encoded in the enum variant (see crate::AppManifestVersion)
    // Using serde to decide from the value _schemaVersion which variant should be built
    // consumes this field which results in an error later if it is required by the schema.
    remove_schema_version(&mut schema.schema);
    for schema in schema.definitions.values_mut() {
        if let Schema::Object(schema) = schema {
            remove_schema_version(schema)
        }
    }
    let mut type_space = TypeSpace::new(
        TypeSpaceSettings::default()
            .with_struct_builder(true)
            .with_derive("PartialEq".to_string())
            .with_derive("Eq".to_string()),
    );
    type_space.add_ref_types(schema.definitions)?;
    type_space.add_type(&Schema::Object(schema.schema))?;
    out_file.write_fmt(format_args!(
        r#"
#[doc = "Generated types for `FLECS App Manifest` for {reference} - {hash}"]
use serde::{{Deserialize, Serialize}};
{}
#[cfg(test)]
mod tests;"#,
        prettyplease::unparse(&syn::parse2::<syn::File>(type_space.to_stream())?),
    ))?;

    std::fs::create_dir_all(&modules_directory)?;
    if !test_module_path.try_exists()? {
        File::create(test_module_path)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    for (reference, module_name) in [
        (
            Reference::TagOrBranch("2.0.0".to_string()),
            "manifest_2_0_0",
        ),
        (
            Reference::TagOrBranch("3.0.0".to_string()),
            "manifest_3_0_0",
        ),
        (
            Reference::TagOrBranch("3.1.0".to_string()),
            "manifest_3_1_0",
        ),
        (
            Reference::TagOrBranch("3.2.0".to_string()),
            "manifest_3_2_0",
        ),
    ] {
        generate_code(reference, module_name)?;
    }
    Ok(())
}
