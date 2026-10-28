//! Observación pasiva de mountinfo; no accede a los puntos de montaje.
use std::collections::HashSet;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct StorageStatus {
    pub autofs: bool,
    pub srv_ext4: bool,
    pub docker_overlays: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capacity {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub reserved_bytes: u64,
}

/// f_bavail excludes blocks reserved for privileged use; all byte arithmetic is checked.
pub fn capacity_from_blocks(
    blocks: u64,
    free: u64,
    available: u64,
    fragment_size: u64,
) -> Option<Capacity> {
    if fragment_size == 0 || free > blocks || available > free {
        return None;
    }
    let total_bytes = blocks.checked_mul(fragment_size)?;
    let free_bytes = free.checked_mul(fragment_size)?;
    let available_bytes = available.checked_mul(fragment_size)?;
    Some(Capacity {
        total_bytes,
        used_bytes: total_bytes.checked_sub(free_bytes)?,
        available_bytes,
        reserved_bytes: free_bytes.checked_sub(available_bytes)?,
    })
}

/// Calls the getter only when mountinfo unambiguously confirms active ext4 at /srv.
pub fn observe_capacity<E>(
    raw: &str,
    getter: impl FnOnce() -> Result<Capacity, E>,
) -> Result<(StorageStatus, Option<Capacity>), String> {
    let status = inspect_checked(raw).map_err(str::to_owned)?;
    if !status.srv_ext4 {
        return Ok((status, None));
    }
    let capacity = getter().ok();
    Ok((status, capacity))
}

#[allow(clippy::unnecessary_cast)] // statvfs field widths vary across Unix targets.
fn stat_capacity() -> Result<Capacity, String> {
    let path = std::ffi::CString::new("/srv").expect("static path");
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // Lookup may activate autofs; mountinfo confirmation remains subject to TOCTOU.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let stats = unsafe { stats.assume_init() };
    capacity_from_blocks(
        stats.f_blocks as u64,
        stats.f_bfree as u64,
        stats.f_bavail as u64,
        stats.f_frsize as u64,
    )
    .ok_or_else(|| "estadísticas de capacidad inválidas".into())
}

/// Fails closed on mountinfo read/parse errors; capacity errors become unknown without retry.
pub fn read_capacity() -> Result<(StorageStatus, Option<Capacity>), String> {
    let raw = std::fs::read_to_string("/proc/self/mountinfo").map_err(|e| e.to_string())?;
    observe_capacity(&raw, stat_capacity)
}

fn unescape_mount_field(field: &str) -> Option<String> {
    let bytes = field.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            decoded.push(bytes[i]);
            i += 1;
            continue;
        }
        let code = bytes.get(i + 1..i + 4)?;
        decoded.push(match code {
            b"040" => b' ',
            b"011" => b'\t',
            b"012" => b'\n',
            b"134" => b'\\',
            _ => return None,
        });
        i += 4;
    }
    String::from_utf8(decoded).ok()
}

/// Interpreta evidencia suministrada; false significa desconocido, no ausencia.
pub fn inspect(raw: &str) -> StorageStatus {
    inspect_inner(raw, false).unwrap_or_default()
}

/// Interpreta mountinfo y rechaza evidencia estructuralmente inválida.
pub fn inspect_checked(raw: &str) -> Result<StorageStatus, &'static str> {
    inspect_inner(raw, true)
}

fn inspect_inner(raw: &str, checked: bool) -> Result<StorageStatus, &'static str> {
    if raw.is_empty() {
        return Err("mountinfo vacío");
    }
    let mut result = StorageStatus::default();
    let mut ids = HashSet::new();
    let mut entries = Vec::new();
    for line in raw.lines() {
        let Some((left, right)) = line.split_once(" - ") else {
            return Err("línea de mountinfo inválida");
        };
        let fields: Vec<_> = left.split_whitespace().collect();
        let after: Vec<_> = right.split_whitespace().collect();
        if fields.len() < 6 || after.len() < 3 {
            return Err("línea de mountinfo inválida");
        }
        let (Ok(id), Ok(parent)) = (fields[0].parse::<u64>(), fields[1].parse::<u64>()) else {
            return Err("identificadores de mountinfo inválidos");
        };
        if id == 0 || parent == 0 || !ids.insert(id) {
            return Err("identificadores de mountinfo inválidos");
        }
        let mount = unescape_mount_field(fields[4]);
        let root = unescape_mount_field(fields[3]);
        // Malformed unrelated paths are ignored; malformed /srv candidates invalidate evidence.
        let Some(mount) = mount else {
            if fields[4].starts_with('/') {
                return Err("ruta de montaje inválida");
            }
            continue;
        };
        if !mount.starts_with('/') {
            continue;
        }
        let Some(root) = root else {
            if mount == "/srv"
                || mount.starts_with("/srv/")
                || (checked && after[0] == "overlay" && mount.starts_with("/var/lib/docker/"))
            {
                return Err("raíz de montaje inválida");
            }
            continue;
        };
        entries.push((id, parent, root, mount, after[0], fields[2], after[1]));
    }
    let srv: Vec<_> = entries.iter().filter(|e| e.3 == "/srv").collect();
    if let [auto, ext] = srv.as_slice() {
        let (auto, ext) = if auto.4 == "autofs" {
            (*auto, *ext)
        } else {
            (*ext, *auto)
        };
        if auto.4 == "autofs" && auto.2 == "/" && auto.1 != ext.0 && ext.1 == auto.0 {
            result.autofs = true;
            result.srv_ext4 = is_ext4(ext);
        }
    } else if let [entry] = srv.as_slice() {
        result.autofs = entry.4 == "autofs" && entry.2 == "/";
        result.srv_ext4 = is_ext4(entry);
    }
    for entry in entries {
        if entry.4 == "overlay"
            && (entry.3.starts_with("/srv/docker/") || entry.3.starts_with("/var/lib/docker/"))
        {
            result.docker_overlays.push(entry.3);
        }
    }
    result.docker_overlays.sort();
    Ok(result)
}

fn is_ext4(entry: &(u64, u64, String, String, &str, &str, &str)) -> bool {
    entry.4 == "ext4"
        && entry.2 == "/"
        && entry.6 != "none"
        && entry.5.split_once(':').is_some_and(|(a, b)| {
            a.parse::<u32>().is_ok_and(|n| n != 0) && b.parse::<u32>().is_ok()
        })
}

pub fn read_status() -> StorageStatus {
    std::fs::read_to_string("/proc/self/mountinfo")
        .map(|raw| inspect(&raw))
        .unwrap_or_default()
}

/// Lectura explícita de mountinfo que conserva errores de I/O y parseo.
pub fn read_status_fallible() -> Result<StorageStatus, String> {
    let raw = std::fs::read_to_string("/proc/self/mountinfo").map_err(|error| error.to_string())?;
    inspect_checked(&raw).map_err(str::to_owned)
}
