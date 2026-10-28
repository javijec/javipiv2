use javipiv2::modules::storage::inspect;

const STACK: &str = "56 20 0:29 / /srv rw - autofs systemd-1 rw\n37 56 8:1 / /srv rw - ext4 /dev/sda1 rw\n103 37 0:44 / /srv/docker/rootfs/overlayfs/a rw - overlay overlay rw\n104 37 8:2 / /srv/workspaces rw - ext4 /dev/sdb rw\n";

#[test]
fn observes_srv_stack_and_separates_docker_overlay() {
    let status = inspect(STACK);
    assert!(status.autofs);
    assert!(status.srv_ext4);
    assert_eq!(
        status.docker_overlays,
        vec!["/srv/docker/rootfs/overlayfs/a"]
    );
}

#[test]
fn autofs_without_active_data_mount_remains_independently_visible() {
    let status = inspect("56 20 0:29 / /srv rw - autofs systemd-1 rw\n");
    assert!(status.autofs);
    assert!(!status.srv_ext4);
}

#[test]
fn ambiguous_or_misparented_stack_is_unknown() {
    assert!(!inspect(&STACK.replace("37 56", "37 20")).srv_ext4);
    assert!(!inspect(&STACK.replace("8:1 / /srv", "8:1 /sub /srv")).srv_ext4);
    assert!(!inspect(&format!("{STACK}{STACK}")).srv_ext4);
    assert!(!inspect("malformed").autofs);
}

#[test]
fn decodes_paths_and_ignores_unrelated_mounts() {
    let raw = concat!(
        "56 20 0:29 / /srv rw - autofs systemd-1 rw\n",
        "37 56 8:1 / /srv rw - ext4 /dev/sda1 rw\n",
        "103 37 0:44 / /srv/docker/rootfs/overlayfs/a\\040b rw - overlay overlay rw\n",
        "104 37 8:2 / /srv/work\\040spaces rw - tmpfs tmpfs rw\n",
        "105 1 0:5 / /unrelated\\040mount rw - tmpfs tmpfs rw\n",
    );
    let status = inspect(raw);
    assert!(status.srv_ext4);
    assert_eq!(
        status.docker_overlays,
        vec!["/srv/docker/rootfs/overlayfs/a b"]
    );
    assert!(inspect(&raw.replace("/unrelated\\\\040mount", "/unrelated\\\\999mount")).srv_ext4);
    assert!(!inspect(&raw.replace("/srv rw - autofs", "/s\\\\999rv rw - autofs")).srv_ext4);
}

#[test]
fn cli_command_remains_read_only_and_help_lists_storage() {
    use std::ffi::OsString;
    let call = |args: &[&str]| javipiv2::interfaces::cli::run(args.iter().map(OsString::from));
    assert!(call(&["help"]).is_ok());
    assert!(call(&["storage", "status", "--fixture"]).is_err());
}
