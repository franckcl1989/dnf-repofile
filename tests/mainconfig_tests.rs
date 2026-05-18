use dnf_repofile::mainconfig::MainConfig;
use dnf_repofile::types::*;

#[test]
fn test_mainconfig_defaults() {
    let mc = MainConfig::default();
    assert!(mc.arch.is_none());
    assert!(mc.best.is_none());
    assert!(mc.installonly_limit.is_none());
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_mainconfig_set_debuglevel() {
    let mut mc = MainConfig {
        debuglevel: Some(DebugLevel::try_new(5).unwrap()),
        ..Default::default()
    };
    mc.debuglevel = Some(DebugLevel::try_new(5).unwrap());
    assert_eq!(*mc.debuglevel.unwrap(), 5);
}

#[test]
fn test_mainconfig_extras() {
    let mut mc = MainConfig::default();
    mc.extras.insert("custom".into(), vec!["value1".into()]);
    assert_eq!(mc.extras.get("custom").unwrap()[0], "value1");
}
