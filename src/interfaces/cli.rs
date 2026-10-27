//! Una sola interfaz de comandos, sin operaciones privilegiadas.

use crate::core::catalog::{all_modules, profile_modules};
use crate::modules::health;
use std::ffi::OsString;

fn help() {
    println!(
        "JaviPiV2 {} — preview de solo lectura",
        env!("CARGO_PKG_VERSION")
    );
    println!("Uso: javipiv2 <comando>");
    println!("  status                 Host, uptime y memoria");
    println!("  module list            Catálogo (no instalaciones reales)");
    println!("  plan <perfil>          Módulos previstos: minimal, developer, full");
    println!("  version                Versión del proyecto");
    println!("  help                   Esta ayuda");
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let parts = args
        .iter()
        .map(|s| s.to_str().ok_or("argumento no UTF-8"))
        .collect::<Result<Vec<_>, _>>()?;
    match parts.as_slice() {
        [] | ["help"] | ["--help"] | ["-h"] => help(),
        ["version"] | ["--version"] => println!("javipiv2 {}", env!("CARGO_PKG_VERSION")),
        ["status"] => {
            let status = health::read_status().map_err(|e| format!("consulta /proc: {e}"))?;
            println!("Host: {}", status.hostname);
            println!(
                "Uptime: {} h {} min",
                status.uptime_seconds / 3600,
                (status.uptime_seconds / 60) % 60
            );
            println!(
                "Memoria disponible: {} / {} MiB",
                status.memory_available_kib / 1024,
                status.memory_total_kib / 1024
            );
        }
        ["module", "list"] => {
            for module in all_modules()? {
                println!(
                    "{:<15} {:<8} {}",
                    module.id, module.state, module.description
                );
            }
            println!("Nota: el catálogo no acredita instalación ni activación.");
        }
        ["plan", profile] => {
            println!("Perfil {profile} (propuesta, no ejecuta instalaciones):");
            for module in profile_modules(profile)? {
                println!("  {:<15} {}", module.id, module.state);
            }
        }
        _ => return Err("comando desconocido; usá javipiv2 help".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::ffi::OsString;

    #[test]
    fn known_read_only_commands() {
        for args in [
            vec!["help"],
            vec!["version"],
            vec!["module", "list"],
            vec!["plan", "minimal"],
        ] {
            assert!(run(args.into_iter().map(OsString::from)).is_ok());
        }
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(run(vec![OsString::from("service"), OsString::from("restart")]).is_err());
    }
}
