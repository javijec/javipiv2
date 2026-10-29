//! Isolated, in-memory simulation of canonical declarative installation plans.
use super::install_plan::{plan_profile, InstallPlan, Operation};

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
