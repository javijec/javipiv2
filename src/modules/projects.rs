//! Inventario de lectura: no invoca Git ni sigue enlaces de metadatos.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

const NOFOLLOW: i32 = 0x20000;
const DIRECTORY: i32 = 0x10000;
const NONBLOCK: i32 = 0x800;
const LIMIT: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitState { Compatible, NotGit, Unknown }

#[derive(Debug, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub git: GitState,
    pub branch: Option<String>,
    pub head: Option<String>,
}

/// Acepta únicamente un montaje ext4 inequívoco en /srv, sin montajes descendientes.
/// No demuestra identidad física; el estado puede cambiar después de esta lectura.
pub fn compatible_mountinfo(raw: &str) -> bool {
    let mut found = 0;
    for line in raw.lines() {
        let Some((left, right)) = line.split_once(" - ") else { return false };
        let fields: Vec<_> = left.split_whitespace().collect();
        let after: Vec<_> = right.split_whitespace().collect();
        if fields.len() < 6 || after.len() < 3 { return false; }
        // No interpretar rutas escapadas: podrían ocultar un montaje relevante.
        if fields[4].contains('\\') { return false; }
        if fields[4] == "/srv" {
            found += 1;
            if fields[3] != "/" || after[0] != "ext4" || after[1] == "none" || !fields[2].split_once(':').is_some_and(|(a,b)| a.parse::<u32>().is_ok() && b.parse::<u32>().is_ok() && a != "0") { return false; }
        } else if fields[4].starts_with("/srv/") { return false; }
    }
    found == 1
}

fn name_ok(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\0')
}

fn project_name(name: std::ffi::OsString) -> Result<String, String> {
    let name = name.into_string().map_err(|_| "workspaces: unknown (nombre de proyecto no UTF-8)".to_owned())?;
    if !name_ok(&name) { return Err("workspaces: unknown (nombre de proyecto inválido)".into()); }
    Ok(name)
}

fn valid_branch(name: &str) -> bool {
    !name.is_empty() && !name.contains("..") && !name.contains("@{") && !name.ends_with('.')
        && name.split('/').all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
        && !name.bytes().any(|b| b <= 0x20 || b == 0x7f || matches!(b, b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\'))
}

fn open_child(parent: &File, name: &str, directory: bool) -> io::Result<File> {
    let path = format!("/proc/self/fd/{}/{name}", std::os::fd::AsRawFd::as_raw_fd(parent));
    OpenOptions::new().read(true).custom_flags(NOFOLLOW | NONBLOCK | if directory { DIRECTORY } else { 0 }).open(path)
}

fn text(parent: &File, name: &str) -> io::Result<String> {
    let file = open_child(parent, name, false)?;
    if !file.metadata()?.is_file() { return Err(io::ErrorKind::InvalidData.into()); }
    let mut raw = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > LIMIT { return Err(io::ErrorKind::InvalidData.into()); }
    String::from_utf8(raw).map_err(|_| io::ErrorKind::InvalidData.into())
}

fn loose_ref(git: &File, components: &[&str]) -> io::Result<String> {
    let refs = open_child(git, "refs", true)?;
    let mut dir = open_child(&refs, "heads", true)?;
    for part in &components[..components.len()-1] { dir = open_child(&dir, part, true)?; }
    text(&dir, components[components.len()-1])
}

fn oid(s: &str) -> Option<String> {
    let s = s.trim();
    if matches!(s.len(), 40 | 64) && s.bytes().all(|b| b.is_ascii_hexdigit()) { Some(s.to_ascii_lowercase()) } else { None }
}

fn project(parent: &File, name: &str) -> Project {
    let mut result = Project { name: name.into(), git: GitState::Unknown, branch: None, head: None };
    let Ok(dir) = open_child(parent, name, true) else { return result };
    let git = match open_child(&dir, ".git", true) {
        Ok(git) => git,
        Err(e) if e.kind() == io::ErrorKind::NotFound => { result.git = GitState::NotGit; return result; }
        Err(_) => return result,
    };
    // Un archivo .git de worktree, enlaces, y metadatos incompletos quedan unknown.
    if open_child(&git, "objects", true).is_err() || open_child(&git, "refs", true).is_err() { return result; }
    let Ok(head) = text(&git, "HEAD") else { return result };
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref: refs/heads/") {
        if !valid_branch(reference) { return result; }
        let components: Vec<_> = reference.split('/').collect();
        result.branch = Some(reference.to_owned());
        match loose_ref(&git, &components) {
            Ok(raw) => match oid(&raw) {
                Some(id) => result.head = Some(id),
                None => return result,
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                match text(&git, "packed-refs") {
                    Ok(packed) => {
                        let target = format!("refs/heads/{reference}");
                        let matches: Vec<_> = packed.lines().filter_map(|line| line.split_once(' ')).filter(|(_, r)| *r == target).collect();
                        if matches.len() > 1 { return result; }
                        if let Some((value, _)) = matches.first() {
                            let Some(id) = oid(value) else { return result };
                            result.head = Some(id);
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {},
                    Err(_) => return result,
                }
            }
            Err(_) => return result,
        }
        result.git = GitState::Compatible;
    } else if let Some(id) = oid(head) {
        result.git = GitState::Compatible;
        result.head = Some(id);
    }
    result
}

/// Configuración explícita: ruta absoluta bajo /srv, sin enlaces en ningún componente.
pub struct Projects { workspace: PathBuf }
impl Projects {
    pub fn new(workspace: impl Into<PathBuf>) -> Result<Self, String> {
        let workspace = workspace.into();
        let mut parts = workspace.components();
        if parts.next() != Some(Component::RootDir) || parts.next() != Some(Component::Normal("srv".as_ref())) || !parts.clone().all(|p| matches!(p, Component::Normal(_))) || workspace == Path::new("/srv") {
            return Err("ruta de workspaces inválida: debe estar bajo /srv".into());
        }
        // Componentes normalizados por Path no deben ocultar . o .. en la entrada.
        if workspace.as_os_str().as_encoded_bytes().split(|b| *b == b'/').any(|p| p == b"." || p == b"..") { return Err("ruta de workspaces inválida".into()); }
        Ok(Self { workspace })
    }
    fn open(&self) -> Result<File, String> {
        let mounts = fs::read_to_string("/proc/self/mountinfo").map_err(|_| "workspaces: unknown (mountinfo no disponible)".to_owned())?;
        if !compatible_mountinfo(&mounts) { return Err("workspaces: unknown (montaje /srv no confirmado)".into()); }
        let mut dir = OpenOptions::new().read(true).custom_flags(NOFOLLOW | DIRECTORY).open("/").map_err(|_| "workspaces: unknown".to_owned())?;
        for part in self.workspace.components().skip(1) {
            let Component::Normal(part) = part else { return Err("workspaces: unknown".into()) };
            let part = part.to_str().ok_or("workspaces: unknown")?;
            dir = open_child(&dir, part, true).map_err(|_| "workspaces: unknown (ruta inaccesible o enlace)".to_owned())?;
        }
        Ok(dir)
    }
    pub fn list(&self) -> Result<Vec<Project>, String> {
        let dir = self.open()?;
        let path = format!("/proc/self/fd/{}", std::os::fd::AsRawFd::as_raw_fd(&dir));
        let entries = fs::read_dir(path).map_err(|_| "workspaces: unknown (lectura fallida)".to_owned())?;
        let mut projects = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| "workspaces: unknown (lectura fallida)".to_owned())?;
            let kind = entry.file_type().map_err(|_| "workspaces: unknown (lectura fallida)".to_owned())?;
            if kind.is_dir() {
                let name = project_name(entry.file_name())?;
                projects.push(project(&dir, &name));
            }
        }
        projects.sort_by(|a,b| a.name.cmp(&b.name));
        Ok(projects)
    }
    pub fn status(&self, name: &str) -> Result<Project, String> {
        if !name_ok(name) { return Err("nombre de proyecto inválido".into()); }
        let dir = self.open()?;
        Ok(project(&dir, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mount_evidence_is_conservative() {
        let good = "42 1 8:1 / /srv rw - ext4 /dev/sda1 rw\n";
        assert!(compatible_mountinfo(good));
        assert!(!compatible_mountinfo(&good.replace("ext4", "autofs")));
        assert!(!compatible_mountinfo(&good.replace(" / /srv ", " /sub /srv ")));
        assert!(!compatible_mountinfo(&format!("{good}{good}")));
        assert!(!compatible_mountinfo(&format!("{good}43 42 0:9 / /srv/workspaces rw - autofs none rw\n")));
    }
    #[test]
    fn reads_only_direct_metadata_without_following_links() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("javipiv2-project-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let parent = File::open(&root).unwrap();
        fs::create_dir(root.join("plain")).unwrap();
        assert_eq!(project(&parent, "plain").git, GitState::NotGit);
        fs::create_dir_all(root.join("repo/.git/objects")).unwrap();
        fs::create_dir_all(root.join("repo/.git/refs/heads")).unwrap();
        fs::write(root.join("repo/.git/HEAD"), "ref: refs/heads/main\n").unwrap();
        let hash = "1234567890123456789012345678901234567890";
        fs::write(root.join("repo/.git/refs/heads/main"), hash).unwrap();
        let repo = project(&parent, "repo");
        assert_eq!(repo.git, GitState::Compatible);
        assert_eq!(repo.branch.as_deref(), Some("main"));
        assert_eq!(repo.head.as_deref(), Some(hash));
        symlink(root.join("repo/.git"), root.join("plain/.git")).unwrap();
        assert_eq!(project(&parent, "plain").git, GitState::Unknown);
        drop(parent);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn accepts_safe_project_and_git_branch_names() {
        for name in ["my project", "proyecto+uno", "proyecto á"] { assert!(name_ok(name)); }
        use std::os::unix::ffi::OsStringExt;
        assert!(project_name(std::ffi::OsString::from_vec(vec![0xff])).is_err());
        for branch in ["feature+test", "feature/nested", "á"] { assert!(valid_branch(branch)); }
        for branch in ["a/../b", "a b", "a.lock", "a//b", "a@{b"] { assert!(!valid_branch(branch)); }
    }
    #[test]
    fn packed_nested_ref_and_invalid_loose_ref() {
        let root = std::env::temp_dir().join(format!("javipiv2-packed-test-{}", std::process::id()));
        fs::create_dir_all(root.join("repo/.git/objects")).unwrap();
        fs::create_dir_all(root.join("repo/.git/refs/heads")).unwrap();
        fs::write(root.join("repo/.git/HEAD"), "ref: refs/heads/feature+test/nested\n").unwrap();
        let hash = "1234567890123456789012345678901234567890";
        fs::write(root.join("repo/.git/packed-refs"), format!("{hash} refs/heads/feature+test/nested\n")).unwrap();
        let parent = File::open(&root).unwrap();
        let packed = project(&parent, "repo");
        assert_eq!(packed.head.as_deref(), Some(hash));
        assert_eq!(packed.git, GitState::Compatible);
        fs::create_dir(root.join("repo/.git/refs/heads/feature+test")).unwrap();
        fs::write(root.join("repo/.git/refs/heads/feature+test/nested"), "invalid\n").unwrap();
        let invalid = project(&parent, "repo");
        assert_eq!(invalid.git, GitState::Unknown);
        assert_eq!(invalid.head, None);
        fs::remove_file(root.join("repo/.git/refs/heads/feature+test/nested")).unwrap();
        std::os::unix::fs::symlink("missing", root.join("repo/.git/refs/heads/feature+test/nested")).unwrap();
        let link = project(&parent, "repo");
        assert_eq!(link.git, GitState::Unknown);
        assert_eq!(link.head, None);
        drop(parent);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_paths_and_names() {
        assert!(Projects::new("/srv/workspaces").is_ok());
        for path in ["/tmp", "/srv/../tmp", "/srv", "relative"] { assert!(Projects::new(path).is_err()); }
        for name in ["", ".", "..", "a/b", "../a"] { assert!(!name_ok(name)); }
    }
}
