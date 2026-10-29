use javipiv2::core::{install_plan::plan_profile, simulation_sandbox::Sandbox};
use serde_json::{json, Value};

#[test]
fn checkpoint_replay_is_deterministic_and_final_step_is_idempotent() {
    for profile in ["minimal", "developer", "full"] {
        let plan = plan_profile(profile).unwrap();
        let mut original = Sandbox::new(&plan).unwrap();
        original.step();
        let bytes = original.checkpoint().unwrap();
        assert_eq!(bytes, original.checkpoint().unwrap());
        let mut restored = Sandbox::from_checkpoint(&bytes).unwrap();
        assert_eq!(original.effects(), restored.effects());
        while original.step().is_some() {}
        while restored.step().is_some() {}
        assert_eq!(original.effects(), restored.effects());
        let final_bytes = restored.checkpoint().unwrap();
        assert_eq!(restored.step(), None);
        assert_eq!(final_bytes, restored.checkpoint().unwrap());
    }
}

#[test]
fn rejects_invalid_and_forged_checkpoints() {
    let sandbox = Sandbox::new(&plan_profile("minimal").unwrap()).unwrap();
    let bytes = sandbox.checkpoint().unwrap();
    let base: Value = serde_json::from_slice(&bytes).unwrap();
    for (key, value) in [
        ("version", json!(2)),
        ("profile", json!("unknown")),
        ("identity", json!([])),
        ("cursor", json!(999999)),
        ("effects", json!([])),
    ] {
        let mut changed = base.clone();
        changed[key] = value;
        assert!(
            Sandbox::from_checkpoint(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "{key}"
        );
    }
    assert!(Sandbox::from_checkpoint(b"{").is_err());
    assert!(Sandbox::from_checkpoint(&vec![b' '; 1024 * 1024]).is_err());
    let mut plan = plan_profile("minimal").unwrap();
    plan.steps[0].module.push('x');
    assert!(Sandbox::new(&plan).is_err());
}
