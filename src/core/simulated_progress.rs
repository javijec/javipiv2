//! Progreso de operaciones simuladas en memoria; nunca ejecuta instalaciones.
use super::installation_state::InstallationState;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Started(String),
    Completed(String),
    Failed(String, String),
    Skipped(String),
    /// Solo reconcilia una interrupción simulada; no representa operaciones reales.
    Interrupted(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    id: String,
    state: InstallationState,
}

/// El estado validado no admite deserialización directa.
///
/// ```compile_fail
/// use javipiv2::core::simulated_progress::SimulatedProgress;
/// let _: SimulatedProgress = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, serde::Serialize)]
pub struct SimulatedProgress {
    operations: Vec<Operation>,
    events: Vec<Event>,
}

#[derive(serde::Serialize)]
struct Checkpoint<'a> {
    version: u32,
    progress: &'a SimulatedProgress,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnedCheckpoint {
    version: u32,
    progress: CheckpointProgress,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointProgress {
    operations: Vec<Operation>,
    events: Vec<Event>,
}

impl SimulatedProgress {
    pub fn new<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<Self, String> {
        let mut operations: Vec<Operation> = Vec::new();
        for id in ids {
            if id.is_empty()
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return Err(format!("identificador inválido: {id}"));
            }
            if operations.iter().any(|op| op.id == id) {
                return Err(format!("identificador duplicado: {id}"));
            }
            operations.push(Operation {
                id: id.to_owned(),
                state: InstallationState::Pending,
            });
        }
        Ok(Self {
            operations,
            events: Vec::new(),
        })
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn state(&self, id: &str) -> Option<InstallationState> {
        self.operations
            .iter()
            .find(|op| op.id == id)
            .map(|op| op.state)
    }
    pub fn count(&self, state: InstallationState) -> usize {
        self.operations
            .iter()
            .filter(|op| op.state == state)
            .count()
    }
    pub fn active(&self) -> Option<&str> {
        self.operations
            .iter()
            .find(|op| op.state == InstallationState::Running)
            .map(|op| op.id.as_str())
    }
    pub fn last_finished(&self) -> Option<&str> {
        self.events.iter().rev().find_map(|event| match event {
            Event::Completed(id) | Event::Failed(id, _) | Event::Skipped(id) => Some(id.as_str()),
            Event::Started(_) | Event::Interrupted(_) => None,
        })
    }
    pub fn start(&mut self, id: &str) -> Result<(), String> {
        if self.active().is_some() {
            return Err("ya hay una operación activa".into());
        }
        self.transition(
            id,
            InstallationState::Pending,
            InstallationState::Running,
            Event::Started(id.into()),
        )
    }
    pub fn complete(&mut self, id: &str) -> Result<(), String> {
        self.transition(
            id,
            InstallationState::Running,
            InstallationState::Completed,
            Event::Completed(id.into()),
        )
    }
    pub fn fail(&mut self, id: &str, error: &str) -> Result<(), String> {
        if error.trim().is_empty() {
            return Err("error simulado vacío".into());
        }
        self.transition(
            id,
            InstallationState::Running,
            InstallationState::Failed,
            Event::Failed(id.into(), error.into()),
        )
    }
    pub fn skip(&mut self, id: &str) -> Result<(), String> {
        self.transition(
            id,
            InstallationState::Pending,
            InstallationState::Skipped,
            Event::Skipped(id.into()),
        )
    }
    /// Guarda únicamente progreso simulado; rename no garantiza durabilidad ante cortes de energía.
    pub fn save_checkpoint(&self, path: &std::path::Path) -> Result<(), String> {
        self.save_checkpoint_with(path, |file, bytes| {
            use std::io::Write;
            file.write_all(bytes)?;
            file.flush()
        })
    }

    fn save_checkpoint_with(
        &self,
        path: &std::path::Path,
        write: impl FnOnce(&mut std::fs::File, &[u8]) -> std::io::Result<()>,
    ) -> Result<(), String> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let bytes = self.checkpoint_bytes()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        let file_name = path.file_name().ok_or("ruta de checkpoint inválida")?;
        let mut temporary_name = file_name.to_os_string();
        temporary_name.push(format!(
            ".tmp-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let temporary = parent.join(temporary_name);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        write(&mut file, &bytes).map_err(|e| e.to_string())?;
        std::fs::rename(temporary, path).map_err(|e| e.to_string())
    }

    /// Serializa el progreso simulado sin elegir ni abrir rutas.
    pub fn checkpoint_bytes(&self) -> Result<Vec<u8>, String> {
        let bytes = serde_json::to_vec(&Checkpoint {
            version: 1,
            progress: self,
        })
        .map_err(|e| e.to_string())?;
        if bytes.len() > 1_048_576 {
            return Err("checkpoint demasiado grande".into());
        }
        Ok(bytes)
    }

    /// Reconstruye y valida todo antes de devolver el progreso simulado.
    pub fn load_checkpoint<'a>(
        path: &std::path::Path,
        plan: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, String> {
        use std::io::Read;
        const LIMIT: u64 = 1_048_576;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        Self::from_checkpoint_bytes(&bytes, plan)
    }

    /// Valida datos externos sin aceptar ni abrir una ruta de archivo.
    pub fn from_checkpoint_bytes<'a>(
        bytes: &[u8],
        plan: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, String> {
        if bytes.len() > 1_048_576 {
            return Err("checkpoint demasiado grande".into());
        }
        let checkpoint: OwnedCheckpoint =
            serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if checkpoint.version != 1 {
            return Err("versión de checkpoint incompatible".into());
        }
        let mut recovered = Self::new(plan)?;
        if checkpoint.progress.operations.len() != recovered.operations.len()
            || checkpoint
                .progress
                .operations
                .iter()
                .zip(&recovered.operations)
                .any(|(a, b)| a.id != b.id)
        {
            return Err("plan incompatible".into());
        }
        for event in &checkpoint.progress.events {
            match event {
                Event::Started(id) => recovered.start(id)?,
                Event::Completed(id) => recovered.complete(id)?,
                Event::Failed(id, error) => recovered.fail(id, error)?,
                Event::Skipped(id) => recovered.skip(id)?,
                Event::Interrupted(id) => recovered.transition(
                    id,
                    InstallationState::Running,
                    InstallationState::Pending,
                    event.clone(),
                )?,
            }
        }
        if checkpoint
            .progress
            .operations
            .iter()
            .zip(&recovered.operations)
            .any(|(a, b)| a.state != b.state)
        {
            return Err("estados y eventos incompatibles".into());
        }
        if let Some(id) = recovered.active().map(str::to_owned) {
            recovered.transition(
                &id,
                InstallationState::Running,
                InstallationState::Pending,
                Event::Interrupted(id.clone()),
            )?;
        }
        Ok(recovered)
    }

    fn transition(
        &mut self,
        id: &str,
        expected: InstallationState,
        next: InstallationState,
        event: Event,
    ) -> Result<(), String> {
        let operation = self
            .operations
            .iter_mut()
            .find(|op| op.id == id)
            .ok_or_else(|| format!("operación desconocida: {id}"))?;
        if operation.state != expected {
            return Err(format!("transición inválida: {id}"));
        }
        operation.state = next;
        self.events.push(event);
        Ok(())
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn fallo_de_escritura_no_reemplaza_checkpoint_previo() {
        let dir = std::env::temp_dir().join(format!(
            "javipiv2-checkpoint-failure-{}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("checkpoint.json");
        let mut progress = SimulatedProgress::new(["one"]).unwrap();
        progress.save_checkpoint(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        progress.start("one").unwrap();
        let result = progress.save_checkpoint_with(&path, |file, _| {
            file.write_all(b"partial")?;
            Err(std::io::Error::other("simulated failure"))
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(
            SimulatedProgress::load_checkpoint(&path, ["one"])
                .unwrap()
                .state("one"),
            Some(InstallationState::Pending)
        );
    }
}
