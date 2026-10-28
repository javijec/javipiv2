use std::io::Write;
use std::process::{Command, Stdio};

fn inspect(input: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["backup", "inspect", "--stdin"])
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
    let result = child.wait_with_output().unwrap();
    (
        result.status.code().unwrap(),
        String::from_utf8(result.stdout).unwrap(),
        String::from_utf8(result.stderr).unwrap(),
    )
}
fn unknown(count: usize) -> (i32, String, String) {
    (0, format!("Snapshots: {count}\nÚltimo snapshot: desconocido\nÚltimo intento: desconocido\nResultado del último intento: desconocido\n"), String::new())
}
fn known(count: usize, time: &str, id: &str, profile: &str) -> (i32, String, String) {
    (0, format!("Snapshots: {count}\nFecha: {time}\nID: {id}\nPerfil: {profile}\nÚltimo intento: desconocido\nResultado del último intento: desconocido\n"), String::new())
}
#[test]
fn empty_and_malformed_are_distinct() {
    assert_eq!(inspect("[]"), unknown(0));
    for input in ["{", "{}", "null", "123"] {
        assert_eq!(
            inspect(input),
            (1, "".into(), "Error: JSON de snapshots inválido\n".into())
        );
    }
}
#[test]
fn newest_uses_instant_and_profiles() {
    for profile in ["full", "projects", "engram"] {
        let input = format!(
            r#"[{{"time":"2024-01-01T12:00:00Z","id":"abcdef012345","tags":["profile:{profile}"]}},{{"time":"2024-01-01T13:00:00+02:00","id":"abcdef00"}}]"#
        );
        assert_eq!(
            inspect(&input),
            known(2, "2024-01-01T12:00:00Z", "abcdef01", profile)
        );
    }
}
#[test]
fn older_tie_does_not_hide_unique_maximum() {
    let input = r#"[{"time":"2024-01-01T00:00:00Z"},{"time":"2024-01-01T01:00:00+01:00"},{"time":"2024-01-02T00:00:00Z"}]"#;
    assert_eq!(
        inspect(input),
        known(3, "2024-01-02T00:00:00Z", "desconocido", "desconocido")
    );
}
#[test]
fn fractional_and_timezone_ordering_and_ties() {
    let input = r#"[{"time":"2024-01-01T01:00:00.100000001+01:00"},{"time":"2023-12-31T19:00:00.100000002-05:00","short_id":"abcdef12"}]"#;
    assert_eq!(
        inspect(input),
        known(
            2,
            "2023-12-31T19:00:00.100000002-05:00",
            "abcdef12",
            "desconocido"
        )
    );
    assert_eq!(
        inspect(r#"[{"time":"2024-01-01T00:00:00.10Z"},{"time":"2024-01-01T01:00:00.1+01:00"}]"#),
        unknown(2)
    );
}
#[test]
fn unknown_fields_and_ambiguous_dates() {
    assert_eq!(
        inspect(r#"[{"time":"2024-01-01T00:00:00Z","tags":["other"]}]"#),
        known(1, "2024-01-01T00:00:00Z", "desconocido", "desconocido")
    );
    assert_eq!(
        inspect(
            r#"[{"time":"2024-01-01T00:00:00Z","id":3,"tags":["profile:full","profile:projects"]}]"#
        ),
        known(1, "2024-01-01T00:00:00Z", "desconocido", "desconocido")
    );
    assert_eq!(
        inspect(r#"[{"time":"2024-01-01T00:00:00Z","tags":true}]"#),
        known(1, "2024-01-01T00:00:00Z", "desconocido", "desconocido")
    );
    for input in [
        r#"[{"time":"bad"}]"#,
        r#"[{"time":null}]"#,
        r#"[42]"#,
        r#"[{"time":"2024-02-30T00:00:00Z"}]"#,
    ] {
        assert_eq!(inspect(input), unknown(1));
    }
    assert_eq!(
        inspect(r#"[{}, {"time":"2024-01-01T00:00:00Z"}]"#),
        unknown(2)
    );
}
#[test]
fn invalid_short_id_falls_back_to_valid_id() {
    let input = r#"[{"time":"2024-01-01T00:00:00Z","short_id":"bad!","id":"abcdef012345"}]"#;
    assert_eq!(
        inspect(input),
        known(1, "2024-01-01T00:00:00Z", "abcdef01", "desconocido")
    );
}
#[test]
fn mixed_type_tags_do_not_establish_profile() {
    let input =
        r#"[{"time":"2024-01-01T00:00:00Z","id":"abcdef012345","tags":["profile:full",42]}]"#;
    assert_eq!(
        inspect(input),
        known(1, "2024-01-01T00:00:00Z", "abcdef01", "desconocido")
    );
}
#[test]
fn unknown_offset_prevents_latest_selection() {
    let input = r#"[{"time":"2024-01-01T00:00:00-00:00"},{"time":"2024-01-02T00:00:00Z"}]"#;
    assert_eq!(inspect(input), unknown(2));
}
#[test]
fn input_size_is_limited() {
    assert_eq!(
        inspect(&" ".repeat(4 * 1024 * 1024 + 1)),
        (
            1,
            "".into(),
            "Error: entrada de snapshots demasiado grande\n".into()
        )
    );
}
