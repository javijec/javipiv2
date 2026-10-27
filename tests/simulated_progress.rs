use javipiv2::core::installation_state::InstallationState;
use javipiv2::core::simulated_progress::{Event, SimulatedProgress};

#[test]
fn registra_progreso_en_orden_y_cuenta_todos_los_estados() {
    let mut p = SimulatedProgress::new(["one", "two", "three", "four", "five"]).unwrap();
    assert_eq!(p.count(InstallationState::Pending), 5);
    assert_eq!(p.active(), None);
    assert_eq!(p.last_finished(), None);
    p.start("one").unwrap();
    assert_eq!(p.active(), Some("one"));
    assert_eq!(p.state("one"), Some(InstallationState::Running));
    assert_eq!(p.state("missing"), None);
    assert_eq!(p.count(InstallationState::Running), 1);
    p.complete("one").unwrap();
    p.start("two").unwrap();
    p.complete("two").unwrap();
    p.start("three").unwrap();
    p.fail("three", "simulated error").unwrap();
    p.skip("four").unwrap();
    assert_eq!(p.active(), None);
    assert_eq!(p.last_finished(), Some("four"));
    assert_eq!(p.state("three"), Some(InstallationState::Failed));
    assert_eq!(p.state("four"), Some(InstallationState::Skipped));
    for state in [
        InstallationState::Pending,
        InstallationState::Completed,
        InstallationState::Failed,
        InstallationState::Skipped,
    ] {
        assert_eq!(
            p.count(state),
            if state == InstallationState::Completed {
                2
            } else {
                1
            }
        );
    }
    assert_eq!(p.count(InstallationState::Running), 0);
    assert_eq!(
        p.events(),
        &[
            Event::Started("one".into()),
            Event::Completed("one".into()),
            Event::Started("two".into()),
            Event::Completed("two".into()),
            Event::Started("three".into()),
            Event::Failed("three".into(), "simulated error".into()),
            Event::Skipped("four".into())
        ]
    );
}

#[test]
fn rechaza_entradas_y_transiciones_sin_modificar_estado() {
    assert!(SimulatedProgress::new(["", "valid"]).is_err());
    assert!(SimulatedProgress::new(["duplicate", "duplicate"]).is_err());
    let mut p = SimulatedProgress::new(["one", "two"]).unwrap();
    assert!(p.start("missing").is_err());
    assert!(p.complete("one").is_err());
    assert!(p.fail("one", "error").is_err());
    assert!(p.events().is_empty());
    assert_eq!(p.count(InstallationState::Pending), 2);
    p.start("one").unwrap();
    assert!(p.start("two").is_err());
    assert!(p.fail("one", "").is_err());
    assert!(p.skip("one").is_err());
    assert_eq!(p.active(), Some("one"));
    assert_eq!(p.events(), &[Event::Started("one".into())]);
    p.complete("one").unwrap();
    assert!(p.start("one").is_err());
    assert_eq!(p.last_finished(), Some("one"));
}
