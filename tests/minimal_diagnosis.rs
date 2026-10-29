use javipiv2::modules::{
    health::SystemStatus, installation, services::Service, storage::StorageStatus,
};

#[test]
fn synthetic_queries_do_not_claim_installation_or_leak_details() {
    let report = installation::diagnose_with(
        || {
            Ok(SystemStatus {
                hostname: "private-host".into(),
                uptime_seconds: 42,
                memory_total_kib: 100,
                memory_available_kib: 50,
            })
        },
        || {
            Ok(StorageStatus {
                autofs: true,
                srv_ext4: false,
                docker_overlays: vec!["/private-path".into()],
            })
        },
        || {
            Ok(vec![Service {
                name: "private.service".into(),
                load: Some("loaded".into()),
                active: Some("active".into()),
                sub: None,
            }])
        },
    )
    .unwrap();
    let text = report.to_string();
    for module in ["core", "health", "storage", "services"] {
        assert!(text.contains(&format!("{module}: no verificado")), "{text}");
    }
    for private in ["private-host", "/private-path", "private.service"] {
        assert!(!text.contains(private));
    }
    assert!(text.contains("Health: disponible"));
    assert!(text.contains("Storage: disponible"));
    assert!(text.contains("Services: disponible"));
    assert!(text.contains("Plan de instalación ejecutable: no definido"));
}

#[test]
fn partial_failures_and_recovery_never_change_installation_state() {
    let failed = installation::diagnose_with(
        || Err(std::io::Error::other("secret health")),
        || Err("secret storage".to_owned()),
        || Err("secret services".to_owned()),
    )
    .unwrap()
    .to_string();
    for name in ["Health", "Storage", "Services"] {
        assert!(failed.contains(&format!("{name}: error de consulta")));
    }
    assert!(!failed.contains("secret"));
    assert!(failed.contains("Plan de instalación ejecutable: no definido"));
    let recovered = installation::diagnose_with(
        || {
            Ok(SystemStatus {
                hostname: "host".into(),
                uptime_seconds: 1,
                memory_total_kib: 2,
                memory_available_kib: 1,
            })
        },
        || Ok(StorageStatus::default()),
        || Ok(vec![]),
    )
    .unwrap()
    .to_string();
    assert!(recovered.contains("Health: disponible"));
    assert!(recovered.contains("Storage: desconocido"));
    assert!(recovered.contains("Services: desconocido"));
    assert!(recovered.contains("core: no verificado"));
    assert!(!recovered.contains("secret"));
}

#[test]
fn cli_help_and_invalid_profile_do_not_query_host() {
    use std::process::Command;
    let bin = env!("CARGO_BIN_EXE_javipiv2");
    let help = Command::new(bin).arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8(help.stdout)
        .unwrap()
        .contains("diagnose minimal"));
    assert!(help.stderr.is_empty());
    let invalid = Command::new(bin)
        .args(["diagnose", "other"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(!invalid.stderr.is_empty());
}
