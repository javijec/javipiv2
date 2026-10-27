use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn minimal_fixture_is_read_only_and_never_infers_installation() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["preflight", "minimal", "--fixture"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"dietpi\naarch64\nmounted\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8(out.stderr).unwrap(), "");
    let text = String::from_utf8(out.stdout).unwrap();
    for expected in [
        "DietPi: verificado",
        "ARM64: verificado",
        "Almacenamiento: montaje informado por fixture; capacidad desconocida",
        "core: dependencias: ninguna; presencia: desconocida; acciones: desconocidas",
        "health: dependencias: core; presencia: desconocida; acciones: desconocidas",
        "storage: dependencias: core; presencia: desconocida; acciones: desconocidas",
        "services: dependencias: core; presencia: desconocida; acciones: desconocidas",
    ] {
        assert!(text.contains(expected), "falta {expected}: {text}");
    }
    assert!(text.find("core:").unwrap() < text.find("health:").unwrap());
    assert!(text.find("health:").unwrap() < text.find("storage:").unwrap());
    assert!(text.find("storage:").unwrap() < text.find("services:").unwrap());
}

fn assert_fixture_error(input: &[u8], error: &str) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["preflight", "minimal", "--fixture"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"");
    assert_eq!(String::from_utf8(out.stderr).unwrap(), error);
}

#[test]
fn rejects_invalid_fixtures() {
    assert_fixture_error(b"", "Error: fixture inválida\n");
    assert_fixture_error(b"dietpi\naarch64\nwrong\n", "Error: fixture inválida\n");
    assert_fixture_error(
        b"dietpi\naarch64\nunknown\nextra\n",
        "Error: fixture inválida\n",
    );
    assert_fixture_error(&vec![b'x'; 1025], "Error: fixture demasiado grande\n");
}

#[test]
fn debian_marker_is_evidence_but_missing_marker_is_unknown() {
    for (input, expected) in [
        ("debian-marker\nunknown\nunknown\n", "DietPi: verificado"),
        ("debian\nunknown\nunknown\n", "DietPi: desconocido"),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
            .args(["preflight", "minimal", "--fixture"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(out.stderr, b"");
        assert!(String::from_utf8(out.stdout).unwrap().contains(expected));
    }
}

#[test]
fn unknown_fixture_and_existing_plan() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["preflight", "minimal", "--fixture"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"unknown\nunknown\nunknown\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("DietPi: desconocido"));
    assert!(text.contains("ARM64: desconocido"));
    assert!(text.contains("Almacenamiento: desconocido"));
    let old = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["plan", "minimal"])
        .output()
        .unwrap();
    assert!(old.status.success());
    assert_eq!(String::from_utf8(old.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(old.stdout).unwrap(), "Perfil minimal (propuesta, no ejecuta instalaciones):\n  core            preview\n  health          preview\n  storage         planned\n  services        planned\n");
}
