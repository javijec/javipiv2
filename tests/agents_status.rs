use agents::Probe;
use javipiv2::modules::agents;

#[test]
fn executable_evidence_is_fixed_conservative_and_independent_of_version() {
    let mut paths = Vec::new();
    let result = agents::status_with_probes(
        |_| Some(br#"{"version":"1.2.3"}"#.to_vec()),
        |path| {
            paths.push(path.to_owned());
            if path == "/usr/bin/pi" || path == "/usr/local/bin/engram" {
                Probe::Executable
            } else {
                Probe::Absent
            }
        },
    );
    assert_eq!(paths.len(), 12);
    assert_eq!(result[0].line(), "Pi Agent: ejecutable=/usr/bin/pi (archivo regular con bits de ejecución; acceso no garantizado), versión npm=1.2.3 (no vinculada al ejecutable), estado=desconocido");
    assert!(result[3]
        .line()
        .contains("ejecutable=/usr/local/bin/engram"));
    assert!(result[3].line().contains("versión npm=desconocido"));
}

#[test]
fn unsafe_and_conflicting_probes_stay_unknown() {
    for problem in [Probe::Unknown, Probe::Executable] {
        let result = agents::status_with_probes(
            |_| None,
            |path| {
                if path == "/usr/bin/codex" {
                    Probe::Executable
                } else if path == "/usr/local/bin/codex" {
                    problem
                } else {
                    Probe::Absent
                }
            },
        );
        assert!(result[2].line().contains("ejecutable=desconocido"));
    }
    let result = agents::status_with_probes(|_| None, |_| Probe::Unknown);
    assert!(result
        .iter()
        .all(|o| o.line().contains("ejecutable=desconocido")));
}

#[test]
fn synthetic_metadata_is_not_executable_or_runtime_evidence() {
    let mut paths = Vec::new();
    let observations = agents::status_with(|path| {
        paths.push(path.to_owned());
        if path == "/usr/lib/node_modules/gentle-pi/package.json" {
            Some(br#"{"version":"1.2.3"}"#.to_vec())
        } else {
            None
        }
    });
    assert_eq!(paths.len(), 12);
    assert_eq!(
        observations.iter().map(|o| o.name).collect::<Vec<_>>(),
        ["Pi Agent", "Gentle Shell", "Codex", "Engram"]
    );
    assert_eq!(
        observations[1].line(),
        "Gentle Shell: ejecutable=desconocido, versión npm=1.2.3 (no vinculada al ejecutable), estado=desconocido"
    );
    assert_eq!(
        observations[3].line(),
        "Engram: ejecutable=desconocido, versión npm=desconocido (no vinculada al ejecutable), estado=desconocido"
    );
}

#[test]
fn conflicting_or_invalid_metadata_is_unknown() {
    let observations = agents::status_with(|path| {
        if path.contains("@openai/codex") {
            if path.starts_with("/usr/lib/") {
                Some(br#"{"version":"1.0.0"}"#.to_vec())
            } else if path.starts_with("/usr/local/") {
                Some(br#"{"version":"2.0.0"}"#.to_vec())
            } else {
                None
            }
        } else if path.contains("gentle-pi") {
            Some(b"invalid".to_vec())
        } else {
            None
        }
    });
    assert!(observations.iter().all(|o| o.version.is_none()));
}
