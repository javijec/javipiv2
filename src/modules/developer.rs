//! Comparación offline de versiones suministradas por el usuario.
use super::updates::{self, Comparison};

pub fn compare(installed: &str, reference: &str) -> Comparison {
    fn valid(value: &str) -> bool {
        !value.starts_with('v') && updates::parse_version(value.as_bytes()).is_some()
    }
    if !valid(installed) || !valid(reference) {
        return Comparison::Unknown;
    }
    updates::compare(Some(installed), Some(reference))
}
