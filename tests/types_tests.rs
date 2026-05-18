use dnf_repofile::types::*;

// ---- Identifiers ----

#[test]
fn test_repo_id_valid() {
    let id = RepoId::try_new("fedora-updates").unwrap();
    assert_eq!(id.as_ref(), "fedora-updates");
}

#[test]
fn test_repo_id_trims_whitespace() {
    let id = RepoId::try_new("  myrepo  ").unwrap();
    assert_eq!(id.as_ref(), "myrepo");
}

#[test]
fn test_repo_id_rejects_empty() {
    assert!(RepoId::try_new("").is_err());
    assert!(RepoId::try_new("   ").is_err());
}

#[test]
fn test_repo_id_rejects_special_chars() {
    assert!(RepoId::try_new("bad@id").is_err());
    assert!(RepoId::try_new("repo name").is_err());
    assert!(RepoId::try_new("repo#1").is_err());
}

#[test]
fn test_repo_name_valid() {
    let name = RepoName::try_new("Fedora Updates").unwrap();
    assert_eq!(name.as_ref(), "Fedora Updates");
}

#[test]
fn test_repo_name_rejects_empty() {
    assert!(RepoName::try_new("").is_err());
}

#[test]
fn test_username_trim() {
    let u = Username::new("  alice  ");
    assert_eq!(u.as_ref(), "alice");
}

// ---- Numerics ----

#[test]
fn test_priority_range() {
    assert!(Priority::try_new(1).is_ok());
    assert!(Priority::try_new(50).is_ok());
    assert!(Priority::try_new(99).is_ok());
    assert!(Priority::try_new(0).is_err());
    assert!(Priority::try_new(100).is_err());
}

#[test]
fn test_priority_default() {
    assert_eq!(*Priority::default(), 99);
}

#[test]
fn test_retries_accepts_zero() {
    let r = Retries::try_new(0).unwrap();
    assert_eq!(*r, 0);
}

#[test]
fn test_install_only_limit_rejects_one() {
    assert!(InstallOnlyLimit::try_new(0).is_ok());
    assert!(InstallOnlyLimit::try_new(2).is_ok());
    assert!(InstallOnlyLimit::try_new(1).is_err());
    assert!(InstallOnlyLimit::try_new(3).is_ok());
}

#[test]
fn test_max_parallel_downloads_max_20() {
    assert!(MaxParallelDownloads::try_new(20).is_ok());
    assert!(MaxParallelDownloads::try_new(21).is_err());
}

#[test]
fn test_delta_rpm_percentage_range() {
    assert!(DeltaRpmPercentage::try_new(0).is_ok());
    assert!(DeltaRpmPercentage::try_new(100).is_ok());
    assert!(DeltaRpmPercentage::try_new(101).is_err());
}

#[test]
fn test_debug_level_range() {
    assert!(DebugLevel::try_new(10).is_ok());
    assert!(DebugLevel::try_new(11).is_err());
}

#[test]
fn test_cost_non_negative() {
    assert!(Cost::try_new(0).is_ok());
    assert!(Cost::try_new(-1).is_err());
}

// ---- Composite & Enums ----

#[test]
fn test_dnf_bool_parse_true_variants() {
    for v in &["1", "yes", "true", "on", "Yes", "YES", "True", "TRUE", "On", "ON"] {
        assert_eq!(DnfBool::parse(v).unwrap(), DnfBool::True, "failed for '{v}'");
    }
}

#[test]
fn test_dnf_bool_parse_false_variants() {
    for v in &["0", "no", "false", "off", "No", "NO", "False", "FALSE", "Off", "OFF"] {
        assert_eq!(DnfBool::parse(v).unwrap(), DnfBool::False, "failed for '{v}'");
    }
}

#[test]
fn test_dnf_bool_parse_invalid() {
    assert!(DnfBool::parse("maybe").is_err());
    assert!(DnfBool::parse("").is_err());
    assert!(DnfBool::parse("2").is_err());
}

#[test]
fn test_dnf_bool_display() {
    assert_eq!(DnfBool::True.to_string(), "1");
    assert_eq!(DnfBool::False.to_string(), "0");
}

#[test]
fn test_dnf_bool_from_bool() {
    assert_eq!(DnfBool::from(true), DnfBool::True);
    assert_eq!(DnfBool::from(false), DnfBool::False);
}

#[test]
fn test_metadata_expire_never() {
    assert_eq!(MetadataExpire::Never, MetadataExpire::Never);
}

#[test]
fn test_storage_size() {
    let s = StorageSize(1024);
    assert_eq!(s.0, 1024);
}

#[test]
fn test_proxy_setting_unset() {
    assert!(matches!(ProxySetting::Unset, ProxySetting::Unset));
}

#[test]
fn test_proxy_setting_disabled() {
    assert!(matches!(ProxySetting::Disabled, ProxySetting::Disabled));
}

#[test]
fn test_throttle_percent() {
    if let Throttle::Percent(50) = Throttle::Percent(50) {
        // ok
    } else {
        panic!("expected Percent(50)");
    }
}
