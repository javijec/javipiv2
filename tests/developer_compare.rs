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

#[test]
fn compares_only_supplied_versions() {
    for (a, b, result) in [
        ("1.2.3", "1.2.3", "iguales"),
        ("1.2.3", "1.2.4", "primera menor"),
        ("2.0.0", "1.999999999.999999999", "primera mayor"),
        ("0.0.0", "000.0.0", "iguales"),
    ] {
        assert_eq!(run(&["developer", "compare", a, b]), (0, format!("Versiones suministradas por el usuario; no acreditan instalación ni disponibilidad.\nResultado: {result}\n"), String::new()));
    }
}

#[test]
fn invalid_versions_are_unknown() {
    for value in [
        "",
        "v1.2.3",
        "1.2",
        "1.2.3.4",
        "1.2.3-beta",
        " 1.2.3",
        "1000000000.0.0",
        "1.2.-3",
        "1.2.é",
    ] {
        for args in [[value, "1.2.3"], ["1.2.3", value]] {
            assert_eq!(run(&["developer", "compare", args[0], args[1]]), (0, "Versiones suministradas por el usuario; no acreditan instalación ni disponibilidad.\nResultado: desconocido\n".into(), String::new()));
        }
    }
}

#[test]
fn malformed_invocation_and_sibling_preserved() {
    assert_eq!(
        run(&["developer", "compare", "1.2.3"]),
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
