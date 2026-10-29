//! Test-only fixture installer. The private root has no hostile concurrent writer.
//! Never compiled into production; no host destination is accepted. Per-file rename
//! is atomic, but two files are not a transaction and power-loss durability is unproven.
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BINARY: &[u8] = b"fixture executable, not a host binary";
const RECORD: &[u8] = br#"{"schema_version":1,"product_version":"0.1.0","executable":"/usr/local/bin/javipiv2","sha256":"d68067c32c21853f54e4bd4b50a28d115250619c7f144dece38ebcdf25a5492b"}"#;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    binary_dev: u64,
    binary_ino: u64,
    record_dev: u64,
    record_ino: u64,
}
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        assert!(
            parent.is_dir()
                && !fs::symlink_metadata(&parent)
                    .unwrap()
                    .file_type()
                    .is_symlink()
        );
        loop {
            let name = format!(
                "fixture-installer-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let root = parent.join(name);
            match fs::DirBuilder::new().mode(0o700).create(&root) {
                Ok(()) => return Self { root },
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("fixture root: {e}"),
            }
        }
    }
    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
    fn exists(&self, name: &str) -> io::Result<bool> {
        match fs::symlink_metadata(self.path(name)) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
    fn preflight(&self) -> io::Result<()> {
        for name in [
            "binary",
            "registry",
            "binary.tmp",
            "registry.tmp",
            "journal",
        ] {
            if self.exists(name)? {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "fixture collision",
                ));
            }
        }
        Ok(())
    }
    fn create(&self, name: &str, content: &[u8]) -> io::Result<fs::Metadata> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(if name == "binary.tmp" { 0o755 } else { 0o600 })
            .open(self.path(name))?;
        f.write_all(content)?;
        f.sync_all()?;
        f.metadata()
    }
    fn publish(&self, from: &str, to: &str) -> io::Result<()> {
        let from = CString::new(self.path(from).as_os_str().as_bytes()).unwrap();
        let to = CString::new(self.path(to).as_os_str().as_bytes()).unwrap();
        let result = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                from.as_ptr(),
                libc::AT_FDCWD,
                to.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    // Failure points model process interruption; not an actual power failure.
    fn install(&self, failure: Option<u8>, after_preflight: impl FnOnce(&Path)) -> io::Result<()> {
        self.preflight()?;
        after_preflight(&self.root);
        let binary = self.create("binary.tmp", BINARY)?;
        let record = self.create("registry.tmp", RECORD)?;
        if failure == Some(3) {
            return Err(io::Error::other("injected before journal"));
        }
        let journal = Journal {
            binary_dev: binary.dev(),
            binary_ino: binary.ino(),
            record_dev: record.dev(),
            record_ino: record.ino(),
        };
        self.create("journal", &serde_json::to_vec(&journal)?)?;
        if failure == Some(0) {
            return Err(io::Error::other("injected"));
        }
        self.publish("binary.tmp", "binary")?;
        if failure == Some(1) {
            return Err(io::Error::other("injected"));
        }
        self.publish("registry.tmp", "registry")?;
        if failure == Some(2) {
            return Err(io::Error::other("injected"));
        }
        Ok(())
    }
    fn check_owned(&self, name: &str, dev: u64, ino: u64, content: &[u8]) -> io::Result<bool> {
        let path = self.path(name);
        let m = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(e),
        };
        if !m.is_file()
            || m.nlink() != 1
            || m.dev() != dev
            || m.ino() != ino
            || fs::read(path)? != content
        {
            return Err(io::Error::other("ambiguous fixture state"));
        }
        Ok(true)
    }
    fn recover(&self) -> io::Result<()> {
        let journal_path = self.path("journal");
        if let Ok(metadata) = fs::symlink_metadata(&journal_path) {
            if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > 1024 {
                return Err(io::Error::other("invalid journal"));
            }
        }
        let j = match fs::read(&journal_path) {
            Ok(bytes) => serde_json::from_slice::<Journal>(&bytes)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if ["binary", "registry", "binary.tmp", "registry.tmp"]
                    .iter()
                    .any(|name| self.exists(name).unwrap_or(true))
                {
                    return Err(io::Error::other("missing journal"));
                }
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        let mut owned = Vec::new();
        for (name, dev, ino, content) in [
            ("binary", j.binary_dev, j.binary_ino, BINARY),
            ("binary.tmp", j.binary_dev, j.binary_ino, BINARY),
            ("registry", j.record_dev, j.record_ino, RECORD),
            ("registry.tmp", j.record_dev, j.record_ino, RECORD),
        ] {
            if self.check_owned(name, dev, ino, content)? {
                owned.push(name);
            }
        }
        for name in owned {
            fs::remove_file(self.path(name))?;
        }
        fs::remove_file(journal_path)?;
        Ok(())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn fixture_publication_and_recovery() {
    let fixture = Fixture::new();
    fixture.install(None, |_| {}).unwrap();
    assert_eq!(fs::read(fixture.path("binary")).unwrap(), BINARY);
    assert_eq!(fs::read(fixture.path("registry")).unwrap(), RECORD);
    let record: serde_json::Value = serde_json::from_slice(RECORD).unwrap();
    use sha2::Digest;
    assert_eq!(
        record["sha256"],
        format!("{:x}", sha2::Sha256::digest(BINARY))
    );
    fixture.recover().unwrap();
    fixture.recover().unwrap();
    assert!(!fixture.exists("binary").unwrap());
    assert!(!fixture.exists("registry").unwrap());
}

#[test]
fn interrupted_steps_are_recovered_idempotently() {
    for step in 0..=2 {
        let fixture = Fixture::new();
        assert!(fixture.install(Some(step), |_| {}).is_err());
        // Reconstruct from the persisted fixture root, without in-memory transaction state.
        let reopened = std::mem::ManuallyDrop::new(Fixture {
            root: fixture.root.clone(),
        });
        reopened.recover().unwrap();
        reopened.recover().unwrap();
        for name in [
            "binary",
            "registry",
            "binary.tmp",
            "registry.tmp",
            "journal",
        ] {
            assert!(!fixture.exists(name).unwrap());
        }
    }
}

#[test]
fn collisions_and_post_preflight_races_do_not_overwrite() {
    let fixture = Fixture::new();
    fs::write(fixture.path("binary"), b"foreign").unwrap();
    assert!(fixture.install(None, |_| {}).is_err());
    assert_eq!(fs::read(fixture.path("binary")).unwrap(), b"foreign");
    assert!(!fixture.exists("journal").unwrap());
    let second = Fixture::new();
    assert!(second
        .install(None, |root| {
            fs::write(root.join("binary"), b"foreign").unwrap();
        })
        .is_err());
    assert_eq!(fs::read(second.path("binary")).unwrap(), b"foreign");
    assert!(second.recover().is_err());
    assert_eq!(fs::read(second.path("binary")).unwrap(), b"foreign");
}

#[test]
fn modified_or_replaced_entry_aborts_without_removing_other_files() {
    let fixture = Fixture::new();
    fixture.install(None, |_| {}).unwrap();
    fs::write(fixture.path("binary"), b"changed").unwrap();
    assert!(fixture.recover().is_err());
    assert_eq!(fs::read(fixture.path("binary")).unwrap(), b"changed");
    assert_eq!(fs::read(fixture.path("registry")).unwrap(), RECORD);
    let second = Fixture::new();
    second.install(None, |_| {}).unwrap();
    fs::remove_file(second.path("registry")).unwrap();
    std::os::unix::fs::symlink("binary", second.path("registry")).unwrap();
    assert!(second.recover().is_err());
    assert_eq!(fs::read(second.path("binary")).unwrap(), BINARY);
    assert!(fs::symlink_metadata(second.path("registry"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn preflight_rejects_directory_and_dangling_symlink() {
    let a = Fixture::new();
    fs::create_dir(a.path("binary")).unwrap();
    assert!(a.install(None, |_| {}).is_err());
    assert!(a.path("binary").is_dir());
    let b = Fixture::new();
    std::os::unix::fs::symlink("missing", b.path("registry")).unwrap();
    assert!(b.install(None, |_| {}).is_err());
    assert!(fs::symlink_metadata(b.path("registry"))
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(!b.exists("journal").unwrap());
}

#[test]
fn failure_before_journal_preserves_ambiguous_files() {
    let fixture = Fixture::new();
    assert!(fixture.install(Some(3), |_| {}).is_err());
    assert!(fixture.recover().is_err());
    assert_eq!(fs::read(fixture.path("binary.tmp")).unwrap(), BINARY);
    assert_eq!(fs::read(fixture.path("registry.tmp")).unwrap(), RECORD);
}
