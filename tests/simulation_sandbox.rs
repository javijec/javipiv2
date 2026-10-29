use javipiv2::core::{
    install_plan::{plan_profile, Operation},
    simulation_sandbox::{Sandbox, StepEffect},
};

#[test]
fn canonical_plans_produce_exact_virtual_effects() {
    for profile in ["minimal", "developer", "full"] {
        let plan = plan_profile(profile).unwrap();
        let mut sandbox = Sandbox::new(&plan).unwrap();
        assert!(sandbox.effects().is_empty());
        for step in &plan.steps {
            let effect = match &step.operation {
                Operation::VerifyDependencies(dependencies) => StepEffect::DependenciesUnverified {
                    module: step.module.clone(),
                    dependencies: dependencies.clone(),
                },
                Operation::InstallationUndefinedBlocked => StepEffect::InstallationBlocked {
                    module: step.module.clone(),
                },
            };
            assert_eq!(sandbox.step(), Some(effect));
        }
        assert_eq!(sandbox.step(), None);
        assert_eq!(sandbox.effects().len(), plan.steps.len());
    }
}

#[test]
fn fresh_sandboxes_are_deterministic_and_independent() {
    let plan = plan_profile("minimal").unwrap();
    let mut first = Sandbox::new(&plan).unwrap();
    let mut second = Sandbox::new(&plan).unwrap();
    first.step();
    assert!(second.effects().is_empty());
    while first.step().is_some() {}
    while second.step().is_some() {}
    assert_eq!(first.effects(), second.effects());
    assert_eq!(plan, plan_profile("minimal").unwrap());
}

#[test]
fn altered_or_unknown_plans_are_rejected() {
    let original = plan_profile("minimal").unwrap();
    let mut variants = Vec::new();
    let mut changed = original.clone();
    changed.steps[0].module.push('x');
    variants.push(changed);
    let mut changed = original.clone();
    changed.steps.reverse();
    variants.push(changed);
    let mut changed = original.clone();
    changed.steps[0].operation = Operation::VerifyDependencies(vec!["forged".into()]);
    variants.push(changed);
    let mut changed = original.clone();
    changed.profile = "unknown".into();
    variants.push(changed);
    let mut changed = original.clone();
    changed.profile = "full".into();
    variants.push(changed);
    for plan in variants {
        assert!(Sandbox::new(&plan).is_err());
    }
}
