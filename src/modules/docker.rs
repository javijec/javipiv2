//! Observación puntual del daemon Docker local, sin operaciones administrativas.

use std::process::Command;

const PATH: &str = "/usr/local/bin:/usr/bin:/bin";
const HOST: &str = "unix:///var/run/docker.sock";

#[derive(Debug, PartialEq, Eq)]
pub struct Container {
    pub name: String,
    pub id: String,
    pub image: String,
    pub state: String,
}

pub fn parse_containers(raw: &str) -> Result<Vec<Container>, String> {
    raw.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 4 {
                return Err("respuesta Docker inválida".into());
            }
            let value = |index| {
                serde_json::from_str::<String>(fields[index])
                    .map_err(|_| "respuesta Docker inválida".to_owned())
            };
            let name: String = value(0)?;
            let id: String = value(1)?;
            let image: String = value(2)?;
            let state: String = value(3)?;
            if name.is_empty() || id.is_empty() || image.is_empty() || state.is_empty() {
                return Err("respuesta Docker incompleta".into());
            }
            Ok(Container {
                name: name.trim_start_matches('/').to_owned(),
                id: id.chars().take(12).collect(),
                image,
                state,
            })
        })
        .collect()
}

fn command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .env_clear()
        .env("PATH", PATH)
        .env("LC_ALL", "C")
        .env("DOCKER_CONFIG", "/dev/null")
        .args(args)
        .output()
        .map_err(|error| {
            if program == "docker" && error.kind() == std::io::ErrorKind::NotFound {
                "Docker no disponible: cliente no encontrado".to_owned()
            } else {
                format!("consulta fallida: {error}")
            }
        })?;
    if !output.status.success() {
        return Err(format!(
            "consulta fallida: {program} devolvió {}",
            output.status
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| "consulta fallida: respuesta no UTF-8".into())
}

fn unit_state(unit: &str) -> Result<(Option<String>, Option<String>), String> {
    let raw = command(
        "systemctl",
        &[
            "--system",
            "--no-pager",
            "--all",
            "--property=LoadState,ActiveState",
            "show",
            "--",
            unit,
        ],
    )?;
    let mut load = None;
    let mut active = None;
    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("LoadState=") {
            load = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("ActiveState=") {
            active = Some(value.to_owned());
        }
    }
    Ok((load, active))
}

/// A state check cannot eliminate a race with changes before the socket connection.
fn preflight() -> Result<(), String> {
    command("docker", &["--version"])?;
    let (load, active) = unit_state("docker.service")?;
    if load.as_deref() == Some("not-found") {
        return Err("Docker no disponible: unidad docker.service no encontrada".into());
    }
    if load.as_deref() != Some("loaded") {
        return Err("consulta fallida: estado de docker.service desconocido".into());
    }
    match active.as_deref() {
        Some("active") => {}
        Some("inactive" | "failed") => return Err("daemon Docker detenido".into()),
        _ => return Err("consulta fallida: estado del daemon desconocido".into()),
    }
    let (load, active) = unit_state("docker.socket")?;
    if load.as_deref() == Some("not-found")
        || (load.as_deref() == Some("loaded") && active.as_deref() == Some("inactive"))
    {
        Ok(())
    } else {
        Err("consulta no segura: docker.socket activo o estado desconocido".into())
    }
}

fn docker(args: &[&str]) -> Result<String, String> {
    let mut fixed = vec!["--host", HOST, "--config", "/dev/null"];
    fixed.extend_from_slice(args);
    command("docker", &fixed)
}

pub fn status() -> Result<(), String> {
    preflight()?;
    docker(&["info", "--format", "{{.ServerVersion}}"])?;
    Ok(())
}

pub fn list() -> Result<Vec<Container>, String> {
    preflight()?;
    let raw = docker(&[
        "ps",
        "--all",
        "--no-trunc",
        "--format",
        "{{json .Names}}\t{{json .ID}}\t{{json .Image}}\t{{json .State}}",
    ])?;
    parse_containers(&raw)
}

pub fn inspect(name: &str) -> Result<Container, String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        || name.starts_with('-')
    {
        return Err("nombre de contenedor inválido".into());
    }
    preflight()?;
    let raw = docker(&[
        "inspect",
        "--format",
        "{{json .Name}}\t{{json .Id}}\t{{json .Config.Image}}\t{{json .State.Status}}",
        "--",
        name,
    ])?;
    let mut containers = parse_containers(&raw)?;
    if containers.len() != 1 {
        return Err("consulta fallida: respuesta de contenedor inesperada".into());
    }
    Ok(containers.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_selected_container_fields() {
        let entries =
            parse_containers("\"/demo\"\t\"1234567890abcdef\"\t\"image:1\"\t\"exited\"\n")
                .unwrap();
        assert_eq!(
            entries[0],
            Container {
                name: "demo".into(),
                id: "1234567890ab".into(),
                image: "image:1".into(),
                state: "exited".into(),
            }
        );
        assert_eq!(parse_containers("").unwrap(), vec![]);
        assert!(parse_containers("bad\tdata").is_err());
    }
}
