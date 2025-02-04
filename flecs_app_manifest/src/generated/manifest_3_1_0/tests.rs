use super::*;

#[test]
fn minimal_manifest() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder"
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(manifest.app, App("tech.flecs.flunder".to_string()));
    assert_eq!(manifest.version.0, "0.10.0");
    assert_eq!(
        manifest.image,
        Image("flecs.azurecr.io/tech.flecs.flunder".to_string())
    );
}

#[test]
#[should_panic]
fn missing_app() {
    const MANIFEST_STR: &str = r#"{
    "version": "0.10.0",
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
    "version": "0.10.0"
}
"#;

    let _: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
}

#[test]
fn args() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "args": [
        "--adminspace-permissions=rw",
        "--rest-http-port=8000"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.args.unwrap().0,
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
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "capabilities": [
        "DOCKER",
        "IPC_LOCK",
        "NET_ADMIN"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.capabilities.unwrap().0,
        vec![
            CapabilitiesItem::Docker,
            CapabilitiesItem::IpcLock,
            CapabilitiesItem::NetAdmin,
        ]
    )
}

#[test]
fn conffiles() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
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
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.conffiles.unwrap().0,
        vec![
            ConffilesItem("default.conf:/etc/my-app/default.conf".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:rw".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:ro".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:init".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:rw,init".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:ro,init".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:no_init".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:rw,no_init".to_string()),
            ConffilesItem("default.conf:/etc/my-app/default.conf:ro,no_init".to_string()),
        ]
    )
}

#[test]
fn devices() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "devices": [
        "/dev/net/tun",
        "/dev/bus/usb"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.devices.unwrap().0,
        vec![
            DevicesItem("/dev/net/tun".to_string()),
            DevicesItem("/dev/bus/usb".to_string()),
        ]
    )
}

#[test]
fn editors() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "editors": [
        {
            "name": "editor1",
            "port": 24357,
            "supportsReverseProxy": false
        },
        {
            "name": "",
            "port": 7895,
            "supportsReverseProxy": true
        },
        {
            "name": "editor115",
            "port": 5567
        }
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.editors.unwrap().0,
        vec![
            EditorsItem {
                name: "editor1".to_string(),
                port: std::num::NonZeroU16::new(24357).unwrap(),
                supports_reverse_proxy: false
            },
            EditorsItem {
                name: "".to_string(),
                port: std::num::NonZeroU16::new(7895).unwrap(),
                supports_reverse_proxy: true
            },
            EditorsItem {
                name: "editor115".to_string(),
                port: std::num::NonZeroU16::new(5567).unwrap(),
                supports_reverse_proxy: true
            },
        ]
    )
}

#[test]
fn env() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "env": [
        "MY_ENV=value",
        "tech.flecs.some-app_value=any"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.env.unwrap().0,
        vec![
            EnvItem("MY_ENV=value".to_string()),
            EnvItem("tech.flecs.some-app_value=any".to_string()),
        ]
    )
}

#[test]
fn interactive() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "interactive": false
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert!(!manifest.interactive.unwrap().0)
}

#[test]
fn labels() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "labels": [
        "tech.flecs",
        "tech.flecs.some-label=Some custom label value"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.labels.unwrap().0,
        vec![
            LabelsItem("tech.flecs".to_string()),
            LabelsItem("tech.flecs.some-label=Some custom label value".to_string()),
        ]
    )
}

#[test]
fn multi_instance() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "multiInstance": false
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert!(!manifest.multi_instance.unwrap().0)
}

#[test]
fn ports() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
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
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.ports.unwrap().0,
        vec![
            PortsItem("8001:8001".to_string()),
            PortsItem("5000".to_string()),
            PortsItem("5001-5008:6001-6008".to_string()),
            PortsItem("6001-6008".to_string()),
        ]
    )
}

#[test]
fn revision() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "revision": "10"
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(manifest.revision.unwrap().0, "10".to_string())
}

#[test]
fn volumes() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "volumes": [
        "my-app-etc:/etc/my-app",
        "/etc/my-app:/etc/my-app"
    ]
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.volumes.unwrap().0,
        vec![
            VolumesItem("my-app-etc:/etc/my-app".to_string()),
            VolumesItem("/etc/my-app:/etc/my-app".to_string()),
        ]
    )
}

#[test]
fn minimum_flecs_version() {
    const MANIFEST_STR: &str = r#"{
    "app": "tech.flecs.flunder",
    "version": "0.10.0",
    "image": "flecs.azurecr.io/tech.flecs.flunder",
    "_minimumFlecsVersion": "3.6.0-hedgehog"
}
"#;

    let manifest: FlecsAppManifest = serde_json::from_str(MANIFEST_STR).unwrap();
    let FlecsAppManifest::Single(manifest) = manifest else {
        panic!("Wrong manifest type");
    };
    assert_eq!(
        manifest.minimum_flecs_version,
        Some(MinimumFlecsVersion("3.6.0-hedgehog".to_string()))
    )
}
