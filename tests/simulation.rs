use std::io::Write;
use std::process::{Command, Stdio};

fn simulate(input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["simulate", "minimal", "--fixture"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn simulation_is_repeatable_and_honest() {
    let first = simulate(b"dietpi\naarch64\nmounted\n");
    let second = simulate(b"dietpi\naarch64\nmounted\n");
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stderr, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(second.stderr, b"");
    let text = String::from_utf8(first.stdout).unwrap();
    for part in [
        "DietPi: verificado",
        "ARM64: verificado",
        "capacidad desconocida",
        "Componentes de código disponibles",
        "core",
        "health",
        "Operaciones previstas",
        "acciones de instalación sin definir",
        "Comprobaciones pendientes",
        "Idempotencia y reanudación: pendientes",
    ] {
        assert!(text.contains(part), "falta {part}: {text}");
    }
    assert!(
        text.find("core: dependencias:").unwrap() < text.find("health: dependencias:").unwrap()
    );
    assert!(
        text.find("health: dependencias:").unwrap() < text.find("storage: dependencias:").unwrap()
    );
    assert!(
        text.find("storage: dependencias:").unwrap()
            < text.find("services: dependencias:").unwrap()
    );
}

#[test]
#[ignore = "requires JAVIPIV2_TEST_SANDBOX with a prepared sentinel"]
fn simulation_does_not_change_isolated_state() {
    let root = std::path::PathBuf::from(
        std::env::var_os("JAVIPIV2_TEST_SANDBOX").expect("set JAVIPIV2_TEST_SANDBOX"),
    );
    assert_eq!(std::fs::read(root.join("sentinel")).unwrap(), b"unchanged");
    for input in [
        b"dietpi\naarch64\nunknown\n".as_slice(),
        b"dietpi\naarch64\nunknown\n",
        b"invalid\n",
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
            .args(["simulate", "minimal", "--fixture"])
            .current_dir(&root)
            .env("HOME", &root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        if input == b"invalid\n" {
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, b"");
            assert_eq!(output.stderr, "Error: fixture inválida\n".as_bytes());
        } else {
            assert_eq!(output.status.code(), Some(0));
            assert_eq!(output.stderr, b"");
        }
        let names: Vec<_> = std::fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["sentinel"]);
        assert_eq!(std::fs::read(root.join("sentinel")).unwrap(), b"unchanged");
    }
}

#[test]
fn simulation_rejects_invalid_input_without_output() {
    let out = simulate(b"invalid\n");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"");
    assert_eq!(out.stderr, "Error: fixture inválida\n".as_bytes());
}
