use javipiv2::modules::agents;

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
        "Gentle Shell: disponibilidad=desconocido, versión=1.2.3, estado=desconocido"
    );
    assert_eq!(
        observations[3].line(),
        "Engram: disponibilidad=desconocido, versión=desconocido, estado=desconocido"
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
