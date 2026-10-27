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
        "Progreso simulado: core completado (no instalado)",
        "Progreso simulado: health completado (no instalado)",
        "Progreso simulado: storage completado (no instalado)",
        "Progreso simulado: services completado (no instalado)",
        "operaciones reales ejecutadas: ninguna",
        "Instalación y activación: desconocidas",
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
    assert!(
        text.find("Progreso simulado: core").unwrap()
            < text.find("Progreso simulado: health").unwrap()
    );
    assert!(
        text.find("Progreso simulado: health").unwrap()
            < text.find("Progreso simulado: storage").unwrap()
    );
    assert!(
        text.find("Progreso simulado: storage").unwrap()
            < text.find("Progreso simulado: services").unwrap()
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
fn simulation_storage_remains_unknown_for_both_fixture_states() {
    for (input, expected) in [
        (
            b"unknown\nunknown\nmounted\n".as_slice(),
            "montaje informado por fixture; capacidad desconocida",
        ),
        (
            b"unknown\nunknown\nunknown\n".as_slice(),
            "Almacenamiento: desconocido",
        ),
    ] {
        let output = simulate(input);
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(expected));
        assert!(!text.contains("disco ausente"));
        assert!(!text.contains("capacidad: 0"));
    }
}

#[test]
fn simulation_details_each_ordered_module_without_claiming_installation() {
    let out = simulate(b"unknown\nunknown\nunknown\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stderr, b"");
    let text = String::from_utf8(out.stdout).unwrap();
    let expected = [
        "1. core: código de catálogo y CLI disponible en este programa; presencia instalada desconocida; instalación sin definir",
        "2. health: consulta de salud disponible en este programa; presencia instalada desconocida; instalación sin definir",
        "3. storage: funcionalidad pendiente; presencia instalada desconocida; instalación sin definir",
        "4. services: funcionalidad pendiente; presencia instalada desconocida; instalación sin definir",
    ];
    let mut last = 0;
    for line in expected {
        let pos = text
            .find(line)
            .unwrap_or_else(|| panic!("falta {line}: {text}"));
        assert!(pos >= last);
        last = pos + line.len();
    }
}

#[test]
fn simulation_rejects_invalid_input_without_output() {
    let out = simulate(b"invalid\n");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"");
    assert_eq!(out.stderr, "Error: fixture inválida\n".as_bytes());
}
