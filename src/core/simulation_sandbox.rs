//! Isolated, in-memory simulation of canonical declarative installation plans.
use super::install_plan::{plan_profile, InstallPlan, Operation};
use serde::{Deserialize, Serialize};

const MAX_CHECKPOINT_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    version: u8,
    profile: String,
    identity: Vec<u8>,
    cursor: usize,
}

/// Virtual observations only; neither variant asserts anything about the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepEffect {
    DependenciesUnverified {
        module: String,
        dependencies: Vec<String>,
    },
    InstallationBlocked {
        module: String,
    },
}

/// An independent execution cursor and its virtual effects, with no persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sandbox {
    plan: InstallPlan,
    effects: Vec<StepEffect>,
}

impl Sandbox {
    /// Reject any plan that differs from the current canonical profile plan.
    pub fn new(plan: &InstallPlan) -> Result<Self, String> {
        let canonical = plan_profile(&plan.profile)?;
        if plan.identity() != canonical.identity() || *plan != canonical {
            return Err("plan no canónico".into());
        }
        Ok(Self {
            plan: canonical,
            effects: Vec::new(),
        })
    }

    /// Return bounded, deterministic bytes; this does not persist them.
    pub fn checkpoint(&self) -> Result<Vec<u8>, String> {
        let checkpoint = Checkpoint {
            version: 1,
            profile: self.plan.profile.clone(),
            identity: self.plan.identity(),
            cursor: self.effects.len(),
        };
        let bytes = serde_json::to_vec(&checkpoint).map_err(|_| "checkpoint inválido")?;
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err("checkpoint demasiado grande".into());
        }
        Ok(bytes)
    }

    /// Validate untrusted bytes and reconstruct effects exclusively by replay.
    pub fn from_checkpoint(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err("checkpoint demasiado grande".into());
        }
        let checkpoint: Checkpoint =
            serde_json::from_slice(bytes).map_err(|_| "checkpoint inválido")?;
        if checkpoint.version != 1 {
            return Err("checkpoint inválido".into());
        }
        let plan = plan_profile(&checkpoint.profile).map_err(|_| "checkpoint inválido")?;
        if checkpoint.identity != plan.identity() || checkpoint.cursor > plan.steps.len() {
            return Err("checkpoint inválido".into());
        }
        let mut sandbox = Self::new(&plan).map_err(|_| "checkpoint inválido")?;
        for _ in 0..checkpoint.cursor {
            sandbox.step();
        }
        Ok(sandbox)
    }

    /// Record the next virtual effect, or return None after the final step.
    pub fn step(&mut self) -> Option<StepEffect> {
        let step = self.plan.steps.get(self.effects.len())?;
        let effect = match &step.operation {
            Operation::VerifyDependencies(dependencies) => StepEffect::DependenciesUnverified {
                module: step.module.clone(),
                dependencies: dependencies.clone(),
            },
            Operation::InstallationUndefinedBlocked => StepEffect::InstallationBlocked {
                module: step.module.clone(),
            },
        };
        self.effects.push(effect.clone());
        Some(effect)
    }

    pub fn effects(&self) -> &[StepEffect] {
        &self.effects
    }
}
