#![cfg(target_os = "linux")]
use javipiv2::modules::installation_registry::{verify_fixture, Availability};
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::PathBuf,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "registry-fixture-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(root.join("var/lib/javipiv2")).unwrap();
        fs::create_dir_all(root.join("usr/local/bin")).unwrap();
        for dir in [
            "",
            "var",
            "var/lib",
            "var/lib/javipiv2",
            "usr",
            "usr/local",
            "usr/local/bin",
        ] {
            fs::set_permissions(root.join(dir), fs::Permissions::from_mode(0o755)).unwrap();
        }
        fs::write(root.join("usr/local/bin/javipiv2"), b"abc").unwrap();
        fs::set_permissions(
            root.join("usr/local/bin/javipiv2"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        let f = Self(root);
        f.record(r#"{"schema_version":1,"product_version":"0.1.0","executable":"/usr/local/bin/javipiv2","sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}"#);
        f
    }
    fn record(&self, s: &str) {
        let path = self.0.join("var/lib/javipiv2/installation.json");
        fs::write(&path, s).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
    }
    fn verify(&self) -> Availability {
        verify_fixture(&self.0, unsafe { libc::geteuid() })
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn valid_and_invalid_records() {
    let f = Fixture::new();
    assert_eq!(f.verify(), Availability::Verified);
    for record in [
        "{",
        "{}",
        r#"{"schema_version":2,"product_version":"1","executable":"/usr/local/bin/javipiv2","sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}"#,
        r#"{"schema_version":1,"schema_version":1,"product_version":"1","executable":"/usr/local/bin/javipiv2","sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}"#,
        r#"{"schema_version":1,"product_version":"1","executable":"/etc/passwd","sha256":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}"#,
        r#"{"schema_version":1,"product_version":"1","executable":"/usr/local/bin/javipiv2","sha256":"0000000000000000000000000000000000000000000000000000000000000000"}"#,
    ] {
        f.record(record);
        assert_eq!(f.verify(), Availability::Unknown);
    }
    f.record(&"x".repeat(65537));
    assert_eq!(f.verify(), Availability::Unknown);
}
#[test]
fn unsafe_and_missing_files_are_unknown() {
    let f = Fixture::new();
    fs::set_permissions(f.0.join("usr/local"), fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
    fs::set_permissions(f.0.join("usr/local"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::remove_file(f.0.join("usr/local/bin/javipiv2")).unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
    symlink("missing", f.0.join("usr/local/bin/javipiv2")).unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
}

#[test]
fn non_executable_and_group_writable_artifacts_are_unknown() {
    let f = Fixture::new();
    let exe = f.0.join("usr/local/bin/javipiv2");
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(f.verify(), Availability::Verified);
    fs::set_permissions(
        f.0.join("var/lib/javipiv2/installation.json"),
        fs::Permissions::from_mode(0o664),
    )
    .unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
}

#[test]
fn symlink_ancestor_and_fifo_registry_are_unknown() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let f = Fixture::new();
    let link = f.0.join("usr/local/bin");
    fs::rename(&link, f.0.join("usr/local/realbin")).unwrap();
    symlink("realbin", &link).unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
    fs::remove_file(&link).unwrap();
    fs::rename(f.0.join("usr/local/realbin"), &link).unwrap();
    assert_eq!(f.verify(), Availability::Verified);
    let registry = f.0.join("var/lib/javipiv2/installation.json");
    fs::remove_file(&registry).unwrap();
    let p = CString::new(registry.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(p.as_ptr(), 0o600) }, 0);
    assert_eq!(f.verify(), Availability::Unknown);
}

#[test]
fn oversized_and_hardlinked_executable_are_unknown() {
    let f = Fixture::new();
    let exe = f.0.join("usr/local/bin/javipiv2");
    fs::hard_link(&exe, f.0.join("usr/local/bin/alias")).unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
    fs::remove_file(f.0.join("usr/local/bin/alias")).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&exe)
        .unwrap()
        .set_len(128 * 1024 * 1024 + 1)
        .unwrap();
    assert_eq!(f.verify(), Availability::Unknown);
}

#[test]
fn verified_fixture_drives_one_binary_diagnosis() {
    let f = Fixture::new();
    let report = javipiv2::modules::installation::diagnose_with_status(
        f.verify(),
        || Err(std::io::Error::other("fixture")),
        || Err("fixture".into()),
        || Err("fixture".into()),
    )
    .unwrap()
    .to_string();
    assert!(report.contains("Binario: instalado verificado"));
    assert!(report.contains("Capacidades internas: core, health, storage, services"));
    fs::remove_file(f.0.join("var/lib/javipiv2/installation.json")).unwrap();
    let report = javipiv2::modules::installation::diagnose_with_status(
        f.verify(),
        || Err(std::io::Error::other("fixture")),
        || Err("fixture".into()),
        || Err("fixture".into()),
    )
    .unwrap()
    .to_string();
    assert!(report.contains("Binario: instalación desconocida"));
    assert!(!report.contains("ausente"));
}
