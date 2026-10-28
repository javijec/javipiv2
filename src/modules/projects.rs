//! Inventario de lectura: no invoca Git ni sigue enlaces de metadatos.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

const LIMIT: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitState {
    Compatible,
    NotGit,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub git: GitState,
    pub branch: Option<String>,
    pub head: Option<String>,
}

/// Evidencia pasiva; no demuestra identidad física ni evita cambios posteriores.
pub fn compatible_mountinfo(raw: &str) -> bool {
    compatible_for(raw, Path::new("/srv/workspaces"), None)
}

fn mount_path(s: &str) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let code = bytes.get(i + 1..i + 4)?;
            out.push(match code {
                b"040" => b' ',
                b"011" => b'\t',
                b"012" => b'\n',
                b"134" => b'\\',
                _ => return None,
            });
            i += 4;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let path = PathBuf::from(std::ffi::OsString::from_vec(out));
    if !path.is_absolute()
        || path
            .as_os_str()
            .as_encoded_bytes()
            .split(|b| *b == b'/')
            .any(|p| p == b"." || p == b"..")
    {
        return None;
    }
    Some(path)
}

fn compatible_for(raw: &str, workspace: &Path, name: Option<&str>) -> bool {
    let target = name.map_or_else(|| workspace.to_path_buf(), |n| workspace.join(n));
    let mut ids = std::collections::HashSet::new();
    let mut srv = Vec::new();
    for line in raw.lines() {
        let Some((left, right)) = line.split_once(" - ") else {
            return false;
        };
        let fields: Vec<_> = left.split_whitespace().collect();
        let after: Vec<_> = right.split_whitespace().collect();
        if fields.len() < 6 || after.len() < 3 {
            return false;
        }
        let (Ok(id), Ok(parent)) = (fields[0].parse::<u64>(), fields[1].parse::<u64>()) else {
            return false;
        };
        if id == 0 || parent == 0 || !ids.insert(id) {
            return false;
        }
        let Some(path) = mount_path(fields[4]) else {
            return false;
        };
        // Las raíces de otros sistemas, como nsfs, no son necesariamente rutas.
        // La raíz de la pila /srv se valida más abajo antes de aceptarla.
        if path == Path::new("/srv") {
            srv.push((id, parent, fields[3], fields[2], after[0], after[1]));
        } else if path.starts_with("/srv")
            && (target.starts_with(&path) || path.starts_with(&target))
        {
            return false;
        }
    }
    let valid_ext = |entry: &(u64, u64, &str, &str, &str, &str)| {
        entry.2 == "/"
            && entry.4 == "ext4"
            && entry.5 != "none"
            && entry.3.split_once(':').is_some_and(|(a, b)| {
                a.parse::<u32>().is_ok_and(|n| n != 0) && b.parse::<u32>().is_ok()
            })
    };
    match srv.as_slice() {
        [ext] => valid_ext(ext),
        [a, b] => {
            let (auto, ext) = if a.4 == "autofs" { (a, b) } else { (b, a) };
            auto.4 == "autofs"
                && auto.2 == "/"
                && auto.1 != ext.0
                && ext.1 == auto.0
                && valid_ext(ext)
        }
        _ => false,
    }
}

fn name_ok(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\0')
}

fn project_name(name: std::ffi::OsString) -> Result<String, String> {
    let name = name
        .into_string()
        .map_err(|_| "workspaces: unknown (nombre de proyecto no UTF-8)".to_owned())?;
    if !name_ok(&name) {
        return Err("workspaces: unknown (nombre de proyecto inválido)".into());
    }
    Ok(name)
}

fn valid_branch(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains("@{")
        && !name.ends_with('.')
        && name
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
        && !name.bytes().any(|b| {
            b <= 0x20 || b == 0x7f || matches!(b, b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\')
        })
}

fn open_child(parent: &File, name: &str, directory: bool) -> io::Result<File> {
    let path = format!(
        "/proc/self/fd/{}/{name}",
        std::os::fd::AsRawFd::as_raw_fd(parent)
    );
    OpenOptions::new()
        .read(true)
        .custom_flags(
            libc::O_NOFOLLOW | libc::O_NONBLOCK | if directory { libc::O_DIRECTORY } else { 0 },
        )
        .open(path)
}

fn text(parent: &File, name: &str) -> io::Result<String> {
    let file = open_child(parent, name, false)?;
    if !file.metadata()?.is_file() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut raw = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > LIMIT {
        return Err(io::ErrorKind::InvalidData.into());
    }
    String::from_utf8(raw).map_err(|_| io::ErrorKind::InvalidData.into())
}

fn loose_ref(git: &File, components: &[&str]) -> io::Result<String> {
    let refs = open_child(git, "refs", true)?;
    let mut dir = open_child(&refs, "heads", true)?;
    for part in &components[..components.len() - 1] {
        dir = open_child(&dir, part, true)?;
    }
    text(&dir, components[components.len() - 1])
}

fn oid(s: &str) -> Option<String> {
    let s = s.trim();
    if matches!(s.len(), 40 | 64) && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(s.to_ascii_lowercase())
    } else {
        None
    }
}

fn project(parent: &File, name: &str) -> Project {
    let mut result = Project {
        name: name.into(),
        git: GitState::Unknown,
        branch: None,
        head: None,
    };
    let Ok(dir) = open_child(parent, name, true) else {
        return result;
    };
    let git = match open_child(&dir, ".git", true) {
        Ok(git) => git,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            result.git = GitState::NotGit;
            return result;
        }
        Err(_) => return result,
    };
    // Un archivo .git de worktree, enlaces, y metadatos incompletos quedan unknown.
    if open_child(&git, "objects", true).is_err() || open_child(&git, "refs", true).is_err() {
        return result;
    }
    let Ok(head) = text(&git, "HEAD") else {
        return result;
    };
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref: refs/heads/") {
        if !valid_branch(reference) {
            return result;
        }
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
                        let matches: Vec<_> = packed
                            .lines()
                            .filter_map(|line| line.split_once(' '))
                            .filter(|(_, r)| *r == target)
                            .collect();
                        if matches.len() > 1 {
                            return result;
                        }
                        if let Some((value, _)) = matches.first() {
                            let Some(id) = oid(value) else { return result };
                            result.head = Some(id);
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
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
pub struct Projects {
    workspace: PathBuf,
}
impl Projects {
    pub fn new(workspace: impl Into<PathBuf>) -> Result<Self, String> {
        let workspace = workspace.into();
        let mut parts = workspace.components();
        if parts.next() != Some(Component::RootDir)
            || parts.next() != Some(Component::Normal("srv".as_ref()))
            || !parts.clone().all(|p| matches!(p, Component::Normal(_)))
            || workspace == Path::new("/srv")
        {
            return Err("ruta de workspaces inválida: debe estar bajo /srv".into());
        }
        // Componentes normalizados por Path no deben ocultar . o .. en la entrada.
        if workspace
            .as_os_str()
            .as_encoded_bytes()
            .split(|b| *b == b'/')
            .any(|p| p == b"." || p == b"..")
        {
            return Err("ruta de workspaces inválida".into());
        }
        Ok(Self { workspace })
    }
    fn open(&self, name: Option<&str>) -> Result<File, String> {
        let mounts = fs::read_to_string("/proc/self/mountinfo")
            .map_err(|_| "workspaces: unknown (mountinfo no disponible)".to_owned())?;
        if !compatible_for(&mounts, &self.workspace, name) {
            return Err("workspaces: unknown (montaje /srv no confirmado)".into());
        }
        let mut dir = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY)
            .open("/")
            .map_err(|_| "workspaces: unknown".to_owned())?;
        for part in self.workspace.components().skip(1) {
            let Component::Normal(part) = part else {
                return Err("workspaces: unknown".into());
            };
            let part = part.to_str().ok_or("workspaces: unknown")?;
            dir = open_child(&dir, part, true)
                .map_err(|_| "workspaces: unknown (ruta inaccesible o enlace)".to_owned())?;
        }
        Ok(dir)
    }
    pub fn list(&self) -> Result<Vec<Project>, String> {
        let dir = self.open(None)?;
        let path = format!("/proc/self/fd/{}", std::os::fd::AsRawFd::as_raw_fd(&dir));
        let entries =
            fs::read_dir(path).map_err(|_| "workspaces: unknown (lectura fallida)".to_owned())?;
        let mut projects = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| "workspaces: unknown (lectura fallida)".to_owned())?;
            let kind = entry
                .file_type()
                .map_err(|_| "workspaces: unknown (lectura fallida)".to_owned())?;
            if kind.is_dir() {
                let name = project_name(entry.file_name())?;
                projects.push(project(&dir, &name));
            }
        }
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(projects)
    }
    pub fn status(&self, name: &str) -> Result<Project, String> {
        if !name_ok(name) {
            return Err("nombre de proyecto inválido".into());
        }
        let dir = self.open(Some(name))?;
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
        assert!(!compatible_mountinfo(
            &good.replace(" / /srv ", " /sub /srv ")
        ));
        assert!(!compatible_mountinfo(&format!("{good}{good}")));
        assert!(!compatible_mountinfo(&format!(
            "{good}43 42 0:9 / /srv/workspaces rw - autofs none rw\n"
        )));
    }
    #[test]
    fn mount_stack_and_query_scope() {
        let stack =
            "10 1 0:5 / /srv rw - autofs systemd-1 rw\n11 10 8:1 / /srv rw - ext4 /dev/sda1 rw\n";
        let ws = Path::new("/srv/workspaces");
        assert!(compatible_for(stack, ws, None));
        assert!(compatible_for(
            &format!("{stack}12 11 8:2 / /srv/docker rw - ext4 /dev/sdb rw\n"),
            ws,
            None
        ));
        assert!(!compatible_for(&stack.replace("11 10", "11 1"), ws, None));
        assert!(!compatible_for(
            &format!("{stack}12 11 8:2 / /srv/workspaces/a/.git rw - ext4 /dev/sdb rw\n"),
            ws,
            Some("a")
        ));
        assert!(compatible_for(
            &format!("{stack}12 11 8:2 / /srv/workspaces/b rw - ext4 /dev/sdb rw\n"),
            ws,
            Some("a")
        ));
        assert!(!compatible_for(
            &format!("{stack}12 11 8:2 / /srv/workspaces/b rw - ext4 /dev/sdb rw\n"),
            ws,
            None
        ));
        assert!(!compatible_for(
            &format!("{stack}12 11 8:2 / /srv/workspaces/a\\057.git rw - ext4 /dev/sdb rw\n"),
            ws,
            Some("a")
        ));
        assert!(!compatible_for(
            &format!("{stack}12 11 8:2 / /srv/workspaces/a\\999 rw - ext4 /dev/sdb rw\n"),
            ws,
            Some("a")
        ));
    }
    #[test]
    fn accepts_unrelated_namespace_mounts_with_active_srv_stack() {
        let mounts = concat!(
            "56 20 0:29 / /srv rw,relatime shared:17 - autofs systemd-1 rw\n",
            "37 56 8:1 / /srv rw,noatime shared:36 - ext4 /dev/sda1 rw\n",
            "103 37 0:44 / /srv/docker/rootfs/overlayfs/a rw - overlay overlay rw\n",
            "104 37 0:45 / /srv/docker/rootfs/overlayfs/b rw - overlay overlay rw\n",
            "252 37 0:46 / /srv/docker/rootfs/overlayfs/c rw - overlay overlay rw\n",
            "337 37 0:47 / /srv/docker/rootfs/overlayfs/d rw - overlay overlay rw\n",
            "476 27 0:4 net:[4026532528] /run/docker/netns/a rw shared:314 - nsfs nsfs rw\n",
        );
        assert!(compatible_for(
            mounts,
            Path::new("/srv/workspaces"),
            Some("JaviPiV2")
        ));
    }
    #[test]
    fn reads_only_direct_metadata_without_following_links() {
        use std::os::unix::fs::symlink;
        let root =
            std::env::temp_dir().join(format!("javipiv2-project-test-{}", std::process::id()));
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
        for name in ["my project", "proyecto+uno", "proyecto á"] {
            assert!(name_ok(name));
        }
        use std::os::unix::ffi::OsStringExt;
        assert!(project_name(std::ffi::OsString::from_vec(vec![0xff])).is_err());
        for branch in ["feature+test", "feature/nested", "á"] {
            assert!(valid_branch(branch));
        }
        for branch in ["a/../b", "a b", "a.lock", "a//b", "a@{b"] {
            assert!(!valid_branch(branch));
        }
    }
    #[test]
    fn packed_nested_ref_and_invalid_loose_ref() {
        let root =
            std::env::temp_dir().join(format!("javipiv2-packed-test-{}", std::process::id()));
        fs::create_dir_all(root.join("repo/.git/objects")).unwrap();
        fs::create_dir_all(root.join("repo/.git/refs/heads")).unwrap();
        fs::write(
            root.join("repo/.git/HEAD"),
            "ref: refs/heads/feature+test/nested\n",
        )
        .unwrap();
        let hash = "1234567890123456789012345678901234567890";
        fs::write(
            root.join("repo/.git/packed-refs"),
            format!("{hash} refs/heads/feature+test/nested\n"),
        )
        .unwrap();
        let parent = File::open(&root).unwrap();
        let packed = project(&parent, "repo");
        assert_eq!(packed.head.as_deref(), Some(hash));
        assert_eq!(packed.git, GitState::Compatible);
        fs::create_dir(root.join("repo/.git/refs/heads/feature+test")).unwrap();
        fs::write(
            root.join("repo/.git/refs/heads/feature+test/nested"),
            "invalid\n",
        )
        .unwrap();
        let invalid = project(&parent, "repo");
        assert_eq!(invalid.git, GitState::Unknown);
        assert_eq!(invalid.head, None);
        fs::remove_file(root.join("repo/.git/refs/heads/feature+test/nested")).unwrap();
        std::os::unix::fs::symlink(
            "missing",
            root.join("repo/.git/refs/heads/feature+test/nested"),
        )
        .unwrap();
        let link = project(&parent, "repo");
        assert_eq!(link.git, GitState::Unknown);
        assert_eq!(link.head, None);
        drop(parent);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_paths_and_names() {
        assert!(Projects::new("/srv/workspaces").is_ok());
        for path in ["/tmp", "/srv/../tmp", "/srv", "relative"] {
            assert!(Projects::new(path).is_err());
        }
        for name in ["", ".", "..", "a/b", "../a"] {
            assert!(!name_ok(name));
        }
    }
}
