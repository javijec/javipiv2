use javipiv2::core::catalog::{all_modules, parse_catalog, profile_modules};

#[test]
fn validates_entire_declarative_graph() {
    let valid = "core\tbase\t-\tpreview\tCore\na\tbase\tcore\tplanned\tA\nb\tbase\ta\tplanned\tB\n";
    let modules = parse_catalog(valid).unwrap();
    assert_eq!(
        modules.iter().map(|m| m.id).collect::<Vec<_>>(),
        ["core", "a", "b"]
    );
    for source in [
        "core\tbase\t-\tpreview\tCore\na\tbase\tb\tplanned\tA\nb\tbase\ta\tplanned\tB\n",
        "core\tbase\t-\tpreview\tCore\na\tbase\tb\tplanned\tA\nb\tbase\tc\tplanned\tB\nc\tbase\ta\tplanned\tC\n",
    ] {
        assert!(parse_catalog(source).is_err());
    }
}

#[test]
fn existing_profiles_still_resolve() {
    assert_eq!(all_modules().unwrap().len(), 21);
    for profile in ["minimal", "developer", "full"] {
        assert_eq!(profile_modules(profile).unwrap()[0].id, "core");
    }
    assert_eq!(profile_modules("full").unwrap().len(), 21);
}
