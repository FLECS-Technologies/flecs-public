use super::*;

#[test]
fn minimal_manifest() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder"
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.app,
        FlecsAppManifestApp("tech.flecs.flunder".to_string())
    );
    assert_eq!(manifest.version, "3.0.0");
    assert_eq!(
        manifest.image,
        FlecsAppManifestImage("flecs.azurecr.io/tech.flecs.flunder".to_string())
    );
}

#[test]
#[should_panic]
fn missing_app() {
    const MANIFEST_STR: &str = r#"{
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder"
}
"#;

    let _: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
}
#[test]
#[should_panic]
fn missing_version() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "image": "flecs.azurecr.io/tech.flecs.flunder"
}
"#;

    let _: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
}
#[test]
#[should_panic]
fn missing_image() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0"
}
"#;

    let _: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
}

#[test]
fn args() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "args": [
        "--adminspace-permissions=rw",
        "--rest-http-port=8000"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.args,
        vec![
            "--adminspace-permissions=rw".to_string(),
            "--rest-http-port=8000".to_string()
        ]
    )
}

#[test]
fn capabilities() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "capabilities": [
        "DOCKER",
        "IPC_LOCK",
        "NET_ADMIN"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.capabilities,
        Some(vec![
            FlecsAppManifestCapabilitiesItem::Docker,
            FlecsAppManifestCapabilitiesItem::IpcLock,
            FlecsAppManifestCapabilitiesItem::NetAdmin,
        ])
    )
}

#[test]
fn conffiles() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "conffiles": [
        "default.conf:/etc/my-app/default.conf",
        "default.conf:/etc/my-app/default.conf:rw",
        "default.conf:/etc/my-app/default.conf:ro",
        "default.conf:/etc/my-app/default.conf:init",
        "default.conf:/etc/my-app/default.conf:rw,init",
        "default.conf:/etc/my-app/default.conf:ro,init",
        "default.conf:/etc/my-app/default.conf:no_init",
        "default.conf:/etc/my-app/default.conf:rw,no_init",
        "default.conf:/etc/my-app/default.conf:ro,no_init"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.conffiles,
        vec![
            FlecsAppManifestConffilesItem("default.conf:/etc/my-app/default.conf".to_string()),
            FlecsAppManifestConffilesItem("default.conf:/etc/my-app/default.conf:rw".to_string()),
            FlecsAppManifestConffilesItem("default.conf:/etc/my-app/default.conf:ro".to_string()),
            FlecsAppManifestConffilesItem("default.conf:/etc/my-app/default.conf:init".to_string()),
            FlecsAppManifestConffilesItem(
                "default.conf:/etc/my-app/default.conf:rw,init".to_string()
            ),
            FlecsAppManifestConffilesItem(
                "default.conf:/etc/my-app/default.conf:ro,init".to_string()
            ),
            FlecsAppManifestConffilesItem(
                "default.conf:/etc/my-app/default.conf:no_init".to_string()
            ),
            FlecsAppManifestConffilesItem(
                "default.conf:/etc/my-app/default.conf:rw,no_init".to_string()
            ),
            FlecsAppManifestConffilesItem(
                "default.conf:/etc/my-app/default.conf:ro,no_init".to_string()
            ),
        ]
    )
}

#[test]
fn devices() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "devices": [
        "/dev/net/tun",
        "/dev/bus/usb"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.devices,
        vec![
            FlecsAppManifestDevicesItem("/dev/net/tun".to_string()),
            FlecsAppManifestDevicesItem("/dev/bus/usb".to_string()),
        ]
    )
}

#[test]
fn editor() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "editor": ":12345"
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.editor,
        Some(FlecsAppManifestEditor(":12345".to_string()))
    )
}

#[test]
fn env() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "env": [
        "MY_ENV=value",
        "tech.flecs.some-app_value=any"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.env,
        vec![
            FlecsAppManifestEnvItem("MY_ENV=value".to_string()),
            FlecsAppManifestEnvItem("tech.flecs.some-app_value=any".to_string()),
        ]
    )
}

#[test]
fn interactive() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "interactive": false
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(manifest.interactive, Some(false))
}

#[test]
fn labels() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "labels": [
        "tech.flecs",
        "tech.flecs.some-label=Some custom label value"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.labels,
        vec![
            FlecsAppManifestLabelsItem("tech.flecs".to_string()),
            FlecsAppManifestLabelsItem("tech.flecs.some-label=Some custom label value".to_string()),
        ]
    )
}

#[test]
fn multi_instance() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "multiInstance": false
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(manifest.multi_instance, Some(false))
}

#[test]
fn ports() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "ports": [
        "8001:8001",
        "5000",
        "5001-5008:6001-6008",
        "6001-6008"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.ports,
        vec![
            FlecsAppManifestPortsItem("8001:8001".to_string()),
            FlecsAppManifestPortsItem("5000".to_string()),
            FlecsAppManifestPortsItem("5001-5008:6001-6008".to_string()),
            FlecsAppManifestPortsItem("6001-6008".to_string()),
        ]
    )
}

#[test]
fn revision() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "revision": "10"
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(manifest.revision, Some("10".to_string()))
}

#[test]
fn volumes() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "3.0.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "volumes": [
        "my-app-etc:/etc/my-app",
        "/etc/my-app:/etc/my-app"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    assert_eq!(
        manifest.volumes,
        vec![
            FlecsAppManifestVolumesItem("my-app-etc:/etc/my-app".to_string()),
            FlecsAppManifestVolumesItem("/etc/my-app:/etc/my-app".to_string()),
        ]
    )
}
