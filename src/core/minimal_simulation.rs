//! Integración del perfil Minimal exclusivamente para simulaciones.
use super::catalog::profile_modules;
use super::installation_state::InstallationState;
use super::simulated_progress::{Event, SimulatedProgress};

pub struct MinimalSimulation {
    ids: Vec<&'static str>,
    progress: SimulatedProgress,
}

impl MinimalSimulation {
    pub fn new() -> Result<Self, String> {
        let ids = Self::plan()?;
        let progress = SimulatedProgress::new(ids.iter().copied())?;
        Ok(Self { ids, progress })
    }

    fn plan() -> Result<Vec<&'static str>, String> {
        Ok(profile_modules("minimal")?
            .into_iter()
            .map(|module| module.id)
            .collect())
    }

    /// Reconstruye desde bytes, nunca desde una ruta proporcionada por el usuario.
    pub fn from_checkpoint(bytes: &[u8]) -> Result<Self, String> {
        let ids = Self::plan()?;
        let progress = SimulatedProgress::from_checkpoint_bytes(bytes, ids.iter().copied())?;
        Ok(Self { ids, progress })
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>, String> {
        self.progress.checkpoint_bytes()
    }

    pub fn ids(&self) -> &[&'static str] {
        &self.ids
    }
    pub fn state(&self, id: &str) -> Option<InstallationState> {
        self.progress.state(id)
    }
    pub fn events(&self) -> &[Event] {
        self.progress.events()
    }

    /// Una operación simulada por llamada; no ejecuta acciones sobre el host.
    pub fn step(&mut self) -> Result<Option<&'static str>, String> {
        if self.progress.active().is_some() {
            return Err("operación simulada activa; se requiere reconciliación".into());
        }
        let Some(id) = self
            .ids
            .iter()
            .copied()
            .find(|id| self.progress.state(id) == Some(InstallationState::Pending))
        else {
            return Ok(None);
        };
        self.progress.start(id)?;
        self.progress.complete(id)?;
        Ok(Some(id))
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.step()?.is_some() {}
        Ok(())
    }

    pub fn report(&self) -> String {
        let mut result = String::from("Simulación Minimal; operaciones reales ejecutadas: ninguna. Instalación y activación: desconocidas.\n");
        for id in &self.ids {
            result.push_str(&format!(
                "{id}: {:?} (simulado)",
                self.progress.state(id).expect("plan validado")
            ));
            if let Some(error) = self
                .progress
                .events()
                .iter()
                .rev()
                .find_map(|event| match event {
                    Event::Failed(failed_id, error) if failed_id == id => Some(error),
                    _ => None,
                })
            {
                result.push_str(&format!("; error simulado: {error}"));
            }
            result.push('\n');
        }
        result
    }
}
