use javipiv2::modules::storage::{inspect, inspect_checked};

const MOUNTINFO: &str = "56 20 0:29 / /srv rw - autofs systemd-1 rw\n37 56 8:1 / /srv rw - ext4 /dev/sda1 rw\n103 37 0:44 / /srv/docker/rootfs/overlayfs/a rw - overlay overlay rw\n";

#[test]
fn storage_tui_source_reports_confirmed_mounts_and_overlay_count() {
    let status = inspect_checked(MOUNTINFO).expect("synthetic mountinfo is valid");
    assert!(status.autofs);
    assert!(status.srv_ext4);
    assert_eq!(status.docker_overlays.len(), 1);
}

#[test]
fn malformed_docker_overlay_root_is_not_reported_as_zero() {
    let raw = "103 37 0:44 /bad\\999 /var/lib/docker/overlay2/a/merged rw - overlay overlay rw\n";
    assert!(inspect_checked(raw).is_err());
    assert_eq!(inspect(raw).docker_overlays.len(), 0);
}

#[test]
fn invalid_evidence_fails_explicitly_while_legacy_inspection_stays_unknown() {
    assert!(inspect_checked("not mountinfo").is_err());
    assert!(inspect_checked("").is_err());
    assert_eq!(inspect("not mountinfo"), Default::default());
}
