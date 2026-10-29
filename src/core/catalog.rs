//! Catálogo y perfiles declarativos del proyecto.
//! Representar un módulo no significa que esté instalado o disponible.

use std::collections::{HashMap, HashSet};

pub const MODULES: &str = include_str!("../../catalog/modules.tsv");
pub const MINIMAL: &str = include_str!("../../profiles/minimal.list");
pub const DEVELOPER: &str = include_str!("../../profiles/developer.list");
pub const FULL: &str = include_str!("../../profiles/full.list");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<'a> {
    pub id: &'a str,
    pub group: &'a str,
    pub dependencies: Vec<&'a str>,
    pub state: &'a str,
    pub description: &'a str,
}

pub fn all_modules() -> Result<Vec<Module<'static>>, String> {
    parse_catalog(MODULES)
}

/// Parses and validates an entire declarative catalog without inspecting the host.
pub fn parse_catalog(source: &str) -> Result<Vec<Module<'_>>, String> {
    let mut ids = HashSet::new();
    let mut modules = Vec::new();
    for (index, row) in source.lines().enumerate() {
        if row.trim().is_empty() || row.starts_with('#') {
            continue;
        }
        let columns: Vec<_> = row.split('\t').collect();
        if columns.len() != 5 {
            return Err(format!(
                "catálogo, línea {}: se esperan 5 columnas",
                index + 1
            ));
        }
        let id = columns[0];
        if !valid_id(id) || !ids.insert(id) {
            return Err(format!("módulo duplicado o inválido: {id}"));
        }
        let (group, depends, state, description) = (columns[1], columns[2], columns[3], columns[4]);
        if group.is_empty()
            || description.trim().is_empty()
            || !matches!(state, "preview" | "planned")
        {
            return Err(format!("metadatos inválidos para: {id}"));
        }
        let dependencies = if depends == "-" {
            Vec::new()
        } else {
            depends.split(',').collect()
        };
        if dependencies.iter().any(|d| !valid_id(d) || *d == id) {
            return Err(format!("dependencias inválidas para: {id}"));
        }
        modules.push(Module {
            id,
            group,
            dependencies,
            state,
            description,
        });
    }
    for module in &modules {
        for dependency in &module.dependencies {
            if !ids.contains(dependency) {
                return Err(format!(
                    "{} depende de un módulo desconocido: {dependency}",
                    module.id
                ));
            }
        }
    }
    // Visit every component, including modules absent from any selected profile.
    let indices: HashMap<_, _> = modules.iter().enumerate().map(|(i, m)| (m.id, i)).collect();
    let mut states = vec![0u8; modules.len()];
    for start in 0..modules.len() {
        if states[start] != 0 {
            continue;
        }
        states[start] = 1;
        let mut stack = vec![(start, 0)];
        while let Some((node, next)) = stack.last_mut() {
            if *next == modules[*node].dependencies.len() {
                states[*node] = 2;
                stack.pop();
                continue;
            }
            let dependency = modules[*node].dependencies[*next];
            *next += 1;
            let target = indices[dependency];
            match states[target] {
                1 => {
                    return Err(format!(
                        "ciclo de dependencias: {} -> {dependency}",
                        modules[*node].id
                    ))
                }
                0 => {
                    states[target] = 1;
                    stack.push((target, 0));
                }
                _ => {}
            }
        }
    }
    Ok(modules)
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub fn profile_source(profile: &str) -> Option<&'static str> {
    match profile {
        "minimal" => Some(MINIMAL),
        "developer" => Some(DEVELOPER),
        "full" => Some(FULL),
        _ => None,
    }
}

pub fn profile_modules(profile: &str) -> Result<Vec<Module<'static>>, String> {
    let source = profile_source(profile).ok_or_else(|| format!("perfil desconocido: {profile}"))?;
    let registry = all_modules()?;
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for row in source.lines() {
        let id = row.trim();
        if id.is_empty() || id.starts_with('#') {
            continue;
        }
        if !seen.insert(id) {
            return Err(format!("módulo duplicado en {profile}: {id}"));
        }
        let module = registry
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| format!("módulo desconocido en {profile}: {id}"))?;
        for dependency in &module.dependencies {
            if !selected.iter().any(|m: &Module<'_>| m.id == *dependency) {
                return Err(format!(
                    "{id} requiere {dependency} antes en el perfil {profile}"
                ));
            }
        }
        selected.push(module.clone());
    }
    if selected.first().map(|m| m.id) != Some("core") {
        return Err(format!("el perfil {profile} debe comenzar por core"));
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_catalog_has_21_unique_modules() {
        let modules = all_modules().unwrap();
        assert_eq!(modules.len(), 21);
        assert_eq!(modules.iter().filter(|m| m.id == "core").count(), 1);
    }

    #[test]
    fn profiles_have_ordered_dependencies() {
        for profile in ["minimal", "developer", "full"] {
            assert!(!profile_modules(profile).unwrap().is_empty());
        }
    }

    #[test]
    fn full_profile_covers_every_catalogued_module() {
        assert_eq!(
            profile_modules("full").unwrap().len(),
            all_modules().unwrap().len()
        );
    }

    #[test]
    fn rejects_unknown_profile() {
        assert!(profile_modules("production").is_err());
    }
}
