use javipiv2::modules::{agents, developer};
use std::process::Command;

#[test]
fn developer_status_renders_independent_synthetic_evidence() {
    let observations = vec![
        agents::Observation {
            name: "Pi Agent",
            version: Some("1.2.3".into()),
            executable: Some("/synthetic/pi".into()),
        },
        agents::Observation {
            name: "Engram",
            version: None,
            executable: None,
        },
    ];
    let lines = developer::status_from(observations);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "Pi Agent: ejecutable=/synthetic/pi (ruta consultada; destino regular con bits de ejecución; acceso no garantizado), versión npm=1.2.3 (fuente: metadatos npm locales; no vinculada al ejecutable), disponibilidad efectiva=desconocido, estado de ejecución=desconocido");
    assert_eq!(lines[1], "Engram: ejecutable=desconocido, versión npm=desconocido (fuente: metadatos npm locales; no vinculada al ejecutable), disponibilidad efectiva=desconocido, estado de ejecución=desconocido");
}

#[test]
fn developer_compare_remains_unchanged() {
    let output = Command::new(env!("CARGO_BIN_EXE_javipiv2"))
        .args(["developer", "compare", "1.2.3", "1.2.4"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"Versiones suministradas por el usuario; no acreditan instalaci\xc3\xb3n ni disponibilidad.\nResultado: primera menor\n");
    assert!(output.stderr.is_empty());
}
