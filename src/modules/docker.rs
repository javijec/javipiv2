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

fn unit_state_with(
    run: &mut impl FnMut(&str, &[&str]) -> Result<String, String>,
    unit: &str,
) -> Result<(Option<String>, Option<String>), String> {
    let raw = run(
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
fn socket_is_admissible(load: Option<&str>, active: Option<&str>) -> bool {
    load == Some("not-found")
        || (load == Some("loaded") && matches!(active, Some("active" | "inactive")))
}

fn preflight_with(
    run: &mut impl FnMut(&str, &[&str]) -> Result<String, String>,
) -> Result<(), String> {
    run("docker", &["--version"])?;
    let (load, active) = unit_state_with(run, "docker.service")?;
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
    let (load, active) = unit_state_with(run, "docker.socket")?;
    if socket_is_admissible(load.as_deref(), active.as_deref()) {
        Ok(())
    } else {
        Err("consulta no segura: estado de docker.socket no permitido".into())
    }
}

fn docker_with(
    run: &mut impl FnMut(&str, &[&str]) -> Result<String, String>,
    args: &[&str],
) -> Result<String, String> {
    let mut fixed = vec!["--host", HOST, "--config", "/dev/null"];
    fixed.extend_from_slice(args);
    run("docker", &fixed)
}

fn status_with(
    run: &mut impl FnMut(&str, &[&str]) -> Result<String, String>,
) -> Result<(), String> {
    preflight_with(run)?;
    docker_with(run, &["info", "--format", "{{.ServerVersion}}"])?;
    Ok(())
}

pub fn status() -> Result<(), String> {
    status_with(&mut command)
}

fn list_with(
    run: &mut impl FnMut(&str, &[&str]) -> Result<String, String>,
) -> Result<Vec<Container>, String> {
    preflight_with(run)?;
    let raw = docker_with(
        run,
        &[
            "ps",
            "--all",
            "--no-trunc",
            "--format",
            "{{json .Names}}\t{{json .ID}}\t{{json .Image}}\t{{json .State}}",
        ],
    )?;
    parse_containers(&raw)
}

pub fn list() -> Result<Vec<Container>, String> {
    list_with(&mut command)
}

fn inspect_with(
    run: &mut impl FnMut(&str, &[&str]) -> Result<String, String>,
    name: &str,
) -> Result<Container, String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        || name.starts_with('-')
    {
        return Err("nombre de contenedor inválido".into());
    }
    preflight_with(run)?;
    let raw = docker_with(
        run,
        &[
            "inspect",
            "--format",
            "{{json .Name}}\t{{json .Id}}\t{{json .Config.Image}}\t{{json .State.Status}}",
            "--",
            name,
        ],
    )?;
    let mut containers = parse_containers(&raw)?;
    if containers.len() != 1 {
        return Err("consulta fallida: respuesta de contenedor inesperada".into());
    }
    Ok(containers.remove(0))
}

pub fn inspect(name: &str) -> Result<Container, String> {
    inspect_with(&mut command, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct State<'a> {
        service_load: &'a str,
        service_active: &'a str,
        socket_load: &'a str,
        socket_active: &'a str,
    }

    fn runner<'a>(
        state: State<'a>,
        calls: &'a mut Vec<(String, Vec<String>)>,
    ) -> impl FnMut(&str, &[&str]) -> Result<String, String> + 'a {
        move |program, args| {
            calls.push((
                program.to_owned(),
                args.iter().map(|arg| (*arg).to_owned()).collect(),
            ));
            match program {
                "systemctl" => {
                    let unit = args[args.len() - 1];
                    let (load, active) = if unit == "docker.service" {
                        (state.service_load, state.service_active)
                    } else {
                        (state.socket_load, state.socket_active)
                    };
                    Ok(format!("LoadState={load}\nActiveState={active}\n"))
                }
                "docker" if args == ["--version"] => Ok("Docker version test\n".into()),
                "docker" => Ok(match args.get(4).copied() {
                    Some("info") => "27.0\n".into(),
                    Some("ps") => {
                        "\"/demo\"\t\"1234567890abcdef\"\t\"image:1\"\t\"running\"\n".into()
                    }
                    Some("inspect") => {
                        "\"/demo\"\t\"1234567890abcdef\"\t\"image:1\"\t\"running\"\n".into()
                    }
                    _ => panic!("unexpected Docker args: {args:?}"),
                }),
                _ => panic!("unexpected program {program}"),
            }
        }
    }

    fn assert_no_daemon(calls: &[(String, Vec<String>)]) {
        assert!(!calls
            .iter()
            .any(|(program, args)| program == "docker"
                && args.first() != Some(&"--version".to_owned())));
    }

    #[test]
    fn active_socket_allows_only_fixed_read_only_operations() {
        let state = State {
            service_load: "loaded",
            service_active: "active",
            socket_load: "loaded",
            socket_active: "active",
        };
        let mut calls = vec![];
        assert!(status_with(&mut runner(state, &mut calls)).is_ok());
        assert!(list_with(&mut runner(state, &mut calls)).is_ok());
        assert!(inspect_with(&mut runner(state, &mut calls), "demo").is_ok());
        let daemon_calls = calls
            .iter()
            .filter(|(program, args)| {
                program == "docker" && args.first() != Some(&"--version".to_owned())
            })
            .collect::<Vec<_>>();
        assert_eq!(daemon_calls, vec![
            &("docker".into(), vec!["--host", HOST, "--config", "/dev/null", "info", "--format", "{{.ServerVersion}}"].into_iter().map(str::to_owned).collect()),
            &("docker".into(), vec!["--host", HOST, "--config", "/dev/null", "ps", "--all", "--no-trunc", "--format", "{{json .Names}}\t{{json .ID}}\t{{json .Image}}\t{{json .State}}"].into_iter().map(str::to_owned).collect()),
            &("docker".into(), vec!["--host", HOST, "--config", "/dev/null", "inspect", "--format", "{{json .Name}}\t{{json .Id}}\t{{json .Config.Image}}\t{{json .State.Status}}", "--", "demo"].into_iter().map(str::to_owned).collect()),
        ]);
    }

    #[test]
    fn rejected_service_states_never_run_daemon_operation() {
        for active in ["inactive", "failed", "unknown"] {
            let state = State {
                service_load: "loaded",
                service_active: active,
                socket_load: "loaded",
                socket_active: "active",
            };
            let mut calls = vec![];
            assert!(status_with(&mut runner(state, &mut calls)).is_err());
            assert_no_daemon(&calls);
        }
    }

    #[test]
    fn uncertain_socket_states_never_run_daemon_operation() {
        for (load, active) in [
            ("loaded", "failed"),
            ("loaded", "unknown"),
            ("masked", "inactive"),
            ("error", "active"),
        ] {
            let state = State {
                service_load: "loaded",
                service_active: "active",
                socket_load: load,
                socket_active: active,
            };
            let mut calls = vec![];
            assert!(list_with(&mut runner(state, &mut calls)).is_err());
            assert_no_daemon(&calls);
        }
    }

    #[test]
    fn loaded_inactive_and_not_found_socket_preserve_inspect_behavior() {
        for (load, active) in [("loaded", "inactive"), ("not-found", "inactive")] {
            let state = State {
                service_load: "loaded",
                service_active: "active",
                socket_load: load,
                socket_active: active,
            };
            let mut calls = vec![];
            assert_eq!(
                inspect_with(&mut runner(state, &mut calls), "demo")
                    .unwrap()
                    .name,
                "demo"
            );
            assert!(calls
                .iter()
                .any(|(program, args)| program == "docker" && args.contains(&"inspect".into())));
        }
    }

    #[test]
    fn invalid_inspect_name_precedes_every_subprocess() {
        let state = State {
            service_load: "loaded",
            service_active: "active",
            socket_load: "loaded",
            socket_active: "active",
        };
        let mut calls = vec![];
        assert_eq!(
            inspect_with(&mut runner(state, &mut calls), "../demo"),
            Err("nombre de contenedor inválido".into())
        );
        assert!(calls.is_empty());
    }

    #[test]
    fn parses_only_selected_container_fields() {
        let entries =
            parse_containers("\"/demo\"\t\"1234567890abcdef\"\t\"image:1\"\t\"exited\"\n").unwrap();
        assert_eq!(
            entries[0],
            Container {
                name: "demo".into(),
                id: "1234567890ab".into(),
                image: "image:1".into(),
                state: "exited".into()
            }
        );
        assert_eq!(parse_containers("").unwrap(), vec![]);
        assert!(parse_containers("bad\tdata").is_err());
    }
}
