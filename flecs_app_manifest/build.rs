use schemars::schema::{RootSchema, Schema, SchemaObject};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use typify::{TypeSpace, TypeSpaceSettings};

struct Error(String, bool);

fn download_app_manifest_schema(version: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://raw.githubusercontent.com/FLECS-Technologies/app-manifest/{version}/manifest.schema.json"
    );
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

fn generate_code(version: &str) -> Result<(), Error> {
    let path_version = version.replace(['.', '-'], "_");
    let directory: PathBuf = format!("src/generated/manifest_{path_version}/").into();
    let file_path = directory.join("mod.rs");
    let file_exists = file_path
        .try_exists()
        .map_err(|e| Error(format!("{e}"), false))?;
    std::fs::create_dir_all(&directory).unwrap();
    let mut out_file = File::create(file_path).map_err(|e| Error(format!("{e}"), file_exists))?;

    out_file
        .write_all(b"#![cfg_attr(any(), rustfmt::skip)]")
        .unwrap();
    out_file
        .write_all(b"#![allow(clippy::clone_on_copy)]")
        .unwrap();
    out_file
        .write_all(b"#![allow(clippy::to_string_trait_impl)]")
        .unwrap();
    let schema =
        download_app_manifest_schema(version).map_err(|e| Error(format!("{e}"), file_exists))?;
    let mut schema: RootSchema =
        serde_json::from_str(&schema).map_err(|e| Error(format!("{e}"), file_exists))?;
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
    type_space
        .add_ref_types(schema.definitions)
        .map_err(|e| Error(format!("{e}"), file_exists))?;
    type_space
        .add_type(&Schema::Object(schema.schema))
        .map_err(|e| Error(format!("{e}"), file_exists))?;
    out_file
        .write_fmt(format_args!(
            r#"
#[doc = "Generated types for `FLECS App Manifest` version {version}"]
use serde::{{Deserialize, Serialize}};
{}
#[cfg(test)]
mod tests;"#,
            prettyplease::unparse(
                &syn::parse2::<syn::File>(type_space.to_stream())
                    .map_err(|e| Error(format!("{e}"), file_exists))?
            ),
        ))
        .map_err(|e| Error(format!("{e}"), file_exists))?;

    Ok(())
}

fn main() {
    for version in ["2.0.0", "3.0.0", "3.1.0", "3.2.0"] {
        match generate_code(version) {
            Ok(()) => {}
            Err(Error(message, file_exists)) => {
                if file_exists {
                    println!(
                        "cargo::warning=Could not generate code for app manifest {version} but previously generated code exists: {message}"
                    )
                } else {
                    panic!("{message}")
                }
            }
        }
    }
}
