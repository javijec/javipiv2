//! Diagnóstico puntual de capacidad de la raíz; no realiza limpieza.
use super::storage::{capacity_from_blocks, Capacity};

pub fn status_with<E>(query: impl FnOnce() -> Result<Capacity, E>) -> String {
    format_observation(query().ok())
}

pub fn format_observation(capacity: Option<Capacity>) -> String {
    let value = capacity.map_or_else(
        || "desconocido".to_owned(),
        |c| {
            format!(
                "total={} B, usada={} B, disponible={} B, reservada={} B",
                c.total_bytes, c.used_bytes, c.available_bytes, c.reserved_bytes
            )
        },
    );
    format!("Raíz (/): {value}\nNo identifica espacio recuperable ni examina /srv.")
}

#[allow(clippy::unnecessary_cast)]
fn root_capacity() -> Result<Capacity, ()> {
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // Only the fixed root filesystem is queried, never /srv or a candidate path.
    if unsafe { libc::statvfs(c"/".as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(());
    }
    let stats = unsafe { stats.assume_init() };
    capacity_from_blocks(
        stats.f_blocks as u64,
        stats.f_bfree as u64,
        stats.f_bavail as u64,
        stats.f_frsize as u64,
    )
    .ok_or(())
}

pub fn observe() -> Option<Capacity> {
    root_capacity().ok()
}
pub fn status() -> String {
    format_observation(observe())
}
