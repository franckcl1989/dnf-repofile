use crate::error::ParseError;
use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::types::*;
use camino::Utf8PathBuf;
use indexmap::IndexMap;
use std::str::FromStr;
use url::Url;

// ============================================================================
// Helper macro for nutype numeric types that lack FromStr
// ============================================================================

macro_rules! try_parse_nutype {
    ($val:expr, $typ:ty, $inner:ty) => {
        $val.trim()
            .parse::<$inner>()
            .ok()
            .and_then(|n| <$typ>::try_new(n).ok())
    };
}

// ============================================================================
// Known option keys
// ============================================================================

#[allow(dead_code)]
const KNOWN_REPO_KEYS: &[&str] = &[
    "name", "baseurl", "mirrorlist", "metalink", "gpgkey", "enabled",
    "priority", "cost", "module_hotfixes", "type", "mediaid", "enabled_metadata",
    "excludepkgs", "includepkgs",
    "gpgcheck", "repo_gpgcheck", "localpkg_gpgcheck", "skip_if_unavailable",
    "deltarpm", "deltarpm_percentage", "enablegroups", "fastestmirror", "countme",
    "bandwidth", "throttle", "minrate", "retries", "timeout",
    "max_parallel_downloads", "metadata_expire", "ip_resolve",
    "sslverify", "sslverifystatus", "sslcacert", "sslclientcert", "sslclientkey",
    "proxy", "proxy_username", "proxy_password", "proxy_auth_method",
    "proxy_sslverify", "proxy_sslcacert", "proxy_sslclientcert", "proxy_sslclientkey",
    "username", "password", "user_agent",
];

#[allow(dead_code)]
const KNOWN_MAIN_KEYS: &[&str] = &[
    "arch", "basearch", "releasever", "cachedir", "persistdir", "logdir",
    "config_file_path", "installroot", "reposdir", "varsdir", "pluginconfpath",
    "pluginpath", "debuglevel", "logfilelevel", "log_rotate", "log_size",
    "installonly_limit", "errorlevel", "metadata_timer_sync",
    "allow_vendor_change", "assumeno", "assumeyes", "autocheck_running_kernel",
    "best", "cacheonly", "check_config_file_age", "clean_requirements_on_remove",
    "debug_solver", "defaultyes", "diskspacecheck", "exclude_from_weak_autodetect",
    "exit_on_lock", "gpgkey_dns_verification", "ignorearch", "install_weak_deps",
    "keepcache", "log_compress", "module_obsoletes", "module_stream_switch",
    "obsoletes", "plugins", "protect_running_kernel", "strict",
    "upgrade_group_objects_upgrade", "zchunk",
    "installonlypkgs", "protected_packages", "exclude_from_weak",
    "group_package_types", "optional_metadata_types", "tsflags",
    "usr_drift_protected_paths",
    "multilib_policy", "persistence", "rpmverbosity", "module_platform_id",
];

// ============================================================================
// Core public types
// ============================================================================

/// A section block: typed data + formatting metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionBlock<T> {
    pub header_comments: Vec<String>,
    pub data: T,
    pub item_comments: IndexMap<String, String>,
    pub item_order: Vec<String>,
    pub raw_entries: Vec<RawEntry>,
}

/// An unrecognized key-value entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    pub key: String,
    pub value: String,
    pub inline_comment: Option<String>,
    pub leading_comments: Vec<String>,
}

/// A complete parsed .repo file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoFile {
    pub preamble: Vec<String>,
    pub main: Option<SectionBlock<MainConfig>>,
    pub repos: IndexMap<RepoId, SectionBlock<Repo>>,
}

// ============================================================================
// Internal parse types
// ============================================================================

#[derive(Debug)]
struct ParseState {
    preamble: Vec<String>,
    pending_comments: Vec<String>,
    current_section: Option<String>,
    current_entries: Vec<RawLine>,
    sections: IndexMap<String, Vec<RawLine>>,
    section_header_comments: IndexMap<String, Vec<String>>,
}

/// A raw INI entry being built up during parsing (private)
#[derive(Debug, Clone)]
struct RawLine {
    key: String,
    value: String,
    inline_comment: Option<String>,
    leading_comments: Vec<String>,
}

// ============================================================================
// Helper: split value and inline comment
// ============================================================================

fn split_value_and_comment(value_part: &str) -> (String, Option<String>) {
    let mut in_quotes = false;
    for (i, ch) in value_part.char_indices() {
        if ch == '"' {
            in_quotes = !in_quotes;
        }
        if ch == '#' && !in_quotes {
            return (
                value_part[..i].to_string(),
                Some(value_part[i + 1..].trim().to_string()),
            );
        }
    }
    (value_part.to_string(), None)
}

// ============================================================================
// Enum value parsers
// ============================================================================

fn parse_ip_resolve(val: &str) -> Option<IpResolve> {
    match val.trim().to_lowercase().as_str() {
        "4" | "ipv4" => Some(IpResolve::V4),
        "6" | "ipv6" => Some(IpResolve::V6),
        _ => None,
    }
}

fn parse_proxy_auth_method(val: &str) -> Option<ProxyAuthMethod> {
    match val.trim().to_lowercase().as_str() {
        "any" => Some(ProxyAuthMethod::Any),
        "none" => Some(ProxyAuthMethod::None_),
        "basic" => Some(ProxyAuthMethod::Basic),
        "digest" => Some(ProxyAuthMethod::Digest),
        "negotiate" => Some(ProxyAuthMethod::Negotiate),
        "ntlm" => Some(ProxyAuthMethod::Ntlm),
        "digest_ie" | "digestie" => Some(ProxyAuthMethod::DigestIe),
        "ntlm_wb" | "ntlmwb" => Some(ProxyAuthMethod::NtlmWb),
        _ => None,
    }
}

fn parse_proxy(val: &str) -> ProxySetting {
    let trimmed = val.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("_none_") {
        return ProxySetting::Disabled;
    }
    Url::from_str(trimmed).map(ProxySetting::Url).unwrap_or(ProxySetting::Unset)
}

fn parse_multilib_policy(val: &str) -> Option<MultilibPolicy> {
    match val.trim().to_lowercase().as_str() {
        "best" => Some(MultilibPolicy::Best),
        "all" => Some(MultilibPolicy::All),
        _ => None,
    }
}

fn parse_persistence(val: &str) -> Option<Persistence> {
    match val.trim().to_lowercase().as_str() {
        "auto" => Some(Persistence::Auto),
        "transient" => Some(Persistence::Transient),
        "persist" => Some(Persistence::Persist),
        _ => None,
    }
}

fn parse_rpmverbosity(val: &str) -> Option<RpmVerbosity> {
    match val.trim().to_lowercase().as_str() {
        "critical" => Some(RpmVerbosity::Critical),
        "emergency" => Some(RpmVerbosity::Emergency),
        "error" => Some(RpmVerbosity::Error),
        "warn" => Some(RpmVerbosity::Warn),
        "info" => Some(RpmVerbosity::Info),
        "debug" => Some(RpmVerbosity::Debug),
        _ => None,
    }
}

fn parse_tsflags(val: &str) -> Vec<TsFlag> {
    val.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            "notriggers" | "notrigger" => Some(TsFlag::NoTriggers),
            "noscripts" | "noscript" => Some(TsFlag::NoScripts),
            "test" => Some(TsFlag::Test),
            "nodocs" | "nodoc" => Some(TsFlag::NoDocs),
            "justdb" => Some(TsFlag::JustDb),
            "nocontexts" => Some(TsFlag::NoContexts),
            "nocaps" => Some(TsFlag::NoCaps),
            "nocrypto" => Some(TsFlag::NoCrypto),
            "deploops" => Some(TsFlag::Deploops),
            "noplugins" => Some(TsFlag::NoPlugins),
            _ => None,
        })
        .collect()
}

fn parse_storage_size(val: &str) -> Option<StorageSize> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (num_str, multiplier) = match trimmed.chars().last() {
        Some('k') | Some('K') => (&trimmed[..trimmed.len() - 1], 1024),
        Some('M') => (&trimmed[..trimmed.len() - 1], 1024 * 1024),
        Some('G') => (&trimmed[..trimmed.len() - 1], 1024 * 1024 * 1024),
        _ => (trimmed, 1),
    };
    let num: u64 = num_str.trim().parse().ok()?;
    Some(StorageSize(num * multiplier))
}

fn parse_metadata_expire(val: &str) -> Option<MetadataExpire> {
    let trimmed = val.trim();
    if trimmed.eq_ignore_ascii_case("never") {
        return Some(MetadataExpire::Never);
    }
    if let Ok(secs) = trimmed.parse::<u64>() {
        return Some(MetadataExpire::Duration(secs));
    }
    None
}

fn parse_throttle(val: &str) -> Option<Throttle> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Try percentage format (nn%)
    if let Some(pct_str) = trimmed.strip_suffix('%') {
        if let Ok(pct) = pct_str.trim().parse::<u8>() {
            if pct <= 100 {
                return Some(Throttle::Percent(pct));
            }
        }
    }
    // Try absolute storage size
    parse_storage_size(trimmed).map(Throttle::Absolute)
}

fn parse_repo_type(val: &str) -> Option<RepoMetadataType> {
    match val.trim().to_lowercase().as_str() {
        "rpm-md" | "rpm" => Some(RepoMetadataType::RpmMd),
        _ => None,
    }
}

// ============================================================================
// Section entry parsers
// ============================================================================

/// Parse raw entries into a `Repo` struct, extracting typed values for known
/// keys and stashing unknown keys into `repo.extras`.
fn parse_entries_into_repo(
    repo: &mut Repo,
    entries: &[RawLine],
) -> (Vec<String>, IndexMap<String, String>, Vec<RawEntry>) {
    let mut item_order = Vec::new();
    let mut item_comments = IndexMap::new();
    let mut raw_entries = Vec::new();

    for entry in entries {
        let key = entry.key.clone();
        let value = entry.value.clone();

        // Always track raw entry for round-trip fidelity
        raw_entries.push(RawEntry {
            key: key.clone(),
            value: value.clone(),
            inline_comment: entry.inline_comment.clone(),
            leading_comments: entry.leading_comments.clone(),
        });

        item_order.push(key.clone());
        if let Some(ref ic) = entry.inline_comment {
            item_comments.insert(key.clone(), ic.clone());
        }

        match key.as_str() {
            // ---- Identifiers ----
            "name" => {
                if let Ok(v) = RepoName::from_str(&value) {
                    repo.name = Some(v);
                }
            }
            "mediaid" => {
                repo.mediaid = Some(value.clone());
            }

            // ---- URL sources ----
            "baseurl" => {
                if let Ok(v) = Url::from_str(&value) {
                    repo.baseurl.push(v);
                }
            }
            "mirrorlist" => {
                if let Ok(v) = Url::from_str(&value) {
                    repo.mirrorlist = Some(v);
                }
            }
            "metalink" => {
                if let Ok(v) = Url::from_str(&value) {
                    repo.metalink = Some(v);
                }
            }

            // ---- String lists ----
            "gpgkey" => {
                repo.gpgkey.push(value.clone());
            }
            "enabled_metadata" => {
                repo.enabled_metadata.push(value.clone());
            }
            "excludepkgs" => {
                repo.excludepkgs.push(value.clone());
            }
            "includepkgs" => {
                repo.includepkgs.push(value.clone());
            }

            // ---- DNF booleans ----
            "enabled" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.enabled = Some(v);
                }
            }
            "module_hotfixes" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.module_hotfixes = Some(v);
                }
            }
            "gpgcheck" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.gpgcheck = Some(v);
                }
            }
            "repo_gpgcheck" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.repo_gpgcheck = Some(v);
                }
            }
            "localpkg_gpgcheck" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.localpkg_gpgcheck = Some(v);
                }
            }
            "skip_if_unavailable" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.skip_if_unavailable = Some(v);
                }
            }
            "deltarpm" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.deltarpm = Some(v);
                }
            }
            "enablegroups" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.enablegroups = Some(v);
                }
            }
            "fastestmirror" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.fastestmirror = Some(v);
                }
            }
            "countme" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.countme = Some(v);
                }
            }
            "sslverify" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.sslverify = Some(v);
                }
            }
            "sslverifystatus" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.sslverifystatus = Some(v);
                }
            }
            "proxy_sslverify" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    repo.proxy_sslverify = Some(v);
                }
            }

            // ---- Numerics ----
            "priority" => {
                if let Some(v) = try_parse_nutype!(&value, Priority, i32) {
                    repo.priority = Some(v);
                }
            }
            "cost" => {
                if let Some(v) = try_parse_nutype!(&value, Cost, i32) {
                    repo.cost = Some(v);
                }
            }
            "deltarpm_percentage" => {
                if let Some(v) = try_parse_nutype!(&value, DeltaRpmPercentage, u32) {
                    repo.deltarpm_percentage = Some(v);
                }
            }
            "retries" => {
                if let Some(v) = try_parse_nutype!(&value, Retries, u32) {
                    repo.retries = Some(v);
                }
            }
            "timeout" => {
                if let Some(v) = try_parse_nutype!(&value, TimeoutSeconds, u32) {
                    repo.timeout = Some(v);
                }
            }
            "max_parallel_downloads" => {
                if let Some(v) = try_parse_nutype!(&value, MaxParallelDownloads, u32) {
                    repo.max_parallel_downloads = Some(v);
                }
            }

            // ---- Storage sizes ----
            "bandwidth" => {
                if let Some(v) = parse_storage_size(&value) {
                    repo.bandwidth = Some(v);
                }
            }
            "minrate" => {
                if let Some(v) = parse_storage_size(&value) {
                    repo.minrate = Some(v);
                }
            }

            // ---- Throttle ----
            "throttle" => {
                if let Some(v) = parse_throttle(&value) {
                    repo.throttle = Some(v);
                }
            }

            // ---- Metadata expire ----
            "metadata_expire" => {
                if let Some(v) = parse_metadata_expire(&value) {
                    repo.metadata_expire = Some(v);
                }
            }

            // ---- IP resolve ----
            "ip_resolve" => {
                if let Some(v) = parse_ip_resolve(&value) {
                    repo.ip_resolve = Some(v);
                }
            }

            // ---- SSL path fields ----
            "sslcacert" => {
                repo.sslcacert = Some(Utf8PathBuf::from(&value));
            }
            "sslclientcert" => {
                repo.sslclientcert = Some(Utf8PathBuf::from(&value));
            }
            "sslclientkey" => {
                repo.sslclientkey = Some(Utf8PathBuf::from(&value));
            }

            // ---- Proxy ----
            "proxy" => {
                repo.proxy = parse_proxy(&value);
            }
            "proxy_username" => {
                repo.proxy_username = ProxyUsername::from_str(&value).ok();
            }
            "proxy_password" => {
                repo.proxy_password = ProxyPassword::from_str(&value).ok();
            }
            "proxy_auth_method" => {
                if let Some(v) = parse_proxy_auth_method(&value) {
                    repo.proxy_auth_method = Some(v);
                }
            }
            "proxy_sslcacert" => {
                repo.proxy_sslcacert = Some(Utf8PathBuf::from(&value));
            }
            "proxy_sslclientcert" => {
                repo.proxy_sslclientcert = Some(Utf8PathBuf::from(&value));
            }
            "proxy_sslclientkey" => {
                repo.proxy_sslclientkey = Some(Utf8PathBuf::from(&value));
            }

            // ---- Authentication ----
            "username" => {
                repo.username = Username::from_str(&value).ok();
            }
            "password" => {
                repo.password = Password::from_str(&value).ok();
            }
            "user_agent" => {
                repo.user_agent = UserAgent::from_str(&value).ok();
            }

            // ---- Type ----
            "type" => {
                if let Some(v) = parse_repo_type(&value) {
                    repo.metadata_type = Some(v);
                }
            }

            // ---- Unknown -> extras ----
            _ => {
                repo.extras
                    .entry(key.clone())
                    .or_default()
                    .push(value.clone());
            }
        }
    }

    (item_order, item_comments, raw_entries)
}

/// Parse raw entries into a `MainConfig` struct.
fn parse_entries_into_mainconfig(
    config: &mut MainConfig,
    entries: &[RawLine],
) -> (Vec<String>, IndexMap<String, String>, Vec<RawEntry>) {
    let mut item_order = Vec::new();
    let mut item_comments = IndexMap::new();
    let mut raw_entries = Vec::new();

    for entry in entries {
        let key = entry.key.clone();
        let value = entry.value.clone();

        raw_entries.push(RawEntry {
            key: key.clone(),
            value: value.clone(),
            inline_comment: entry.inline_comment.clone(),
            leading_comments: entry.leading_comments.clone(),
        });

        item_order.push(key.clone());
        if let Some(ref ic) = entry.inline_comment {
            item_comments.insert(key.clone(), ic.clone());
        }

        match key.as_str() {
            // ---- String fields ----
            "arch" => config.arch = Some(value.clone()),
            "basearch" => config.basearch = Some(value.clone()),
            "releasever" => config.releasever = Some(value.clone()),

            // ---- Path fields ----
            "cachedir" => config.cachedir = Some(Utf8PathBuf::from(&value)),
            "persistdir" => config.persistdir = Some(Utf8PathBuf::from(&value)),
            "logdir" => config.logdir = Some(Utf8PathBuf::from(&value)),
            "config_file_path" => config.config_file_path = Some(Utf8PathBuf::from(&value)),
            "installroot" => config.installroot = Some(Utf8PathBuf::from(&value)),

            // ---- Path lists ----
            "reposdir" => config.reposdir.push(Utf8PathBuf::from(&value)),
            "varsdir" => config.varsdir.push(Utf8PathBuf::from(&value)),
            "pluginconfpath" => config.pluginconfpath.push(Utf8PathBuf::from(&value)),
            "pluginpath" => config.pluginpath.push(Utf8PathBuf::from(&value)),

            // ---- String lists ----
            "installonlypkgs" => config.installonlypkgs.push(value.clone()),
            "protected_packages" => config.protected_packages.push(value.clone()),
            "exclude_from_weak" => config.exclude_from_weak.push(value.clone()),
            "group_package_types" => config.group_package_types.push(value.clone()),
            "optional_metadata_types" => config.optional_metadata_types.push(value.clone()),
            "usr_drift_protected_paths" => config.usr_drift_protected_paths.push(value.clone()),

            // ---- DNF booleans ----
            "allow_vendor_change" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.allow_vendor_change = Some(v);
                }
            }
            "assumeno" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.assumeno = Some(v);
                }
            }
            "assumeyes" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.assumeyes = Some(v);
                }
            }
            "autocheck_running_kernel" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.autocheck_running_kernel = Some(v);
                }
            }
            "best" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.best = Some(v);
                }
            }
            "cacheonly" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.cacheonly = Some(v);
                }
            }
            "check_config_file_age" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.check_config_file_age = Some(v);
                }
            }
            "clean_requirements_on_remove" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.clean_requirements_on_remove = Some(v);
                }
            }
            "debug_solver" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.debug_solver = Some(v);
                }
            }
            "defaultyes" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.defaultyes = Some(v);
                }
            }
            "diskspacecheck" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.diskspacecheck = Some(v);
                }
            }
            "exclude_from_weak_autodetect" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.exclude_from_weak_autodetect = Some(v);
                }
            }
            "exit_on_lock" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.exit_on_lock = Some(v);
                }
            }
            "gpgkey_dns_verification" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.gpgkey_dns_verification = Some(v);
                }
            }
            "ignorearch" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.ignorearch = Some(v);
                }
            }
            "install_weak_deps" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.install_weak_deps = Some(v);
                }
            }
            "keepcache" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.keepcache = Some(v);
                }
            }
            "log_compress" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.log_compress = Some(v);
                }
            }
            "module_obsoletes" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.module_obsoletes = Some(v);
                }
            }
            "module_stream_switch" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.module_stream_switch = Some(v);
                }
            }
            "obsoletes" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.obsoletes = Some(v);
                }
            }
            "plugins" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.plugins = Some(v);
                }
            }
            "protect_running_kernel" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.protect_running_kernel = Some(v);
                }
            }
            "strict" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.strict = Some(v);
                }
            }
            "upgrade_group_objects_upgrade" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.upgrade_group_objects_upgrade = Some(v);
                }
            }
            "zchunk" => {
                if let Ok(v) = DnfBool::parse(&value) {
                    config.zchunk = Some(v);
                }
            }

            // ---- Numerics ----
            "debuglevel" => {
                if let Some(v) = try_parse_nutype!(&value, DebugLevel, u8) {
                    config.debuglevel = Some(v);
                }
            }
            "logfilelevel" => {
                if let Some(v) = try_parse_nutype!(&value, LogLevel, u8) {
                    config.logfilelevel = Some(v);
                }
            }
            "log_rotate" => {
                if let Some(v) = try_parse_nutype!(&value, LogRotate, u32) {
                    config.log_rotate = Some(v);
                }
            }
            "installonly_limit" => {
                if let Some(v) = try_parse_nutype!(&value, InstallOnlyLimit, u32) {
                    config.installonly_limit = Some(v);
                }
            }
            "errorlevel" => {
                if let Some(v) = try_parse_nutype!(&value, ErrorLevel, u8) {
                    config.errorlevel = Some(v);
                }
            }
            "metadata_timer_sync" => {
                if let Some(v) = try_parse_nutype!(&value, MetadataTimerSync, u32) {
                    config.metadata_timer_sync = Some(v);
                }
            }

            // ---- Storage size ----
            "log_size" => {
                if let Some(v) = parse_storage_size(&value) {
                    config.log_size = Some(v);
                }
            }

            // ---- Enums ----
            "multilib_policy" => {
                if let Some(v) = parse_multilib_policy(&value) {
                    config.multilib_policy = Some(v);
                }
            }
            "persistence" => {
                if let Some(v) = parse_persistence(&value) {
                    config.persistence = Some(v);
                }
            }
            "rpmverbosity" => {
                if let Some(v) = parse_rpmverbosity(&value) {
                    config.rpmverbosity = Some(v);
                }
            }

            // ---- TsFlags ----
            "tsflags" => {
                let flags = parse_tsflags(&value);
                config.tsflags.extend(flags);
            }

            // ---- Module platform id ----
            "module_platform_id" => {
                config.module_platform_id = ModulePlatformId::from_str(&value).ok();
            }

            // ---- Unknown -> extras ----
            _ => {
                config
                    .extras
                    .entry(key.clone())
                    .or_default()
                    .push(value.clone());
            }
        }
    }

    (item_order, item_comments, raw_entries)
}

// ============================================================================
// Build RepoFile from ParseState
// ============================================================================

fn build_repofile(state: ParseState) -> std::result::Result<RepoFile, ParseError> {
    let mut rf = RepoFile::new();
    rf.preamble = state.preamble;

    for (sec_name, entries) in &state.sections {
        let header_comments = state
            .section_header_comments
            .get(sec_name)
            .cloned()
            .unwrap_or_default();
        if sec_name == "main" {
            let mut mc = MainConfig::default();
            let (io, ic, re) = parse_entries_into_mainconfig(&mut mc, entries);
            rf.main = Some(SectionBlock {
                header_comments,
                data: mc,
                item_comments: ic,
                item_order: io,
                raw_entries: re,
            });
        } else {
            let repo_id = RepoId::try_new(sec_name.as_str()).map_err(|_| {
                ParseError::InvalidRepoId {
                    id: sec_name.clone(),
                    reason: "invalid characters in repo ID".into(),
                }
            })?;
            let mut repo = Repo::new(repo_id);
            let (io, ic, re) = parse_entries_into_repo(&mut repo, entries);
            rf.repos.insert(
                repo.id.clone(),
                SectionBlock {
                    header_comments,
                    data: repo,
                    item_comments: ic,
                    item_order: io,
                    raw_entries: re,
                },
            );
        }
    }
    Ok(rf)
}

// ============================================================================
// RepoFile implementation
// ============================================================================

impl RepoFile {
    /// Create an empty RepoFile.
    pub fn new() -> Self {
        RepoFile {
            preamble: Vec::new(),
            main: None,
            repos: IndexMap::new(),
        }
    }

    /// Parse a .repo file string.
    pub fn parse(input: &str) -> std::result::Result<Self, ParseError> {
        let mut state = ParseState {
            preamble: Vec::new(),
            pending_comments: Vec::new(),
            current_section: None,
            current_entries: Vec::new(),
            sections: IndexMap::new(),
            section_header_comments: IndexMap::new(),
        };

        for (line_idx, raw_line) in input.lines().enumerate() {
            let trimmed = raw_line.trim();

            if trimmed.is_empty() {
                if state.current_section.is_some() {
                    state.pending_comments.push(String::new());
                } else {
                    state.preamble.push(String::new());
                }
                continue;
            }

            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                if state.current_section.is_some() {
                    state.pending_comments.push(raw_line.to_owned());
                } else {
                    state.preamble.push(raw_line.to_owned());
                }
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Flush current section (keep pending_comments for the new section)
                if let Some(ref sec_name) = state.current_section.take() {
                    state.sections.insert(
                        sec_name.clone(),
                        std::mem::take(&mut state.current_entries),
                    );
                }

                let section_name = trimmed[1..trimmed.len() - 1].trim().to_string();
                if section_name.is_empty() {
                    return Err(ParseError::EmptySectionName);
                }
                if section_name != "main" {
                    if RepoId::try_new(section_name.as_str()).is_err() {
                        return Err(ParseError::InvalidRepoId {
                            id: section_name.clone(),
                            reason: "invalid characters in repo ID".into(),
                        });
                    }
                }

                // Assign pending comments as header_comments of the NEW section
                if !state.pending_comments.is_empty() {
                    state
                        .section_header_comments
                        .insert(section_name.clone(), std::mem::take(&mut state.pending_comments));
                }

                state.current_section = Some(section_name);
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value_part = &trimmed[eq_pos + 1..];
                let (value, inline_comment) = split_value_and_comment(value_part);

                if key.is_empty() {
                    return Err(ParseError::MissingEquals {
                        line: line_idx + 1,
                        line_text: raw_line.to_owned(),
                    });
                }

                let entry = RawLine {
                    key,
                    value: value.trim().to_string(),
                    inline_comment,
                    leading_comments: std::mem::take(&mut state.pending_comments),
                };

                if state.current_section.is_some() {
                    state.current_entries.push(entry);
                } else {
                    state.preamble.push(raw_line.to_owned());
                }
            } else {
                return Err(ParseError::MissingEquals {
                    line: line_idx + 1,
                    line_text: raw_line.to_owned(),
                });
            }
        }

        // Flush final section and any remaining pending comments
        if let Some(ref sec_name) = state.current_section.take() {
            if !state.pending_comments.is_empty()
                || !state.section_header_comments.contains_key(sec_name)
            {
                state
                    .section_header_comments
                    .insert(sec_name.clone(), std::mem::take(&mut state.pending_comments));
            }
            state
                .sections
                .insert(sec_name.clone(), std::mem::take(&mut state.current_entries));
        } else if !state.pending_comments.is_empty() {
            state.preamble.extend(std::mem::take(&mut state.pending_comments));
        }

        build_repofile(state)
    }

    /// Render the RepoFile back to INI text.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for line in &self.preamble {
            render_line(&mut out, line);
        }
        if let Some(ref block) = self.main {
            for c in &block.header_comments {
                render_line(&mut out, c);
            }
            out.push_str("[main]\n");
            render_section_entries(&mut out, block);
        }
        for (repo_id, block) in &self.repos {
            for c in &block.header_comments {
                render_line(&mut out, c);
            }
            out.push_str(&format!("[{}]\n", repo_id.as_ref()));
            render_section_entries(&mut out, block);
        }
        out
    }

    // ---- Accessors ----

    pub fn get(&self, id: &RepoId) -> Option<&SectionBlock<Repo>> {
        self.repos.get(id)
    }

    pub fn get_mut(&mut self, id: &RepoId) -> Option<&mut SectionBlock<Repo>> {
        self.repos.get_mut(id)
    }

    pub fn add(&mut self, repo: Repo) -> std::result::Result<(), crate::error::AddRepoError> {
        let id = repo.id.clone();
        if self.repos.contains_key(&id) {
            return Err(crate::error::AddRepoError {
                id: id.to_string(),
            });
        }
        self.repos.insert(
            id,
            SectionBlock {
                header_comments: Vec::new(),
                data: repo,
                item_comments: IndexMap::new(),
                item_order: Vec::new(),
                raw_entries: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn set(&mut self, repo: Repo) {
        let id = repo.id.clone();
        self.repos.insert(
            id,
            SectionBlock {
                header_comments: Vec::new(),
                data: repo,
                item_comments: IndexMap::new(),
                item_order: Vec::new(),
                raw_entries: Vec::new(),
            },
        );
    }

    pub fn remove(&mut self, id: &RepoId) -> Option<SectionBlock<Repo>> {
        self.repos.shift_remove(id)
    }

    pub fn contains(&self, id: &RepoId) -> bool {
        self.repos.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.repos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.repos.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RepoId, &SectionBlock<Repo>)> {
        self.repos.iter()
    }

    pub fn repo_ids(&self) -> impl Iterator<Item = &RepoId> {
        self.repos.keys()
    }

    pub fn main(&self) -> Option<&SectionBlock<MainConfig>> {
        self.main.as_ref()
    }

    pub fn main_mut(&mut self) -> Option<&mut SectionBlock<MainConfig>> {
        self.main.as_mut()
    }

    pub fn set_main(&mut self, config: MainConfig) {
        self.main = Some(SectionBlock {
            header_comments: Vec::new(),
            data: config,
            item_comments: IndexMap::new(),
            item_order: Vec::new(),
            raw_entries: Vec::new(),
        });
    }

    pub fn remove_main(&mut self) {
        self.main = None;
    }
}

impl Default for RepoFile {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for RepoFile {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        RepoFile::parse(s)
    }
}

// ============================================================================
// Render helpers
// ============================================================================

fn render_line(out: &mut String, line: &str) {
    out.push_str(line);
    if !line.ends_with('\n') {
        out.push('\n');
    }
}

fn render_section_entries<T: std::fmt::Debug>(out: &mut String, block: &SectionBlock<T>) {
    for entry in &block.raw_entries {
        for c in &entry.leading_comments {
            render_line(out, c);
        }
        let mut line = format!("{}={}", entry.key, entry.value);
        if let Some(ref ic) = entry.inline_comment {
            line.push_str(&format!(" #{}", ic));
        }
        out.push_str(&line);
        out.push('\n');
    }
}
