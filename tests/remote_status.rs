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
        Ok(services::Service {
            name: unit.into(),
            load: Some(
                if unit == "ssh.service" {
                    "loaded"
                } else {
                    "not-found"
                }
                .into(),
            ),
            active: if unit == "ssh.service" {
                None
            } else {
                Some("inactive".into())
            },
            sub: None,
        })
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

const HEADER: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n";
const V4: &str = "   0: 0100007F:1234 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000 0 12345 1 0000000000000000 100 0 0 10 0\n";
const V6: &str = "   0: 00000000000000000000000001000000:ABCD 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000  1000 0 12345 1 0000000000000000 100 0 0 10 0\n";

fn check_output(v4: Result<&[u8], ()>, v6: Result<&[u8], ()>, expected: &str) {
    let observations = remote::status_with_tcp(|_| Err("private error".into()), v4, v6);
    let output = observations
        .iter()
        .map(|o| o.line())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(output, format!("SSH (ssh.service): carga=desconocido, actividad=desconocido\nTailscale (tailscaled.service): carga=desconocido, actividad=desconocido\nTCP en escucha (sin atribución): {expected}; SSH en escucha=desconocido; conectividad SSH=desconocido; conectividad Tailscale=desconocido\n"));
    for secret in ["0100007F", "1234", "ABCD", "12345", "private error"] {
        assert!(!output.contains(secret));
    }
}

#[test]
fn remote_tcp6_kernel_header_is_accepted() {
    let header6 = HEADER.replace("rem_address", "remote_address");
    let v6 = format!("{header6}{V6}");
    assert!(
        remote::tcp_line_with(Ok(HEADER.as_bytes()), Ok(v6.as_bytes()))
            .starts_with("TCP en escucha (sin atribución): sí;")
    );
}

#[test]
fn remote_tcp_synthetic_and_redacted() {
    let empty = HEADER.as_bytes();
    let v4 = format!("{HEADER}{V4}");
    let v6 = format!("{HEADER}{V6}");
    for (a, b, expected) in [
        (v4.as_bytes(), empty, "sí"),
        (empty, v6.as_bytes(), "sí"),
        (empty, empty, "no"),
        (b"broken".as_slice(), empty, "desconocido"),
        (v4.trim_end().as_bytes(), empty, "desconocido"),
    ] {
        check_output(Ok(a), Ok(b), expected);
    }
    check_output(Err(()), Ok(empty), "desconocido");
    check_output(Ok(v4.as_bytes()), Err(()), "desconocido");
    check_output(Err(()), Err(()), "desconocido");
}

#[test]
fn remote_rejects_malformed_rows_and_headers() {
    let empty = HEADER.as_bytes();
    for (old, new) in [
        ("0:", "x:"),
        ("0100007F:1234", "0100007G:1234"),
        ("0100007F:1234", "0100007F:12G4"),
        ("00000000:0000", "0000000G:0000"),
        (" 0A ", " 0G "),
        ("00000000:00000000", "0000000G:00000000"),
        ("00:00000000", "0G:00000000"),
        ("00000000  1000", "0000000G  1000"),
        ("  1000 0 12345", "  x000 0 12345"),
        ("  1000 0 12345", "  1000 x 12345"),
        ("  1000 0 12345", "  1000 0 1234x"),
    ] {
        let row = V4.replacen(old, new, 1);
        assert_ne!(row, V4);
        let data = format!("{HEADER}{row}");
        check_output(Ok(data.as_bytes()), Ok(empty), "desconocido");
    }
    for data in [
        "",
        "sl local_address rem_address st\n",
        V4,
        "invalid header\n",
        "sl local_address rem_address st",
    ] {
        check_output(Ok(data.as_bytes()), Ok(empty), "desconocido");
    }
    let non_listener = format!("{HEADER}{}", V4.replace(" 0A ", " 01 "));
    check_output(Ok(non_listener.as_bytes()), Ok(empty), "no");
}

#[test]
fn remote_enforces_exact_tcp_limit() {
    const MAX_TCP: usize = 1024 * 1024;
    let mut exact = format!("{HEADER}{V4}").into_bytes();
    let padding = MAX_TCP - exact.len();
    exact.splice(0..0, std::iter::repeat_n(b' ', padding));
    assert_eq!(exact.len(), MAX_TCP);
    check_output(Ok(&exact), Ok(HEADER.as_bytes()), "sí");
    exact.insert(0, b' ');
    check_output(Ok(&exact), Ok(HEADER.as_bytes()), "desconocido");
    check_output(Ok(HEADER.as_bytes()), Ok(&exact), "desconocido");
}
