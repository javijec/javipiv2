//! Consultas puntuales sin npm, configuración del usuario ni operaciones de instalación.
use std::io::Read;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Current,
    Available,
    InstalledNewer,
    Unknown,
}

pub fn parse_version(bytes: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(bytes)
        .ok()?
        .strip_prefix('v')
        .unwrap_or(std::str::from_utf8(bytes).ok()?);
    if s.len() > 32 {
        return None;
    }
    let parts: Vec<_> = s.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || p.len() > 9 || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    Some(s.to_owned())
}

pub fn compare(installed: Option<&str>, upstream: Option<&str>) -> Comparison {
    fn numbers(value: &str) -> Option<[u32; 3]> {
        let version = parse_version(value.as_bytes())?;
        let mut parts = version.split('.').map(str::parse::<u32>);
        Some([
            parts.next()?.ok()?,
            parts.next()?.ok()?,
            parts.next()?.ok()?,
        ])
    }
    match (installed.and_then(numbers), upstream.and_then(numbers)) {
        (Some(a), Some(b)) => match a.cmp(&b) {
            std::cmp::Ordering::Equal => Comparison::Current,
            std::cmp::Ordering::Less => Comparison::Available,
            std::cmp::Ordering::Greater => Comparison::InstalledNewer,
        },
        _ => Comparison::Unknown,
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub name: &'static str,
    pub installed: Option<String>,
    pub upstream: Option<String>,
    pub comparison: Comparison,
}
impl Observation {
    pub fn line(&self) -> String {
        format!(
            "{}: instalada={}, upstream={}, estado={}",
            self.name,
            self.installed.as_deref().unwrap_or("desconocido"),
            self.upstream.as_deref().unwrap_or("desconocido"),
            match self.comparison {
                Comparison::Current => "actual",
                Comparison::Available => "actualización disponible",
                Comparison::InstalledNewer => "instalada más reciente",
                Comparison::Unknown => "desconocido",
            }
        )
    }
}

// Only fixed package paths are inspected. Reject symlinks at every component.
pub(crate) fn local_file(path: &str) -> Option<Vec<u8>> {
    use std::os::fd::{AsRawFd, FromRawFd};
    let mut dir = std::fs::File::open("/").ok()?;
    let mut parts = path.trim_start_matches('/').split('/').peekable();
    while let Some(part) = parts.next() {
        let name = std::ffi::CString::new(part).ok()?;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | libc::O_NONBLOCK
            | if parts.peek().is_some() {
                libc::O_DIRECTORY
            } else {
                0
            };
        let fd = unsafe { libc::openat(dir.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return None;
        }
        dir = unsafe { std::fs::File::from_raw_fd(fd) };
    }
    if !dir.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    dir.take(65537).read_to_end(&mut bytes).ok()?;
    if bytes.len() > 65536 {
        return None;
    }
    Some(bytes)
}

pub fn json_version(bytes: &[u8], field: &str) -> Option<String> {
    if bytes.len() > 65536 {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    parse_version(value.get(field)?.as_str()?.as_bytes())
}

pub fn transport_body(success: bool, bytes: &[u8]) -> Option<&[u8]> {
    if !success || bytes.len() > 65540 || !bytes.ends_with(b"\n200") {
        return None;
    }
    Some(&bytes[..bytes.len() - 4])
}

fn upstream(url: &str) -> Option<Vec<u8>> {
    // No inherited proxy, credentials, curl configuration, redirects or PATH lookup.
    let mut child = Command::new("/usr/bin/curl")
        .env_clear()
        .args([
            "--disable",
            "--silent",
            "--show-error",
            "--fail",
            "--proto",
            "=https",
            "--noproxy",
            "*",
            "--connect-timeout",
            "2",
            "--max-time",
            "5",
            "--max-filesize",
            "65536",
            "--user-agent",
            "javipiv2",
            "--write-out",
            "\n%{http_code}",
            url,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    let mut bytes = Vec::new();
    let read = child.stdout.take()?.take(65541).read_to_end(&mut bytes);
    if read.is_err() || bytes.len() > 65540 {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    let success = child.wait().ok()?.success();
    Some(transport_body(success, &bytes)?.to_vec())
}

pub(crate) fn local_version(
    package: &str,
    local: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<String> {
    let mut found: Option<String> = None;
    let mut ambiguous = false;
    for prefix in [
        "/usr/local/lib/node_modules",
        "/usr/lib/node_modules",
        "/home/dietpi/.pi/agent/npm/node_modules",
        "/home/dietpi/.npm-global/lib/node_modules",
    ] {
        if let Some(bytes) = local(&format!("{prefix}/{package}/package.json")) {
            match json_version(&bytes, "version") {
                Some(version) => {
                    if found.as_ref().is_some_and(|previous| {
                        compare(Some(previous), Some(&version)) != Comparison::Current
                    }) {
                        ambiguous = true;
                    }
                    found = Some(version);
                }
                None => ambiguous = true,
            }
        }
    }
    if ambiguous {
        None
    } else {
        found
    }
}

pub fn check() -> Vec<Observation> {
    check_with(local_file, upstream)
}

/// Injected sources make mapping and failures testable without filesystem or network access.
pub fn check_with(
    mut local: impl FnMut(&str) -> Option<Vec<u8>>,
    mut fetch: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Vec<Observation> {
    // These are the original's default components and public upstreams. No redirects
    // are followed; a GitHub redirect or any failure is unknown.
    let components = [
        (
            "Codex",
            Some("@openai/codex"),
            "https://registry.npmjs.org/@openai%2fcodex/latest",
            "version",
        ),
        (
            "Pi",
            Some("@earendil-works/pi-coding-agent"),
            "https://registry.npmjs.org/@earendil-works%2fpi-coding-agent/latest",
            "version",
        ),
        (
            "Gentle Shell",
            Some("gentle-pi"),
            "https://registry.npmjs.org/gentle-pi/latest",
            "version",
        ),
        (
            "Paseo",
            Some("@getpaseo/cli"),
            "https://registry.npmjs.org/@getpaseo%2fcli/latest",
            "version",
        ),
        (
            "Gentle AI",
            None,
            "https://api.github.com/repos/Gentleman-Programming/gentle-ai/releases/latest",
            "tag_name",
        ),
        (
            "Engram",
            None,
            "https://api.github.com/repos/Gentleman-Programming/engram/releases/latest",
            "tag_name",
        ),
    ];
    components
        .into_iter()
        .map(|(name, package, url, field)| {
            let installed = package.and_then(|p| local_version(p, &mut local));
            let upstream = fetch(url).and_then(|bytes| json_version(&bytes, field));
            let comparison = compare(installed.as_deref(), upstream.as_deref());
            Observation {
                name,
                installed,
                upstream,
                comparison,
            }
        })
        .collect()
}
