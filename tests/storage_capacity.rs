use javipiv2::modules::storage::{capacity_from_blocks, observe_capacity, Capacity};

const ACTIVE: &str =
    "56 20 0:29 / /srv rw - autofs systemd-1 rw\n37 56 8:1 / /srv rw - ext4 /dev/sda1 rw\n";
const INACTIVE: &str = "56 20 0:29 / /srv rw - autofs systemd-1 rw\n";
const CAPACITY: Capacity = Capacity {
    total_bytes: 1000,
    used_bytes: 600,
    available_bytes: 300,
    reserved_bytes: 100,
};

#[test]
fn reports_standard_capacity_math_and_reserved_blocks() {
    assert_eq!(capacity_from_blocks(100, 40, 30, 10), Some(CAPACITY));
    assert_eq!(capacity_from_blocks(u64::MAX, 0, 0, 2), None);
    assert_eq!(capacity_from_blocks(1, 2, 1, 1), None);
    assert_eq!(capacity_from_blocks(2, 1, 2, 1), None);
    assert_eq!(capacity_from_blocks(100, 40, 30, 0), None);
}

#[test]
fn getter_runs_only_for_confirmed_active_ext4_and_errors_are_unknown() {
    let mut calls = 0;
    let (status, capacity) = observe_capacity(ACTIVE, || {
        calls += 1;
        Ok::<_, ()>(CAPACITY)
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(capacity, Some(CAPACITY));
    assert!(status.srv_ext4);

    let misparented = ACTIVE.replace("37 56", "37 20");
    let stacked = format!("{ACTIVE}38 37 8:2 / /srv rw - ext4 /dev/sdb rw\n");
    let duplicate_ext4 =
        "37 20 8:1 / /srv rw - ext4 /dev/sda1 rw\n38 20 8:2 / /srv rw - ext4 /dev/sdb rw\n";
    for input in [
        INACTIVE,
        "malformed",
        "",
        &misparented,
        &stacked,
        duplicate_ext4,
    ] {
        let mut called = false;
        let result = observe_capacity(input, || {
            called = true;
            Ok::<_, ()>(CAPACITY)
        });
        assert!(!called);
        if input != "malformed" && !input.is_empty() {
            let (status, capacity) = result.unwrap();
            assert!(!status.srv_ext4);
            assert_eq!(capacity, None);
        } else {
            assert!(result.is_err());
        }
    }
    let (status, capacity) =
        observe_capacity(ACTIVE, || Err::<Capacity, _>("capacity failure")).unwrap();
    assert_eq!(capacity, None);
    assert!(status.srv_ext4);
}
