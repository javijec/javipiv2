use javipiv2::modules::managed_units::ManagedUnitRegistry;
use javipiv2::modules::services::{
    execute_service, ExecutionPolicy, PlanAllowlist, ServiceOperation,
};

#[test]
fn registry_is_empty_stable_and_does_not_infer_ownership() {
    let registry = ManagedUnitRegistry;
    assert!(registry.units().is_empty());
    assert_eq!(registry.units(), ManagedUnitRegistry.units());
    for name in [
        "",
        "ssh.service",
        "docker.service",
        "javipiv2-lookalike.service",
    ] {
        assert!(!registry.contains(name));
    }
}

#[test]
fn registry_does_not_authorize_planning_or_execution() {
    let registry = ManagedUnitRegistry;
    assert!(!registry.contains("ssh.service"));
    assert_eq!(
        PlanAllowlist::default().plan(ServiceOperation::Start, "ssh.service"),
        Err("servicio no autorizado para planificación".into())
    );
    let plan = PlanAllowlist::new(["ssh.service".to_owned()])
        .unwrap()
        .plan(ServiceOperation::Start, "ssh.service")
        .unwrap();
    assert_eq!(
        ExecutionPolicy.authorize(&plan),
        Err("ejecución de servicios deshabilitada")
    );
    let mut calls = 0;
    assert_eq!(
        execute_service(&plan, &mut |_| {
            calls += 1;
            Ok(true)
        }),
        Err("ejecución de servicios deshabilitada".into())
    );
    assert_eq!(calls, 0);
}
