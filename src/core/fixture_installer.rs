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

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    dev: u64,
    ino: u64,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
enum Phase {
    Prepared,
    BinaryStaged,
    BothStaged,
    BinaryPublished,
    BothPublished,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    phase: Phase,
    binary: Option<Identity>,
    record: Option<Identity>,
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
            "journal.next",
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
    fn update_journal(&self, journal: &Journal, interrupted: bool) -> io::Result<()> {
        self.create("journal.next", &serde_json::to_vec(journal)?)?;
        if interrupted {
            return Err(io::Error::other("injected journal update"));
        }
        // Exclusive fixture root: replace only our previously prepared journal.
        fs::rename(self.path("journal.next"), self.path("journal"))
    }
    // Failure points model process interruption; not an actual power failure.
    fn install(&self, failure: Option<u8>, after_preflight: impl FnOnce(&Path)) -> io::Result<()> {
        self.preflight()?;
        after_preflight(&self.root);
        if failure == Some(10) {
            return Err(io::Error::other("injected before journal"));
        }
        let mut journal = Journal {
            phase: Phase::Prepared,
            binary: None,
            record: None,
        };
        self.create("journal", &serde_json::to_vec(&journal)?)?;
        if failure == Some(4) {
            return Err(io::Error::other("injected prepared"));
        }
        let binary = self.create("binary.tmp", BINARY)?;
        if failure == Some(5) {
            return Err(io::Error::other("injected unrecorded binary"));
        }
        journal.binary = Some(Identity {
            dev: binary.dev(),
            ino: binary.ino(),
        });
        journal.phase = Phase::BinaryStaged;
        self.update_journal(&journal, failure == Some(9))?;
        if failure == Some(6) {
            return Err(io::Error::other("injected binary staged"));
        }
        let record = self.create("registry.tmp", RECORD)?;
        if failure == Some(3) {
            return Err(io::Error::other("injected unrecorded registry"));
        }
        journal.record = Some(Identity {
            dev: record.dev(),
            ino: record.ino(),
        });
        journal.phase = Phase::BothStaged;
        self.update_journal(&journal, false)?;
        if failure == Some(0) {
            return Err(io::Error::other("injected staged"));
        }
        self.publish("binary.tmp", "binary")?;
        if failure == Some(1) {
            return Err(io::Error::other("injected binary published"));
        }
        journal.phase = Phase::BinaryPublished;
        self.update_journal(&journal, false)?;
        if failure == Some(7) {
            return Err(io::Error::other("injected binary recorded"));
        }
        self.publish("registry.tmp", "registry")?;
        if failure == Some(2) {
            return Err(io::Error::other("injected registry published"));
        }
        journal.phase = Phase::BothPublished;
        self.update_journal(&journal, false)?;
        if failure == Some(8) {
            return Err(io::Error::other("injected complete"));
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
            || m.len() != content.len() as u64
            || fs::read(path)? != content
        {
            return Err(io::Error::other("intervención manual: archivo ambiguo"));
        }
        Ok(true)
    }
    fn recover(&self) -> io::Result<()> {
        let journal_path = self.path("journal");
        if let Ok(metadata) = fs::symlink_metadata(&journal_path) {
            if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > 1024 {
                return Err(io::Error::other("intervención manual: journal inválido"));
            }
        }
        let j = match fs::read(&journal_path) {
            Ok(bytes) => serde_json::from_slice::<Journal>(&bytes)
                .map_err(|_| io::Error::other("intervención manual: journal inválido"))?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if [
                    "binary",
                    "registry",
                    "binary.tmp",
                    "registry.tmp",
                    "journal.next",
                ]
                .iter()
                .any(|name| self.exists(name).unwrap_or(true))
                {
                    return Err(io::Error::other("intervención manual: falta journal"));
                }
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        if self.exists("journal.next")? {
            return Err(io::Error::other("intervención manual: journal temporal"));
        }
        let valid = match j.phase {
            Phase::Prepared => j.binary.is_none() && j.record.is_none(),
            Phase::BinaryStaged => j.binary.is_some() && j.record.is_none(),
            Phase::BothStaged | Phase::BinaryPublished | Phase::BothPublished => {
                j.binary.is_some() && j.record.is_some()
            }
        };
        if !valid {
            return Err(io::Error::other(
                "intervención manual: journal inconsistente",
            ));
        }
        let mut owned = Vec::new();
        for (names, identity, content) in [
            (["binary", "binary.tmp"], j.binary, BINARY),
            (["registry", "registry.tmp"], j.record, RECORD),
        ] {
            for name in names {
                match identity {
                    Some(id) => {
                        if self.check_owned(name, id.dev, id.ino, content)? {
                            owned.push(name);
                        }
                    }
                    None => {
                        if self.exists(name)? {
                            return Err(io::Error::other(
                                "intervención manual: identidad desconocida",
                            ));
                        }
                    }
                }
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
    for step in [0, 1, 2, 4, 6, 7, 8, 10] {
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
            "journal.next",
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
fn failure_before_registry_identity_preserves_ambiguous_files() {
    let fixture = Fixture::new();
    assert!(fixture.install(Some(3), |_| {}).is_err());
    assert!(fixture.exists("journal").unwrap());
    assert!(fixture.recover().is_err());
    assert_eq!(fs::read(fixture.path("binary.tmp")).unwrap(), BINARY);
    assert_eq!(fs::read(fixture.path("registry.tmp")).unwrap(), RECORD);
}

#[test]
fn unrecorded_files_require_manual_intervention_and_are_preserved() {
    for (step, filename, bytes) in [(5, "binary.tmp", BINARY), (3, "registry.tmp", RECORD)] {
        let fixture = Fixture::new();
        assert!(fixture.install(Some(step), |_| {}).is_err());
        let error = fixture.recover().unwrap_err();
        assert!(error.to_string().contains("intervención manual"));
        assert_eq!(fs::read(fixture.path(filename)).unwrap(), bytes);
        assert!(fixture.exists("journal").unwrap());
        assert!(fixture
            .recover()
            .unwrap_err()
            .to_string()
            .contains("intervención manual"));
    }
}

#[test]
fn foreign_pending_journal_is_never_removed() {
    let fixture = Fixture::new();
    assert!(fixture.install(Some(6), |_| {}).is_err());
    fs::write(fixture.path("journal.next"), b"foreign").unwrap();
    assert!(fixture
        .recover()
        .unwrap_err()
        .to_string()
        .contains("intervención manual"));
    assert_eq!(fs::read(fixture.path("journal.next")).unwrap(), b"foreign");
    assert_eq!(fs::read(fixture.path("binary.tmp")).unwrap(), BINARY);
}

#[test]
fn interrupted_journal_update_requires_manual_intervention() {
    let fixture = Fixture::new();
    assert!(fixture.install(Some(9), |_| {}).is_err());
    assert!(fixture
        .recover()
        .unwrap_err()
        .to_string()
        .contains("intervención manual"));
    assert_eq!(fs::read(fixture.path("binary.tmp")).unwrap(), BINARY);
    assert!(fixture.exists("journal.next").unwrap());
}

#[test]
fn malformed_journal_preserves_everything() {
    let fixture = Fixture::new();
    assert!(fixture.install(Some(6), |_| {}).is_err());
    fs::write(fixture.path("journal"), b"partial").unwrap();
    assert!(fixture
        .recover()
        .unwrap_err()
        .to_string()
        .contains("intervención manual"));
    assert_eq!(fs::read(fixture.path("binary.tmp")).unwrap(), BINARY);
    assert_eq!(fs::read(fixture.path("journal")).unwrap(), b"partial");
}

#[test]
fn interrupted_recovery_continues_when_owned_file_already_removed() {
    let fixture = Fixture::new();
    fixture.install(None, |_| {}).unwrap();
    // Model a stop after the first owned deletion, leaving the journal intact.
    fs::remove_file(fixture.path("binary")).unwrap();
    fixture.recover().unwrap();
    fixture.recover().unwrap();
    for name in ["binary", "registry", "journal", "journal.next"] {
        assert!(!fixture.exists(name).unwrap());
    }
}

#[test]
fn inconsistent_journal_does_not_remove_fixture_files() {
    let fixture = Fixture::new();
    assert!(fixture.install(Some(6), |_| {}).is_err());
    let path = fixture.path("journal");
    let mut journal: Journal = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    journal.phase = Phase::Prepared; // Contradicts the recorded binary identity.
    fs::write(&path, serde_json::to_vec(&journal).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    assert!(fixture
        .recover()
        .unwrap_err()
        .to_string()
        .contains("intervención manual"));
    assert_eq!(fs::read(fixture.path("binary.tmp")).unwrap(), BINARY);
    assert_eq!(fs::read(path).unwrap(), before);
}
