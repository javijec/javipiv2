//! Evaluación pura de observaciones de memoria aportadas por el usuario.

#[derive(Debug, PartialEq, Eq)]
pub enum Assessment {
    Healthy,
    BelowMinimum,
    Unknown,
}

impl Assessment {
    pub fn line(&self) -> String {
        let result = match self {
            Self::Healthy => "saludable",
            Self::BelowMinimum => "por debajo del mínimo",
            Self::Unknown => "desconocido",
        };
        format!("Resultado: {result}")
    }
}

/// Consulta puntual; el umbral siempre es proporcionado por el operador.
pub fn status_with(
    minimum: &str,
    query: impl FnOnce() -> std::io::Result<super::health::SystemStatus>,
) -> Assessment {
    match query() {
        Ok(status) => assess_memory(
            &status.memory_total_kib.to_string(),
            &status.memory_available_kib.to_string(),
            minimum,
        ),
        Err(_) => Assessment::Unknown,
    }
}

pub fn status(minimum: &str) -> Assessment {
    status_with(minimum, super::health::read_status)
}

fn number(input: &str) -> Option<u64> {
    if input.is_empty() || input.len() > 20 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    input.parse().ok()
}

pub fn assess_memory(total: &str, available: &str, minimum: &str) -> Assessment {
    let (Some(total), Some(available), Some(minimum)) =
        (number(total), number(available), number(minimum))
    else {
        return Assessment::Unknown;
    };
    // Reuse Health's memory consistency validation without reading the host.
    let meminfo = format!("MemTotal: {total} kB\nMemAvailable: {available} kB");
    if minimum > 100 || super::health::parse_status("supplied", "0.0", &meminfo).is_err() {
        return Assessment::Unknown;
    }
    if u128::from(available) * 100 >= u128::from(total) * u128::from(minimum) {
        Assessment::Healthy
    } else {
        Assessment::BelowMinimum
    }
}
