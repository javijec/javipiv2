use javipiv2::core::{
    catalog,
    install_plan::{plan_profile, Operation},
};

#[test]
fn plans_all_profiles_deterministically_with_declared_operations() {
    for name in ["minimal", "developer", "full"] {
        let plan = plan_profile(name).unwrap();
        assert_eq!(plan, plan_profile(name).unwrap());
        let modules = catalog::profile_modules(name).unwrap();
        assert_eq!(plan.steps.len(), modules.len() * 2);
        for (module, steps) in modules.iter().zip(plan.steps.as_chunks::<2>().0.iter()) {
            assert_eq!(steps[0].module, module.id);
            assert_eq!(
                steps[0].operation,
                Operation::VerifyDependencies(
                    module.dependencies.iter().map(|s| s.to_string()).collect()
                )
            );
            assert_eq!(steps[1].module, module.id);
            assert_eq!(steps[1].operation, Operation::InstallationUndefinedBlocked);
        }
    }
}

#[test]
fn unknown_profile_is_rejected() {
    assert_eq!(
        plan_profile("unknown").unwrap_err(),
        "perfil desconocido: unknown"
    );
}

#[test]
fn identity_encodes_full_content_without_ambiguous_concatenation() {
    let mut plan = plan_profile("minimal").unwrap();
    let original = plan.identity();
    plan.steps[0].module.push('x');
    assert_ne!(original, plan.identity());
    let mut other = plan.clone();
    plan.steps[0].module = "ab".into();
    plan.steps[1].module = "c".into();
    other.steps[0].module = "a".into();
    other.steps[1].module = "bc".into();
    assert_ne!(plan.identity(), other.identity());
}
