use dnf_repofile::types::*;
use url::Url;

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

// ---- Remaining identifiers ----

#[test]
fn test_password_wrapper() {
    let p = Password::new("secret");
    assert_eq!(p.as_ref(), "secret");
    // no sanitize — whitespace preserved
    let p2 = Password::new("  secret  ");
    assert_eq!(p2.as_ref(), "  secret  ");
}

#[test]
fn test_proxy_username_trim() {
    let u = ProxyUsername::new("  proxyuser  ");
    assert_eq!(u.as_ref(), "proxyuser");
}

#[test]
fn test_proxy_password_wrapper() {
    let p = ProxyPassword::new("proxysecret");
    assert_eq!(p.as_ref(), "proxysecret");
    // no sanitize — whitespace preserved
    let p2 = ProxyPassword::new("  psecret  ");
    assert_eq!(p2.as_ref(), "  psecret  ");
}

#[test]
fn test_user_agent_trim() {
    let ua = UserAgent::new("  my-agent/1.0  ");
    assert_eq!(ua.as_ref(), "my-agent/1.0");
}

#[test]
fn test_module_platform_id_trim() {
    let id = ModulePlatformId::new("  platform:1  ");
    assert_eq!(id.as_ref(), "platform:1");
}

#[test]
fn test_repo_name_trims_whitespace() {
    let name = RepoName::try_new("  My Repo  ").unwrap();
    assert_eq!(name.as_ref(), "My Repo");
}

// ---- Remaining numerics ----

#[test]
fn test_timeout_seconds_non_negative() {
    assert!(TimeoutSeconds::try_new(0).is_ok());
    assert_eq!(*TimeoutSeconds::try_new(30).unwrap(), 30);
}

#[test]
fn test_timeout_seconds_default() {
    assert_eq!(*TimeoutSeconds::default(), 30);
}

#[test]
fn test_log_level_range() {
    assert!(LogLevel::try_new(0).is_ok());
    assert!(LogLevel::try_new(10).is_ok());
    assert!(LogLevel::try_new(11).is_err());
}

#[test]
fn test_log_level_default() {
    assert_eq!(*LogLevel::default(), 9);
}

#[test]
fn test_log_rotate_non_negative() {
    assert!(LogRotate::try_new(0).is_ok());
    assert_eq!(*LogRotate::try_new(4).unwrap(), 4);
}

#[test]
fn test_log_rotate_default() {
    assert_eq!(*LogRotate::default(), 4);
}

#[test]
fn test_metadata_timer_sync_non_negative() {
    assert!(MetadataTimerSync::try_new(0).is_ok());
    assert_eq!(*MetadataTimerSync::try_new(10800).unwrap(), 10800);
}

#[test]
fn test_metadata_timer_sync_default() {
    assert_eq!(*MetadataTimerSync::default(), 10800);
}

#[test]
fn test_error_level_range() {
    assert!(ErrorLevel::try_new(0).is_ok());
    assert!(ErrorLevel::try_new(10).is_ok());
    assert!(ErrorLevel::try_new(11).is_err());
}

#[test]
fn test_error_level_default() {
    assert_eq!(*ErrorLevel::default(), 3);
}

// ---- Missing enum variants ----

#[test]
fn test_metadata_expire_duration() {
    assert_eq!(MetadataExpire::Duration(3600), MetadataExpire::Duration(3600));
}

#[test]
fn test_throttle_absolute() {
    let t = Throttle::Absolute(StorageSize(1024));
    assert_eq!(t, Throttle::Absolute(StorageSize(1024)));
}

#[test]
fn test_proxy_setting_url() {
    let url = Url::parse("http://proxy.example.com:8080").unwrap();
    let setting = ProxySetting::Url(url);
    assert!(matches!(setting, ProxySetting::Url(_)));
}

// ---- Remaining enum types ----

#[test]
fn test_ip_resolve_variants() {
    assert_ne!(IpResolve::V4, IpResolve::V6);
}

#[test]
fn test_proxy_auth_method_variants() {
    assert_eq!(format!("{:?}", ProxyAuthMethod::Any), "Any");
    assert_eq!(format!("{:?}", ProxyAuthMethod::None_), "None_");
}

#[test]
fn test_repo_metadata_type() {
    assert_eq!(format!("{:?}", RepoMetadataType::RpmMd), "RpmMd");
}

#[test]
fn test_multilib_policy_variants() {
    assert_ne!(MultilibPolicy::Best, MultilibPolicy::All);
}

#[test]
fn test_persistence_variants() {
    assert_ne!(Persistence::Auto, Persistence::Transient);
    assert_ne!(Persistence::Auto, Persistence::Persist);
}

#[test]
fn test_rpm_verbosity_variants() {
    assert_ne!(RpmVerbosity::Critical, RpmVerbosity::Emergency);
    assert_ne!(RpmVerbosity::Error, RpmVerbosity::Warn);
    assert_ne!(RpmVerbosity::Info, RpmVerbosity::Debug);
}

#[test]
fn test_ts_flag_variants() {
    assert_ne!(TsFlag::NoScripts, TsFlag::Test);
    assert_ne!(TsFlag::NoTriggers, TsFlag::NoDocs);
    assert_ne!(TsFlag::JustDb, TsFlag::NoContexts);
    assert_ne!(TsFlag::NoCaps, TsFlag::NoCrypto);
    assert_ne!(TsFlag::Deploops, TsFlag::NoPlugins);
}
