//! Read-only installation evidence, not installer provenance.
//!
//! Verification trusts the specified owner and does not prevent concurrent privileged
//! writes (including modifications of an already opened inode). A matching digest
//! establishes byte agreement with the record only under that trust assumption.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    ffi::CString,
    fs::File,
    io::Read,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::Path,
};

const REGISTRY_LIMIT: u64 = 64 * 1024;
const EXECUTABLE_LIMIT: u64 = 128 * 1024 * 1024;
const EXECUTABLE: &str = "/usr/local/bin/javipiv2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Verified,
    Unknown,
    AvailableInDevelopment,
}

/// Explicitly marks running code as development; absence of evidence never does so.
pub fn development_availability() -> Availability {
    Availability::AvailableInDevelopment
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: u32,
    product_version: String,
    executable: String,
    sha256: String,
}

/// Verify the sole production registry and executable. Never writes or executes either.
#[cfg(target_os = "linux")]
pub fn verify_production() -> Availability {
    verify(Path::new("/"), 0)
}

/// Verify fixed relative paths under a caller-trusted isolated root with an explicit owner.
/// The caller is responsible for trusting the root's own parent path.
#[cfg(target_os = "linux")]
pub fn verify_fixture(root: &Path, owner: u32) -> Availability {
    verify(root, owner)
}

#[cfg(target_os = "linux")]
fn verify(root: &Path, owner: u32) -> Availability {
    if verify_inner(root, owner).is_some() {
        Availability::Verified
    } else {
        Availability::Unknown
    }
}

#[cfg(target_os = "linux")]
fn verify_inner(root: &Path, owner: u32) -> Option<()> {
    let path = CString::new(root.as_os_str().as_bytes()).ok()?;
    let fd = unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return None;
    }
    let root = unsafe { File::from_raw_fd(fd) };
    trusted(&root, owner, true)?;
    let registry = open_path(
        &root,
        &["var", "lib", "javipiv2", "installation.json"],
        owner,
    )?;
    let bytes = read_stable(registry, owner, REGISTRY_LIMIT)?;
    let record: Record = serde_json::from_slice(&bytes).ok()?;
    if record.schema_version != 1
        || record.product_version.trim().is_empty()
        || record.executable != EXECUTABLE
        || record.sha256.len() != 64
        || !record
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let executable = open_path(&root, &["usr", "local", "bin", "javipiv2"], owner)?;
    if trusted(&executable, owner, false)?.mode() & 0o111 == 0 {
        return None;
    }
    let bytes = read_stable(executable, owner, EXECUTABLE_LIMIT)?;
    if format!("{:x}", Sha256::digest(&bytes)) != record.sha256 {
        return None;
    }
    Some(())
}

#[cfg(target_os = "linux")]
fn trusted(file: &File, owner: u32, dir: bool) -> Option<std::fs::Metadata> {
    let m = file.metadata().ok()?;
    if m.uid() != owner
        || m.mode() & 0o022 != 0
        || if dir {
            !m.is_dir()
        } else {
            !m.is_file() || m.nlink() != 1
        }
    {
        return None;
    }
    Some(m)
}

#[cfg(target_os = "linux")]
fn open_path(root: &File, parts: &[&str], owner: u32) -> Option<File> {
    let mut current = root.try_clone().ok()?;
    for (i, part) in parts.iter().enumerate() {
        let dir = i + 1 < parts.len();
        let name = CString::new(*part).ok()?;
        let flags = libc::O_RDONLY
            | libc::O_NOFOLLOW
            | libc::O_CLOEXEC
            | if dir {
                libc::O_DIRECTORY
            } else {
                libc::O_NONBLOCK
            };
        let fd = unsafe { libc::openat(current.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return None;
        }
        current = unsafe { File::from_raw_fd(fd) };
        trusted(&current, owner, dir)?;
    }
    Some(current)
}

#[cfg(target_os = "linux")]
fn read_stable(mut file: File, owner: u32, limit: u64) -> Option<Vec<u8>> {
    let before = trusted(&file, owner, false)?;
    if before.len() > limit {
        return None;
    }
    let mut bytes = Vec::new();
    (&mut file).take(limit + 1).read_to_end(&mut bytes).ok()?;
    let after = trusted(&file, owner, false)?;
    if bytes.len() as u64 > limit
        || before.len() != bytes.len() as u64
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || before.mode() != after.mode()
        || before.nlink() != after.nlink()
    {
        return None;
    }
    Some(bytes)
}
