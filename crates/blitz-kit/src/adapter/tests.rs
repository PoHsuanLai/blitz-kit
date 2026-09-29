use super::*;

#[test]
fn the_environment_overrides_only_when_it_names_something() {
    let cases: &[(&str, AdapterPref, Option<&str>, AdapterPref)] = &[
        ("unset", AdapterPref::Discrete, None, AdapterPref::Discrete),
        (
            "empty",
            AdapterPref::Integrated,
            Some(""),
            AdapterPref::Integrated,
        ),
        ("blank", AdapterPref::Auto, Some("  "), AdapterPref::Auto),
        (
            "named",
            AdapterPref::Discrete,
            Some("NVIDIA"),
            AdapterPref::Named("NVIDIA".into()),
        ),
        (
            "trimmed",
            AdapterPref::Auto,
            Some(" AMD "),
            AdapterPref::Named("AMD".into()),
        ),
    ];
    for (name, configured, env, want) in cases {
        assert_eq!(configured.clone().with_env(*env), *want, "{name}");
    }
}
