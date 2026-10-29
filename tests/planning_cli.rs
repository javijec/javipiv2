use std::process::Command;

fn call(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn planning_cli_minimal_exact_and_errors() {
    let output = call(&["simulate", "plan", "minimal"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "Plan simulado: minimal (sin consultas al host; no autoriza acciones)\n",
            "core: dependencias no verificadas: ninguna\n",
            "core: instalación indefinida y bloqueada\n",
            "health: dependencias no verificadas: core\n",
            "health: instalación indefinida y bloqueada\n",
            "storage: dependencias no verificadas: core\n",
            "storage: instalación indefinida y bloqueada\n",
            "services: dependencias no verificadas: core\n",
            "services: instalación indefinida y bloqueada\n",
        )
    );
    assert_eq!(output.stderr, b"");
    let output = call(&["simulate", "plan", "invalid"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"Error: perfil desconocido: invalid\n");
}

#[test]
fn planning_cli_managed_and_existing_service_plan() {
    let output = call(&["service", "managed"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, "Declaraciones administradas: vacías. No prueban propiedad en el host ni autorizan acciones.\n".as_bytes());
    assert_eq!(output.stderr, b"");
    let output = call(&[
        "service",
        "plan",
        "start",
        "demo.service",
        "--allow",
        "demo.service",
    ]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        "Plan solicitado (no ejecutado): start en demo.service\n".as_bytes()
    );
    assert_eq!(output.stderr, b"");
}

#[test]
fn planning_cli_preserves_simulate_minimal_fixture() {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["simulate", "minimal", "--fixture"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"unknown\nunknown\nunknown\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Progreso simulado: core completado (no instalado)"));
}

#[test]
fn planning_cli_other_profiles_are_deterministic() {
    for profile in ["developer", "full"] {
        let first = call(&["simulate", "plan", profile]);
        let second = call(&["simulate", "plan", profile]);
        assert_eq!(first.status.code(), Some(0));
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, b"");
        assert!(String::from_utf8(first.stdout)
            .unwrap()
            .contains("instalación indefinida y bloqueada"));
    }
}
