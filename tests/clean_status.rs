use javipiv2::modules::{clean, storage::Capacity};
use std::process::Command;

#[test]
fn clean_help_and_rejected_command_are_safe() {
    let exe = env!("CARGO_BIN_EXE_javipiv2");
    let help = Command::new(exe).arg("help").output().unwrap();
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    assert!(String::from_utf8(help.stdout)
        .unwrap()
        .contains("clean status"));
    let invalid = Command::new(exe)
        .args(["clean", "remove"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stdout.is_empty());
    assert_eq!(
        invalid.stderr,
        "Error: comando desconocido; usá javipiv2 help\n".as_bytes()
    );
}

#[test]
fn root_capacity_is_passive_and_conservative() {
    let capacity = Capacity {
        total_bytes: 100,
        used_bytes: 60,
        available_bytes: 30,
        reserved_bytes: 10,
    };
    assert_eq!(clean::status_with(|| Ok::<_, &str>(capacity)), "Raíz (/): total=100 B, usada=60 B, disponible=30 B, reservada=10 B\nNo identifica espacio recuperable ni examina /srv.");
    assert_eq!(
        clean::status_with(|| Err("secret")),
        "Raíz (/): desconocido\nNo identifica espacio recuperable ni examina /srv."
    );
}
