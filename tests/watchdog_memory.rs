use javipiv2::modules::{health::SystemStatus, watchdog};
use std::process::Command;

fn run(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(args)
        .output()
        .unwrap();
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn assessment(total: &str, available: &str, minimum: &str, result: &str) {
    assert_eq!(run(&["watchdog", "memory", total, available, minimum]), (0, format!("Valores suministrados por el usuario; no son monitoreo real.\nResultado: {result}\n"), String::new()));
}

#[test]
fn assesses_boundaries_without_overflow() {
    for (total, available, minimum, result) in [
        ("100", "20", "20", "saludable"),
        ("100", "19", "20", "por debajo del mínimo"),
        ("3", "1", "33", "saludable"),
        ("3", "1", "34", "por debajo del mínimo"),
        ("1", "0", "0", "saludable"),
        ("1", "1", "100", "saludable"),
        ("1", "0", "100", "por debajo del mínimo"),
        (
            "18446744073709551615",
            "18446744073709551615",
            "100",
            "saludable",
        ),
    ] {
        assessment(total, available, minimum, result);
    }
}

#[test]
fn invalid_values_are_unknown() {
    for (total, available, minimum) in [
        ("0", "0", "0"),
        ("1", "2", "0"),
        ("1", "1", "101"),
        ("", "0", "0"),
        ("+1", "0", "0"),
        (" 1", "0", "0"),
        ("١", "0", "0"),
        ("18446744073709551616", "0", "0"),
        ("000000000000000000001", "0", "0"),
        ("1", "-1", "0"),
        ("1", "0", "1.0"),
    ] {
        assessment(total, available, minimum, "desconocido");
    }
}

#[test]
fn status_uses_injected_health_and_hides_errors() {
    for (total, available, minimum, expected) in [
        (100, 20, "20", "saludable"),
        (100, 19, "20", "por debajo del mínimo"),
        (1, 0, "0", "saludable"),
        (1, 1, "100", "saludable"),
        (0, 0, "0", "desconocido"),
        (1, 2, "0", "desconocido"),
        (1, 1, "101", "desconocido"),
        (1, 1, "x", "desconocido"),
    ] {
        let observation = watchdog::status_with(minimum, || {
            Ok(SystemStatus {
                hostname: "synthetic".into(),
                uptime_seconds: 0,
                memory_total_kib: total,
                memory_available_kib: available,
            })
        });
        assert_eq!(observation.line(), format!("Resultado: {expected}"));
    }
    let observation = watchdog::status_with("20", || Err(std::io::Error::other("secret error")));
    assert_eq!(observation.line(), "Resultado: desconocido");
}

#[test]
fn status_help_and_missing_threshold() {
    let (code, stdout, stderr) = run(&["help"]);
    assert_eq!(code, 0);
    assert_eq!(stderr, "");
    assert!(stdout.lines().any(|line| line
        == "  watchdog status <minimum-percent>  Consultar memoria Health, sin monitoreo"));
    assert_eq!(
        run(&["watchdog", "status"]),
        (
            1,
            String::new(),
            "Error: comando desconocido; usá javipiv2 help\n".into()
        )
    );
}

#[test]
fn malformed_and_existing_commands_preserved() {
    assert_eq!(
        run(&["watchdog", "memory", "1", "1"]),
        (
            1,
            String::new(),
            "Error: comando desconocido; usá javipiv2 help\n".into()
        )
    );
    assert_eq!(
        run(&["version"]),
        (
            0,
            format!("javipiv2 {}\n", env!("CARGO_PKG_VERSION")),
            String::new()
        )
    );
}
