//! Declaraciones de unidades de JaviPiV2, no evidencia del host ni autorización.

/// Origen de una declaración mantenida en el código del proyecto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    JaviPiV2Declarative,
}

impl Provenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JaviPiV2Declarative => "JaviPiV2 declarative",
        }
    }
}

/// Declaración estática; no acredita instalación ni propiedad real en un host.
#[derive(Debug, PartialEq, Eq)]
pub struct ManagedUnit {
    name: &'static str,
    provenance: Provenance,
}

impl ManagedUnit {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn provenance(&self) -> Provenance {
        self.provenance
    }
}

// Ninguna unidad tiene todavía evidencia suficiente para declararla aquí.
const DECLARATIONS: &[ManagedUnit] = &[];

/// Vista inmutable de las declaraciones, independiente de toda política de acciones.
#[derive(Debug, Default)]
pub struct ManagedUnitRegistry;

impl ManagedUnitRegistry {
    pub fn units(&self) -> &'static [ManagedUnit] {
        DECLARATIONS
    }

    pub fn contains(&self, name: &str) -> bool {
        self.units().iter().any(|unit| unit.name == name)
    }
}
