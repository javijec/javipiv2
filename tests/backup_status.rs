use std::io::Write;
use std::process::{Command, Stdio};

fn invoke(args: &[&str], input: Option<&[u8]>) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(bytes) = input {
        child.stdin.take().unwrap().write_all(bytes).unwrap();
    }
    child.wait_with_output().unwrap()
}
const UNKNOWN: &str = "Configuración: desconocida\nSnapshots actuales: desconocido\nÚltimo backup: desconocido\nÚltimo intento: desconocido\nResultado del último intento: desconocido\n";
#[test]
fn status_without_trusted_source_is_unknown() {
    let result = invoke(&["backup", "status"], None);
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, UNKNOWN.as_bytes());
    assert!(result.stderr.is_empty());
}
#[test]
fn supplied_empty_is_not_inferred_to_be_an_empty_repository() {
    let result = invoke(&["backup", "status", "--stdin"], Some(b"[]"));
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(String::from_utf8(result.stdout).unwrap(), format!("{UNKNOWN}Datos suministrados por stdin (origen y actualidad no verificados):\nLista suministrada: sin snapshots; no acredita el estado del repositorio.\n"));
    assert!(result.stderr.is_empty());
}
#[test]
fn supplied_snapshot_never_implies_last_attempt_success() {
    let result = invoke(
        &["backup", "status", "--stdin"],
        Some(br#"[{"time":"2025-01-01T00:00:00Z","short_id":"12345678","tags":["profile:full"]}]"#),
    );
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(String::from_utf8(result.stdout).unwrap(), format!("{UNKNOWN}Datos suministrados por stdin (origen y actualidad no verificados):\nSnapshots suministrados: 1\nSnapshot suministrado más reciente: 2025-01-01T00:00:00Z\n"));
    assert!(result.stderr.is_empty());
}
#[test]
fn invalid_supplied_json_is_rejected_without_stdout() {
    let result = invoke(&["backup", "status", "--stdin"], Some(b"{"));
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"");
    assert_eq!(
        result.stderr,
        "Error: JSON de snapshots inválido\n".as_bytes()
    );
}
