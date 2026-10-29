use javipiv2::modules::{remote, services};
use std::process::Command;

#[test]
fn remote_help_and_invalid_command_are_safe() {
    let executable = env!("CARGO_BIN_EXE_javipiv2");
    let help = Command::new(executable).arg("help").output().unwrap();
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    assert!(String::from_utf8(help.stdout)
        .unwrap()
        .contains("remote status"));
    let invalid = Command::new(executable)
        .args(["remote", "nonsense"])
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
fn remote_queries_only_known_units_and_reports_independent_states() {
    let mut calls = Vec::new();
    let observations = remote::status_with(|unit| {
        calls.push(unit.to_owned());
        if unit == "ssh.service" {
            Ok(services::Service {
                name: unit.into(),
                load: Some("loaded".into()),
                active: None,
                sub: None,
            })
        } else {
            Ok(services::Service {
                name: unit.into(),
                load: Some("not-found".into()),
                active: Some("inactive".into()),
                sub: None,
            })
        }
    });
    assert_eq!(calls, ["ssh.service", "tailscaled.service"]);
    assert_eq!(
        observations[0].line(),
        "SSH (ssh.service): carga=loaded, actividad=desconocido"
    );
    assert_eq!(
        observations[1].line(),
        "Tailscale (tailscaled.service): carga=not-found, actividad=inactive"
    );
}

#[test]
fn remote_errors_are_unknown_and_not_exposed() {
    let observations = remote::status_with(|_| Err("secret error".into()));
    assert_eq!(
        observations[0].line(),
        "SSH (ssh.service): carga=desconocido, actividad=desconocido"
    );
    assert_eq!(
        observations[1].line(),
        "Tailscale (tailscaled.service): carga=desconocido, actividad=desconocido"
    );
}
