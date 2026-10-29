use javipiv2::modules::updates::{
    check_with, compare, json_version, parse_version, transport_body, Comparison,
};

#[test]
fn updates_versions_are_ordered_numerically() {
    assert_eq!(compare(Some("1.2.3"), Some("1.2.3")), Comparison::Current);
    assert_eq!(
        compare(Some("2.0.0"), Some("1.0.0")),
        Comparison::InstalledNewer
    );
    assert_eq!(
        compare(Some("1.9.0"), Some("1.10.0")),
        Comparison::Available
    );
    assert_eq!(compare(Some("01.2.3"), Some("1.2.3")), Comparison::Current);
    assert_eq!(compare(None, Some("1.0.0")), Comparison::Unknown);
    assert_eq!(
        compare(Some("1.0.0"), Some("1.0.0-beta")),
        Comparison::Unknown
    );
}

#[test]
fn updates_injected_sources_cover_all_endpoints_and_no_execution() {
    let mut paths = Vec::new();
    let mut urls = Vec::new();
    let results = check_with(
        |path| {
            paths.push(path.to_owned());
            Some(br#"{"version":"1.2.3"}"#.to_vec())
        },
        |url| {
            urls.push(url.to_owned());
            if url.contains("engram/") {
                None
            } else if url.contains("api.github.com") {
                Some(br#"{"tag_name":"v2.0.0"}"#.to_vec())
            } else {
                Some(br#"{"version":"2.0.0"}"#.to_vec())
            }
        },
    );
    assert_eq!(
        results.iter().map(|r| r.name).collect::<Vec<_>>(),
        [
            "Codex",
            "Pi",
            "Gentle Shell",
            "Paseo",
            "Gentle AI",
            "Engram"
        ]
    );
    assert_eq!(
        urls,
        [
            "https://registry.npmjs.org/@openai%2fcodex/latest",
            "https://registry.npmjs.org/@earendil-works%2fpi-coding-agent/latest",
            "https://registry.npmjs.org/gentle-pi/latest",
            "https://registry.npmjs.org/@getpaseo%2fcli/latest",
            "https://api.github.com/repos/Gentleman-Programming/gentle-ai/releases/latest",
            "https://api.github.com/repos/Gentleman-Programming/engram/releases/latest"
        ]
    );
    assert_eq!(
        paths,
        [
            "/usr/local/lib/node_modules/@openai/codex/package.json",
            "/usr/local/lib/node_modules/@earendil-works/pi-coding-agent/package.json",
            "/usr/local/lib/node_modules/gentle-pi/package.json",
            "/usr/local/lib/node_modules/@getpaseo/cli/package.json"
        ]
    );
    assert!(results[..4]
        .iter()
        .all(|r| r.comparison == Comparison::Available));
    assert_eq!(results[4].comparison, Comparison::Unknown);
    assert_eq!(results[4].upstream.as_deref(), Some("2.0.0"));
    assert_eq!(results[5].comparison, Comparison::Unknown);
    assert_eq!(results[5].upstream, None);
    assert_eq!(
        results[0].line(),
        "Codex: instalada=1.2.3, upstream=2.0.0, estado=actualización disponible"
    );
}

#[test]
fn updates_local_fallback_and_failures_are_unknown() {
    let mut paths = Vec::new();
    let results = check_with(
        |path| {
            paths.push(path.to_owned());
            if path == "/usr/lib/node_modules/gentle-pi/package.json" {
                Some(br#"{"version":"3.0.0"}"#.to_vec())
            } else {
                None
            }
        },
        |_| None,
    );
    assert_eq!(paths.len(), 8);
    assert_eq!(results[2].installed.as_deref(), Some("3.0.0"));
    assert!(results
        .iter()
        .all(|r| r.comparison == Comparison::Unknown && r.upstream.is_none()));
}

#[test]
fn updates_transport_rejects_errors_redirects_and_oversized_bodies() {
    assert_eq!(transport_body(true, b"{}\n200"), Some(&b"{}"[..]));
    assert_eq!(transport_body(false, b"{}\n200"), None);
    assert_eq!(transport_body(true, b"{}\n302"), None);
    assert_eq!(transport_body(true, &vec![b'x'; 65541]), None);
}

#[test]
fn updates_version_parser_rejects_untrusted_content_and_non_utf8() {
    assert_eq!(parse_version(b"v1.2.3"), Some("1.2.3".into()));
    for input in [
        &b"1.2.3 SECRET"[..],
        &b"1.2.3\nsecret"[..],
        &b"\xff"[..],
        &b"1.2"[..],
        &b"1.2.3-beta"[..],
    ] {
        assert_eq!(parse_version(input), None);
    }
}

#[test]
fn updates_json_requires_valid_bounded_public_version() {
    assert_eq!(
        json_version(br#"{"version":"1.2.3"}"#, "version"),
        Some("1.2.3".into())
    );
    assert_eq!(
        json_version(br#"{"tag_name":"v2.3.4"}"#, "tag_name"),
        Some("2.3.4".into())
    );
    assert_eq!(json_version(br#"{"version":"SECRET"}"#, "version"), None);
    assert_eq!(json_version(b"\xff", "version"), None);
    assert_eq!(json_version(&vec![b' '; 65537], "version"), None);
}

#[test]
fn updates_help_and_rejected_arguments_are_safe() {
    let binary = env!("CARGO_BIN_EXE_javipiv2");
    let help = std::process::Command::new(binary)
        .arg("help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8(help.stdout)
        .unwrap()
        .contains("updates check"));
    let rejected = std::process::Command::new(binary)
        .args(["updates", "check", "--install"])
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
    assert_eq!(
        rejected.stderr,
        b"Error: comando desconocido; us\xc3\xa1 javipiv2 help\n"
    );
}
