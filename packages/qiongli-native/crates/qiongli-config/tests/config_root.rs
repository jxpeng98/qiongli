use std::ffi::OsStr;
use std::path::Path;

#[cfg(any(unix, windows))]
use qiongli_config::ConfigError;
#[cfg(unix)]
use qiongli_config::ConfigRootSource;
use qiongli_config::resolve_config_root;

#[test]
#[cfg(unix)]
fn default_root_appends_the_v2_namespace() {
    let root = resolve_config_root(None, Path::new("/users/researcher")).unwrap();
    assert_eq!(root.source(), ConfigRootSource::Default);
    assert_eq!(
        root.compatibility_root(),
        Path::new("/users/researcher/.config/qiongli")
    );
    assert_eq!(
        root.state_root(),
        Path::new("/users/researcher/.config/qiongli/v2")
    );
    assert_eq!(root.symbolic_state_root(), "<user-home>/.config/qiongli/v2");
}

#[test]
#[cfg(unix)]
fn absolute_and_home_relative_overrides_append_exactly_one_namespace() {
    let absolute =
        resolve_config_root(Some(OsStr::new("/srv/qiongli")), Path::new("/home/u")).unwrap();
    assert_eq!(absolute.state_root(), Path::new("/srv/qiongli/v2"));

    let home = resolve_config_root(Some(OsStr::new("~/state")), Path::new("/home/u")).unwrap();
    assert_eq!(home.state_root(), Path::new("/home/u/state/v2"));

    let already_named =
        resolve_config_root(Some(OsStr::new("/srv/v2")), Path::new("/home/u")).unwrap();
    assert_eq!(already_named.state_root(), Path::new("/srv/v2/v2"));
}

#[test]
#[cfg(unix)]
fn unsafe_or_ambiguous_roots_fail_closed() {
    for value in ["", "relative", "~/../escape", "/tmp/../escape"] {
        assert_eq!(
            resolve_config_root(Some(OsStr::new(value)), Path::new("/home/u")),
            Err(ConfigError::InvalidConfigHome),
        );
    }
    assert_eq!(
        resolve_config_root(None, Path::new("relative-home")),
        Err(ConfigError::HomeUnavailable),
    );
}

#[test]
#[cfg(unix)]
fn debug_output_never_contains_the_concrete_path() {
    let root = resolve_config_root(
        Some(OsStr::new("/private/canary-user/qiongli")),
        Path::new("/home/u"),
    )
    .unwrap();
    let debug = format!("{root:?}");
    assert!(!debug.contains("canary-user"));
    assert!(debug.contains("<configured-root>/v2"));
}

#[test]
#[cfg(unix)]
fn absolute_non_utf8_override_does_not_require_lossy_conversion() {
    use std::os::unix::ffi::OsStringExt;

    let configured = std::ffi::OsString::from_vec(b"/tmp/qiongli-\xff".to_vec());
    let root = resolve_config_root(Some(&configured), Path::new("/home/u")).unwrap();
    assert_eq!(
        root.compatibility_root().as_os_str(),
        configured.as_os_str()
    );
}

#[test]
#[cfg(windows)]
fn windows_home_relative_separator_is_supported() {
    let root = resolve_config_root(
        Some(OsStr::new(r"~\state")),
        Path::new(r"C:\Users\researcher"),
    )
    .unwrap();
    assert_eq!(
        root.state_root(),
        Path::new(r"C:\Users\researcher\state\v2")
    );
}

#[test]
#[cfg(windows)]
fn windows_device_namespaces_are_rejected() {
    assert_eq!(
        resolve_config_root(None, Path::new(r"\\.\pipe\qiongli")),
        Err(ConfigError::HomeUnavailable)
    );
    for configured in [
        r"\\.\pipe\qiongli",
        r"\\?\GLOBALROOT\Device\HarddiskVolume1",
    ] {
        assert_eq!(
            resolve_config_root(
                Some(OsStr::new(configured)),
                Path::new(r"C:\Users\researcher")
            ),
            Err(ConfigError::InvalidConfigHome)
        );
    }
}
