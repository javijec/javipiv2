use javipiv2::core::catalog::profile_modules;
use javipiv2::core::installation_state::InstallationState;
use javipiv2::core::minimal_simulation::MinimalSimulation;
use javipiv2::core::simulated_progress::{Event, SimulatedProgress};

#[test]
fn plan_y_ejecucion_completa_son_simulados() {
    let modules = profile_modules("minimal").unwrap();
    let mut simulation = MinimalSimulation::new().unwrap();
    assert_eq!(
        simulation.ids(),
        modules.iter().map(|m| m.id).collect::<Vec<_>>()
    );
    simulation.run().unwrap();
    let expected: Vec<_> = modules
        .iter()
        .flat_map(|m| [Event::Started(m.id.into()), Event::Completed(m.id.into())])
        .collect();
    assert_eq!(simulation.events(), expected);
    for module in modules {
        assert_eq!(
            simulation.state(module.id),
            Some(InstallationState::Completed)
        );
    }
    let result = simulation.report();
    assert!(result.contains("simulad"));
    assert!(result.contains("ninguna"));
}

#[test]
fn conserva_fallo_y_omision_y_reconcilia_running() {
    let modules = profile_modules("minimal").unwrap();
    let ids: Vec<_> = modules.iter().map(|m| m.id).collect();
    assert!(ids.len() >= 3);
    let mut p = SimulatedProgress::new(ids.iter().copied()).unwrap();
    p.skip(ids[0]).unwrap();
    p.start(ids[1]).unwrap();
    p.fail(ids[1], "fallo simulado").unwrap();
    p.start(ids[2]).unwrap();
    let dir = std::env::temp_dir().join(format!("javipiv2-f1d-failure-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("checkpoint.json");
    p.save_checkpoint(&file).unwrap();
    let mut resumed = MinimalSimulation::from_checkpoint(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(resumed.state(ids[2]), Some(InstallationState::Pending));
    resumed.run().unwrap();
    assert_eq!(resumed.state(ids[0]), Some(InstallationState::Skipped));
    assert!(!resumed.events().contains(&Event::Started(ids[0].into())));
    assert_eq!(resumed.state(ids[1]), Some(InstallationState::Failed));
    assert!(resumed
        .events()
        .contains(&Event::Failed(ids[1].into(), "fallo simulado".into())));
    assert!(resumed.report().contains("error simulado: fallo simulado"));
    assert_eq!(
        resumed
            .events()
            .iter()
            .filter(|e| **e == Event::Started(ids[1].into()))
            .count(),
        1
    );
    assert_eq!(resumed.state(ids[2]), Some(InstallationState::Completed));
}

#[test]
fn reanuda_sin_repetir_y_rechaza_checkpoint_invalido() {
    let mut continuous = MinimalSimulation::new().unwrap();
    continuous.run().unwrap();
    let mut interrupted = MinimalSimulation::new().unwrap();
    let first = interrupted.step().unwrap().unwrap();
    let checkpoint = interrupted.checkpoint().unwrap();
    let dir = std::env::temp_dir().join(format!("javipiv2-f1d-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("checkpoint.json");
    std::fs::write(&file, &checkpoint).unwrap();
    let mut restored = MinimalSimulation::from_checkpoint(&std::fs::read(&file).unwrap()).unwrap();
    restored.run().unwrap();
    assert_eq!(
        restored
            .events()
            .iter()
            .filter(|event| **event == Event::Started(first.to_owned()))
            .count(),
        1
    );
    for id in continuous.ids() {
        assert_eq!(continuous.state(id), restored.state(id));
    }
    assert!(MinimalSimulation::from_checkpoint(b"not json").is_err());
    let wrong = SimulatedProgress::new(["other"]).unwrap();
    wrong.save_checkpoint(&file).unwrap();
    assert!(MinimalSimulation::from_checkpoint(&std::fs::read(&file).unwrap()).is_err());
}
