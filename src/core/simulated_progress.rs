//! Progreso de operaciones simuladas en memoria; nunca ejecuta instalaciones.
use super::installation_state::InstallationState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Started(String),
    Completed(String),
    Failed(String, String),
    Skipped(String),
}

#[derive(Debug)]
struct Operation {
    id: String,
    state: InstallationState,
}

#[derive(Debug)]
pub struct SimulatedProgress {
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
            Event::Started(_) => None,
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
