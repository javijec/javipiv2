//! Observación local conservadora; no ejecuta agentes ni consulta red.
use super::updates;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    Absent,
    Executable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub name: &'static str,
    pub version: Option<String>,
    pub executable: Option<String>,
}
impl Observation {
    pub fn line(&self) -> String {
        let executable = self.executable.as_deref().map_or_else(
            || "desconocido".to_owned(),
            |path| format!("{path} (ruta consultada; destino regular con bits de ejecución; acceso no garantizado)"),
        );
        format!(
            "{}: ejecutable={}, versión npm={} (no vinculada al ejecutable), estado=desconocido",
            self.name,
            executable,
            self.version.as_deref().unwrap_or("desconocido")
        )
    }
}

// Only called with the fixed paths below. O_PATH reads metadata, never file contents;
// each component is opened relative to its verified parent without following symlinks.
fn executable_metadata(path: &str) -> Probe {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::fs::MetadataExt;
    let Ok(mut dir) = std::fs::File::open("/") else {
        return Probe::Unknown;
    };
    let mut parts = path.trim_start_matches('/').split('/').peekable();
    while let Some(part) = parts.next() {
        if part.is_empty() || part == "." || part == ".." {
            return Probe::Unknown;
        }
        let Ok(name) = std::ffi::CString::new(part) else {
            return Probe::Unknown;
        };
        let flags = libc::O_PATH
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if parts.peek().is_some() {
                libc::O_DIRECTORY
            } else {
                0
            };
        let fd = unsafe { libc::openat(dir.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                Probe::Absent
            } else {
                Probe::Unknown
            };
        }
        dir = unsafe { std::fs::File::from_raw_fd(fd) };
    }
    match dir.metadata() {
        Ok(m) if m.is_file() && m.mode() & 0o111 != 0 => Probe::Executable,
        Ok(m) if m.file_type().is_symlink() => {
            let mut buffer = [0u8; 4096];
            // An empty pathname reads the link held by this O_PATH descriptor, without following it.
            let n = unsafe {
                libc::readlinkat(
                    dir.as_raw_fd(),
                    c"".as_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if n < 0 || n as usize == buffer.len() {
                return Probe::Unknown;
            }
            let target = &buffer[..n as usize];
            let allowed = allowed_link_target(path, target);
            // Only the two fixed, non-/srv targets above may be inspected; never follow other links.
            allowed.map_or(Probe::Unknown, executable_metadata)
        }
        _ => Probe::Unknown,
    }
}

fn allowed_link_target(path: &str, target: &[u8]) -> Option<&'static str> {
    match (path, target) {
        (
            "/home/dietpi/.local/bin/gentle-shell",
            b"/home/dietpi/.gentle-shell/bin/gentle-shell",
        ) => Some("/home/dietpi/.gentle-shell/bin/gentle-shell"),
        ("/home/dietpi/.local/bin/pi", b"../../.pi/agent/bin/pi") => {
            Some("/home/dietpi/.pi/agent/bin/pi")
        }
        _ => None,
    }
}

pub fn status() -> Vec<Observation> {
    status_with_probes(updates::local_file, executable_metadata)
}
// Preserve the original synthetic interface without any filesystem probes.
pub fn status_with(local: impl FnMut(&str) -> Option<Vec<u8>>) -> Vec<Observation> {
    status_with_probes(local, |_| Probe::Absent)
}
pub fn status_with_probes(
    mut local: impl FnMut(&str) -> Option<Vec<u8>>,
    mut probe: impl FnMut(&str) -> Probe,
) -> Vec<Observation> {
    [
        ("Pi Agent", "pi", Some("@earendil-works/pi-coding-agent")),
        ("Gentle Shell", "gentle-shell", Some("gentle-pi")),
        ("Codex", "codex", Some("@openai/codex")),
        ("Engram", "engram", None),
    ]
    .into_iter()
    .map(|(name, binary, package)| {
        let mut found = None;
        let mut ambiguous = false;
        let preferred = match binary {
            "pi" => Some("/home/dietpi/.pi/agent/bin/pi"),
            "gentle-shell" => Some("/home/dietpi/.local/bin/gentle-shell"),
            _ => None,
        };
        if let Some(path) = preferred {
            match probe(path) {
                Probe::Executable => {
                    return Observation {
                        name,
                        version: package.and_then(|p| updates::local_version(p, &mut local)),
                        executable: Some(path.to_owned()),
                    };
                }
                Probe::Unknown => {
                    return Observation {
                        name,
                        version: package.and_then(|p| updates::local_version(p, &mut local)),
                        executable: None,
                    }
                }
                Probe::Absent => {}
            }
        }
        for prefix in ["/usr/local/bin", "/usr/bin", "/home/dietpi/.local/bin"] {
            let path = format!("{prefix}/{binary}");
            match probe(&path) {
                Probe::Absent => {}
                Probe::Unknown => ambiguous = true,
                Probe::Executable if found.is_none() => found = Some(path),
                Probe::Executable => ambiguous = true,
            }
        }
        Observation {
            name,
            version: package.and_then(|p| updates::local_version(p, &mut local)),
            executable: if ambiguous { None } else { found },
        }
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{allowed_link_target, executable_metadata, Probe};
    use std::os::unix::fs::{symlink, PermissionsExt};

    #[test]
    fn only_exact_authorized_non_srv_links_are_accepted() {
        assert_eq!(
            allowed_link_target(
                "/home/dietpi/.local/bin/gentle-shell",
                b"/home/dietpi/.gentle-shell/bin/gentle-shell"
            ),
            Some("/home/dietpi/.gentle-shell/bin/gentle-shell")
        );
        assert_eq!(
            allowed_link_target("/home/dietpi/.local/bin/codex", b"/srv/tools/codex"),
            None
        );
        assert_eq!(
            allowed_link_target(
                "/home/dietpi/.local/bin/gentle-shell",
                b"/srv/tools/gentle-shell"
            ),
            None
        );
        assert_eq!(
            allowed_link_target("/home/dietpi/.local/bin/pi", b"../../.pi/agent/bin/other"),
            None
        );
    }

    #[test]
    fn agents_metadata_rejects_symlinks_and_traversal() {
        let root = std::env::temp_dir().join(format!("javipiv2-agents-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let file = root.join("file");
        std::fs::write(&file, b"not executed").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        let link = root.join("link");
        symlink(&file, &link).unwrap();
        assert_eq!(
            executable_metadata(file.to_str().unwrap()),
            Probe::Executable
        );
        assert_eq!(executable_metadata(link.to_str().unwrap()), Probe::Unknown);
        assert_eq!(
            executable_metadata(&format!("{}/../file", root.display())),
            Probe::Unknown
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
