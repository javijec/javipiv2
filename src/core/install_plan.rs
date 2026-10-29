//! Pure declarative plans: no host inspection or execution.
use super::catalog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    VerifyDependencies(Vec<String>),
    InstallationUndefinedBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub module: String,
    pub operation: Operation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub profile: String,
    pub steps: Vec<Step>,
}

impl InstallPlan {
    /// Canonical, versioned, length-prefixed encoding of the complete plan.
    /// This is an identity, not a cryptographic digest or authorization.
    pub fn identity(&self) -> Vec<u8> {
        fn field(output: &mut Vec<u8>, value: &str) {
            output.extend_from_slice(&(value.len() as u64).to_be_bytes());
            output.extend_from_slice(value.as_bytes());
        }
        let mut result = b"javipiv2-install-plan\0\x01".to_vec();
        field(&mut result, &self.profile);
        result.extend_from_slice(&(self.steps.len() as u64).to_be_bytes());
        for step in &self.steps {
            field(&mut result, &step.module);
            match &step.operation {
                Operation::VerifyDependencies(dependencies) => {
                    result.push(0);
                    result.extend_from_slice(&(dependencies.len() as u64).to_be_bytes());
                    for dependency in dependencies {
                        field(&mut result, dependency);
                    }
                }
                Operation::InstallationUndefinedBlocked => result.push(1),
            }
        }
        result
    }
}

/// Render deterministic virtual effects without inspecting or changing the host.
pub fn simulate_profile(profile: &str) -> Result<String, String> {
    use super::simulation_sandbox::{Sandbox, StepEffect};
    let plan = plan_profile(profile)?;
    let mut sandbox = Sandbox::new(&plan)?;
    let mut text =
        format!("Plan simulado: {profile} (sin consultas al host; no autoriza acciones)\n");
    while let Some(effect) = sandbox.step() {
        match effect {
            StepEffect::DependenciesUnverified {
                module,
                dependencies,
            } => {
                let dependencies = if dependencies.is_empty() {
                    "ninguna".into()
                } else {
                    dependencies.join(", ")
                };
                text.push_str(&format!(
                    "{module}: dependencias no verificadas: {dependencies}\n"
                ));
            }
            StepEffect::InstallationBlocked { module } => {
                text.push_str(&format!("{module}: instalación indefinida y bloqueada\n"));
            }
        }
    }
    Ok(text)
}

/// Builds a data-only plan from the validated catalog and ordered profile.
pub fn plan_profile(profile: &str) -> Result<InstallPlan, String> {
    let modules = catalog::profile_modules(profile)?;
    let mut steps = Vec::with_capacity(modules.len() * 2);
    for module in modules {
        steps.push(Step {
            module: module.id.to_owned(),
            operation: Operation::VerifyDependencies(
                module
                    .dependencies
                    .iter()
                    .map(|d| (*d).to_owned())
                    .collect(),
            ),
        });
        steps.push(Step {
            module: module.id.to_owned(),
            operation: Operation::InstallationUndefinedBlocked,
        });
    }
    Ok(InstallPlan {
        profile: profile.to_owned(),
        steps,
    })
}
