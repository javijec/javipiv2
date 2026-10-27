use javipiv2::core::installation_state::InstallationState;
use javipiv2::core::simulated_progress::{Event, SimulatedProgress};
use std::path::PathBuf;

fn isolated() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "javipiv2-checkpoint-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    path.join("checkpoint.json")
}

#[test]
fn conserva_estado_y_reconcilia_operacion_interrumpida() {
    let path = isolated();
    let plan = ["one", "two", "three", "four", "five"];
    let mut p = SimulatedProgress::new(plan).unwrap();
    p.start("one").unwrap();
    p.complete("one").unwrap();
    p.start("two").unwrap();
    p.complete("two").unwrap();
    p.start("three").unwrap();
    p.fail("three", "simulated failure").unwrap();
    p.skip("four").unwrap();
    p.start("five").unwrap();
    p.save_checkpoint(&path).unwrap();
    let mut restored = SimulatedProgress::load_checkpoint(&path, plan).unwrap();
    assert_eq!(restored.state("one"), Some(InstallationState::Completed));
    assert_eq!(restored.state("two"), Some(InstallationState::Completed));
    assert_eq!(restored.state("three"), Some(InstallationState::Failed));
    assert_eq!(restored.state("four"), Some(InstallationState::Skipped));
    assert!(restored.start("four").is_err());
    assert_eq!(restored.state("five"), Some(InstallationState::Pending));
    assert!(restored
        .events()
        .contains(&Event::Failed("three".into(), "simulated failure".into())));
    assert!(restored.start("one").is_err());
    assert!(restored.start("three").is_err());
    restored.start("five").unwrap();
    restored.complete("five").unwrap();
    restored.save_checkpoint(&path).unwrap();
    let again = SimulatedProgress::load_checkpoint(&path, plan).unwrap();
    assert_eq!(again.state("five"), Some(InstallationState::Completed));
    assert_eq!(again.count(InstallationState::Completed), 3);
}

#[test]
fn rechaza_historial_y_operaciones_invalidas_sin_cambiar_el_archivo() {
    let path = isolated();
    let progress = SimulatedProgress::new(["one", "two"]).unwrap();
    progress.save_checkpoint(&path).unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let mut cases = Vec::new();
    let mut duplicate = original.clone();
    duplicate["progress"]["operations"][1]["id"] = "one".into();
    cases.push(duplicate);
    let mut invalid = original.clone();
    invalid["progress"]["operations"][0]["id"] = "INVALID".into();
    cases.push(invalid);
    let mut unknown = original.clone();
    unknown["progress"]["operations"][0]["id"] = "other".into();
    cases.push(unknown);
    let mut impossible = original.clone();
    impossible["progress"]["events"] = serde_json::json!([{ "Completed": "one" }]);
    cases.push(impossible);
    let mut empty_error = original.clone();
    empty_error["progress"]["events"] =
        serde_json::json!([{ "Started": "one" }, { "Failed": ["one", "  "] }]);
    empty_error["progress"]["operations"][0]["state"] = "Failed".into();
    cases.push(empty_error);
    let mut unknown_event = original.clone();
    unknown_event["progress"]["events"] = serde_json::json!([{ "Started": "other" }]);
    cases.push(unknown_event);
    for case in cases {
        let bytes = serde_json::to_vec(&case).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(SimulatedProgress::load_checkpoint(&path, ["one", "two"]).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    assert!(SimulatedProgress::new(["one", "one"]).is_err());
    assert!(SimulatedProgress::new(["INVALID"]).is_err());
}

#[test]
fn rechaza_datos_corruptos_y_plan_diferente() {
    let path = isolated();
    let p = SimulatedProgress::new(["one", "two"]).unwrap();
    p.save_checkpoint(&path).unwrap();
    assert!(SimulatedProgress::load_checkpoint(&path, ["two", "one"]).is_err());
    let original = std::fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["version"] = 2.into();
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(SimulatedProgress::load_checkpoint(&path, ["one", "two"]).is_err());
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["progress"]["operations"][0]["state"] = "Completed".into();
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(SimulatedProgress::load_checkpoint(&path, ["one", "two"]).is_err());
    std::fs::write(&path, b"{\"version\":1}").unwrap();
    assert!(SimulatedProgress::load_checkpoint(&path, ["one", "two"]).is_err());
    std::fs::write(&path, b"not json").unwrap();
    assert!(SimulatedProgress::load_checkpoint(&path, ["one", "two"]).is_err());
}
