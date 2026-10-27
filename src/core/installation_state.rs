//! Estado de instalación informado explícitamente para un componente.
//! No implica que se haya ejecutado ninguna operación.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}
