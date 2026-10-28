//! Consultas puntuales y de solo lectura al gestor systemd del sistema.

use std::process::Command;
use std::str::FromStr;

/// Operaciones que se pueden describir, no ejecutar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceOperation {
    Start,
    Stop,
    Restart,
}

impl FromStr for ServiceOperation {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "start" => Ok(Self::Start),
            "stop" => Ok(Self::Stop),
            "restart" => Ok(Self::Restart),
            _ => Err("operación de servicio desconocida".into()),
        }
    }
}

impl ServiceOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
        }
    }
}

// Subconjunto conservador de nombres systemd: sin escapes, plantillas vacías ni rutas.
fn valid_plan_unit(unit: &str) -> bool {
    if unit.len() > 255 || !unit.ends_with(".service") {
        return false;
    }
    let stem = &unit[..unit.len() - ".service".len()];
    if stem.is_empty() || stem.starts_with('.') || stem.starts_with('-') {
        return false;
    }
    let mut parts = stem.split('@');
    let name = parts.next().unwrap_or("");
    let instance = parts.next();
    if name.is_empty() || instance == Some("") || parts.next().is_some() {
        return false;
    }
    stem.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':' | b'@'))
}

/// Lista exacta para autorizar únicamente la creación de planes; vacía por defecto.
#[derive(Debug, Default)]
pub struct PlanAllowlist {
    units: Vec<String>,
}

impl PlanAllowlist {
    pub fn new(units: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let units: Vec<String> = units.into_iter().collect();
        if units.iter().any(|unit| !valid_plan_unit(unit)) {
            return Err("nombre de servicio inválido".into());
        }
        Ok(Self { units })
    }

    pub fn plan(&self, operation: ServiceOperation, unit: &str) -> Result<ServicePlan, String> {
        if !valid_plan_unit(unit) {
            return Err("nombre de servicio inválido".into());
        }
        if !self.units.iter().any(|allowed| allowed == unit) {
            return Err("servicio no autorizado para planificación".into());
        }
        Ok(ServicePlan {
            operation,
            unit: unit.to_owned(),
        })
    }
}

/// Política de ejecución independiente de la autorización para planificar.
/// No existe mecanismo de activación ni unidades administradas en esta etapa.
#[derive(Debug, Default)]
pub struct ExecutionPolicy;

impl ExecutionPolicy {
    pub fn authorize(&self, _plan: &ServicePlan) -> Result<(), &'static str> {
        Err("ejecución de servicios deshabilitada")
    }
}

/// Prepara argumentos separados, sin invocar systemctl ni autorizar la operación.
pub fn execution_arguments(plan: &ServicePlan) -> Result<[String; 4], &'static str> {
    if !valid_plan_unit(&plan.unit) {
        return Err("nombre de servicio inválido");
    }
    Ok([
        "--system".into(),
        plan.operation.as_str().into(),
        "--".into(),
        plan.unit.clone(),
    ])
}

/// Un código de salida no acredita que el servicio alcanzó el estado deseado.
#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionResult {
    CommandExitedSuccessfullyUnverified,
    CommandFailed,
}

fn run_prepared(
    args: &[String; 4],
    runner: &mut impl FnMut(&[String; 4]) -> Result<bool, String>,
) -> Result<ExecutionResult, String> {
    Ok(if runner(args)? {
        ExecutionResult::CommandExitedSuccessfullyUnverified
    } else {
        ExecutionResult::CommandFailed
    })
}

/// El ejecutor inyectado no se invoca mientras la política deniegue la acción.
pub fn execute_service(
    plan: &ServicePlan,
    runner: &mut impl FnMut(&[String; 4]) -> Result<bool, String>,
) -> Result<ExecutionResult, String> {
    ExecutionPolicy.authorize(plan).map_err(str::to_owned)?;
    let args = execution_arguments(plan).map_err(str::to_owned)?;
    run_prepared(&args, runner)
}

/// La política actual rechaza todos los planes antes de arrancar systemctl.
pub fn execute_service_systemctl(plan: &ServicePlan) -> Result<ExecutionResult, String> {
    execute_service(plan, &mut |args| {
        Command::new("/usr/bin/systemctl")
            .env_clear()
            .args(args)
            .status()
            .map(|status| status.success())
            .map_err(|error| format!("no se pudo iniciar systemctl: {error}"))
    })
}

/// Descripción en memoria; no tiene ninguna capacidad de ejecución.
#[derive(Debug, PartialEq, Eq)]
pub struct ServicePlan {
    pub operation: ServiceOperation,
    pub unit: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Service {
    pub name: String,
    pub load: Option<String>,
    pub active: Option<String>,
    pub sub: Option<String>,
}

impl Service {
    pub fn category(&self) -> &'static str {
        if self.load.as_deref() != Some("loaded") {
            return "desconocido";
        }
        match self.active.as_deref() {
            Some("active") => "activo",
            Some("inactive") => "inactivo",
            Some("failed") => "fallido",
            _ => "desconocido",
        }
    }
}

fn field(value: &str) -> Option<String> {
    if value.is_empty() || value == "-" {
        None
    } else {
        Some(value.to_owned())
    }
}

pub fn parse_list(raw: &str) -> Result<Vec<Service>, String> {
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split_whitespace();
            let first = fields.next().ok_or("salida de systemd inválida")?;
            let name = if first == "●" {
                fields.next().ok_or("salida de systemd inválida")?
            } else {
                first
            };
            if !name.ends_with(".service") {
                return Err("unidad de servicio inválida en respuesta de systemd".into());
            }
            Ok(Service {
                name: name.to_owned(),
                load: field(fields.next().ok_or("falta estado de carga")?),
                active: field(fields.next().ok_or("falta estado de actividad")?),
                sub: field(fields.next().ok_or("falta subestado")?),
            })
        })
        .collect()
}

pub fn parse_status(raw: &str) -> Result<Service, String> {
    let mut name = None;
    let mut load = None;
    let mut active = None;
    let mut sub = None;
    for line in raw.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err("salida de systemd inválida".into());
        };
        match key {
            "Id" => name = field(value),
            "LoadState" => load = field(value),
            "ActiveState" => active = field(value),
            "SubState" => sub = field(value),
            _ => {}
        }
    }
    let name = name.ok_or("respuesta de systemd sin nombre de servicio")?;
    if !name.ends_with(".service") {
        return Err("unidad de servicio inválida en respuesta de systemd".into());
    }
    Ok(Service {
        name,
        load,
        active,
        sub,
    })
}

fn query_command(args: &[&str]) -> Command {
    let mut command = Command::new("/usr/bin/systemctl");
    command.env("LC_ALL", "C").args(args);
    command
}

fn query(args: &[&str]) -> Result<String, String> {
    let output = query_command(args)
        .output()
        .map_err(|error| format!("no se pudo consultar systemd: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(format!("consulta a systemd fallida: {}", detail.trim()));
    }
    String::from_utf8(output.stdout).map_err(|_| "respuesta de systemd no UTF-8".into())
}

pub fn list() -> Result<Vec<Service>, String> {
    let raw = query(&[
        "--system",
        "--no-pager",
        "--no-legend",
        "--plain",
        "--full",
        "--all",
        "--type=service",
        "list-units",
    ])?;
    parse_list(&raw)
}

pub fn status(name: &str) -> Result<Service, String> {
    if name.is_empty()
        || name.starts_with('-')
        || !name.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'@' | b':' | b'\\')
        })
    {
        return Err("nombre de servicio inválido".into());
    }
    let unit = if name.ends_with(".service") {
        name.to_owned()
    } else {
        format!("{name}.service")
    };
    let raw = query(&[
        "--system",
        "--no-pager",
        "--property=Id,LoadState,ActiveState,SubState",
        "show",
        "--",
        &unit,
    ])?;
    parse_status(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_queries_use_absolute_systemctl_with_separate_arguments() {
        let command = query_command(&["--system", "show", "--", "demo.service"]);
        assert_eq!(command.get_program(), "/usr/bin/systemctl");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["--system", "show", "--", "demo.service"]
        );
        let list = query_command(&["--system", "--all", "list-units"]);
        assert_eq!(list.get_program(), "/usr/bin/systemctl");
        assert_eq!(
            list.get_args().collect::<Vec<_>>(),
            ["--system", "--all", "list-units"]
        );
    }

    #[test]
    fn plans_only_exact_allowed_units_and_known_operations() {
        let empty = PlanAllowlist::default();
        assert!(empty.plan(ServiceOperation::Start, "demo.service").is_err());
        let allowed =
            PlanAllowlist::new(["demo.service".to_owned(), "worker@one.service".to_owned()])
                .unwrap();
        for (text, operation) in [
            ("start", ServiceOperation::Start),
            ("stop", ServiceOperation::Stop),
            ("restart", ServiceOperation::Restart),
        ] {
            assert_eq!(text.parse::<ServiceOperation>().unwrap(), operation);
            assert_eq!(
                allowed.plan(operation, "demo.service").unwrap(),
                ServicePlan {
                    operation,
                    unit: "demo.service".into()
                }
            );
        }
        assert!("reload".parse::<ServiceOperation>().is_err());
        assert!(allowed
            .plan(ServiceOperation::Stop, "other.service")
            .is_err());
        assert!(allowed
            .plan(ServiceOperation::Stop, "worker@one.service")
            .is_ok());
        for invalid in [
            "",
            "demo",
            ".service",
            "-x.service",
            "../x.service",
            "x@.service",
            "x@@y.service",
            "x\\x2fy.service",
            "x;id.service",
            "x\n.service",
        ] {
            assert!(allowed.plan(ServiceOperation::Start, invalid).is_err());
            assert!(PlanAllowlist::new([invalid.to_owned()]).is_err());
        }
        assert!(allowed
            .plan(
                ServiceOperation::Start,
                &format!("{}.service", "a".repeat(248))
            )
            .is_err());
    }

    #[test]
    fn disabled_execution_never_invokes_injected_runner() {
        let plan = PlanAllowlist::new(["demo.service".to_owned()])
            .unwrap()
            .plan(ServiceOperation::Restart, "demo.service")
            .unwrap();
        let mut calls = 0;
        let mut fake = |_: &[String; 4]| {
            calls += 1;
            Ok(false)
        };
        assert!(execute_service(&plan, &mut fake).is_err());
        assert_eq!(calls, 0);
        assert_eq!(
            execute_service_systemctl(&plan),
            Err("ejecución de servicios deshabilitada".into())
        );
    }

    #[test]
    fn simulated_runner_result_is_not_a_verified_service_outcome() {
        let args = execution_arguments(&ServicePlan {
            operation: ServiceOperation::Stop,
            unit: "demo.service".into(),
        })
        .unwrap();
        let mut fake = |seen: &[String; 4]| {
            assert_eq!(seen[1], "stop");
            Ok(true)
        };
        assert_eq!(
            run_prepared(&args, &mut fake).unwrap(),
            ExecutionResult::CommandExitedSuccessfullyUnverified
        );
        let mut fail = |_: &[String; 4]| Ok(false);
        assert_eq!(
            run_prepared(&args, &mut fail).unwrap(),
            ExecutionResult::CommandFailed
        );
    }

    #[test]
    fn execution_is_denied_independently_of_planning_and_arguments_are_separate() {
        let allowed = PlanAllowlist::new(["demo.service".to_owned()]).unwrap();
        for operation in [
            ServiceOperation::Start,
            ServiceOperation::Stop,
            ServiceOperation::Restart,
        ] {
            let plan = allowed.plan(operation, "demo.service").unwrap();
            assert_eq!(
                ExecutionPolicy.authorize(&plan),
                Err("ejecución de servicios deshabilitada")
            );
            assert_eq!(
                execution_arguments(&plan).unwrap(),
                ["--system", operation.as_str(), "--", "demo.service"]
            );
        }
        let forged = ServicePlan {
            operation: ServiceOperation::Start,
            unit: "--bad.service".into(),
        };
        assert!(ExecutionPolicy.authorize(&forged).is_err());
        assert!(execution_arguments(&forged).is_err());
    }

    #[test]
    fn list_parses_services_and_categories() {
        let services = parse_list(
            "a.service loaded active running A\nb.service loaded inactive dead B\n● c.service loaded failed failed C\n",
        )
        .unwrap();
        assert_eq!(
            services.iter().map(Service::category).collect::<Vec<_>>(),
            ["activo", "inactivo", "fallido"]
        );
        assert_eq!(services[0].sub.as_deref(), Some("running"));
        assert!(parse_list("a.service loaded\n").is_err());
    }

    #[test]
    fn status_does_not_treat_missing_information_as_stopped() {
        let missing = parse_status(
            "Id=missing.service\nLoadState=not-found\nActiveState=inactive\nSubState=dead\n",
        )
        .unwrap();
        assert_eq!(missing.category(), "desconocido");
        let partial =
            parse_status("Id=a.service\nLoadState=loaded\nActiveState=\nSubState=\n").unwrap();
        assert_eq!(partial.category(), "desconocido");
        assert_eq!(partial.sub, None);
        assert!(parse_status("LoadState=loaded\n").is_err());
    }
}
