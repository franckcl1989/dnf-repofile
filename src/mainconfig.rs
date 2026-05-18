//! The DNF `[main]` configuration section.
//!
//! This module defines [`MainConfig`], a strongly-typed representation of the
//! DNF `[main]` section from `/etc/dnf/dnf.conf`. It contains 57 fields
//! covering:
//!
//! - **Paths** — `cachedir`, `logdir`, `installroot`, `reposdir`, `varsdir`, etc.
//! - **Booleans** — `keepcache`, `best`, `strict`, `plugins`, `obsoletes`, etc.
//! - **Numerics** — `debuglevel`, `log_rotate`, `installonly_limit`, `errorlevel`, etc.
//! - **Enums** — `multilib_policy`, `persistence`, `rpmverbosity`
//! - **Raw data** — `extras` for keys not known to the library
//!
//! Use [`MainConfig::default()`] to create an empty config, or parse one via
//! [`RepoFile::parse`](crate::RepoFile::parse).

use crate::types::*;
use camino::Utf8PathBuf;
use indexmap::IndexMap;

/// DNF `[main]` configuration section.
///
/// Represents all 57 known DNF main configuration options as strongly-typed
/// optional fields. Unknown keys are stashed in [`extras`](MainConfig::extras)
/// for round-trip fidelity.
///
/// # Examples
///
/// ```
/// use dnf_repofile::MainConfig;
///
/// // Start with defaults (all fields None)
/// let mut config = MainConfig::default();
///
/// // Set some values
/// config.keepcache = Some(dnf_repofile::DnfBool::True);
/// config.debuglevel = dnf_repofile::DebugLevel::try_new(5).ok();
/// ```
///
/// Parse from a `.repo` file string via [`RepoFile`](crate::RepoFile):
///
/// ```
/// use dnf_repofile::RepoFile;
///
/// let input = "[main]\ncachedir=/var/cache/dnf\nkeepcache=0\ndebuglevel=2\n";
/// let rf = RepoFile::parse(input).unwrap();
/// let main = rf.main().unwrap();
/// assert_eq!(
///     main.data.cachedir.as_ref().map(|p| p.as_str()),
///     Some("/var/cache/dnf")
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MainConfig {
    /// System architecture (e.g., `"x86_64"`, `"aarch64"`).
    pub arch: Option<String>,
    /// Base hardware architecture.
    pub basearch: Option<String>,
    /// OS release version identifier.
    pub releasever: Option<String>,
    /// Directory for the DNF cache.
    pub cachedir: Option<Utf8PathBuf>,
    /// Directory for persistent data.
    pub persistdir: Option<Utf8PathBuf>,
    /// Directory for log files.
    pub logdir: Option<Utf8PathBuf>,
    /// Path to the DNF configuration file.
    pub config_file_path: Option<Utf8PathBuf>,
    /// Root directory for package installation.
    pub installroot: Option<Utf8PathBuf>,
    /// Directories containing `.repo` files.
    pub reposdir: Vec<Utf8PathBuf>,
    /// Directories containing variable definition files.
    pub varsdir: Vec<Utf8PathBuf>,
    /// Directories for plugin configuration files.
    pub pluginconfpath: Vec<Utf8PathBuf>,
    /// Directories containing DNF plugins.
    pub pluginpath: Vec<Utf8PathBuf>,
    /// Debug message verbosity level (0–10, default 2).
    pub debuglevel: Option<DebugLevel>,
    /// Log file verbosity level (0–10, default 9).
    pub logfilelevel: Option<LogLevel>,
    /// Number of log files to keep before rotation (default 4).
    pub log_rotate: Option<LogRotate>,
    /// Maximum log file size in bytes.
    pub log_size: Option<StorageSize>,
    /// Maximum number of kernel packages to keep (default 3).
    pub installonly_limit: Option<InstallOnlyLimit>,
    /// Error message verbosity level (0–10, default 3).
    pub errorlevel: Option<ErrorLevel>,
    /// Time in seconds between metadata timer syncs (default 10800).
    pub metadata_timer_sync: Option<MetadataTimerSync>,
    /// Allow `obsoletes` to replace packages from different vendors.
    pub allow_vendor_change: Option<DnfBool>,
    /// Automatically answer "no" to all questions.
    pub assumeno: Option<DnfBool>,
    /// Automatically answer "yes" to all questions.
    pub assumeyes: Option<DnfBool>,
    /// Check whether the running kernel is the latest installed.
    pub autocheck_running_kernel: Option<DnfBool>,
    /// Upgrade to the highest available package version.
    pub best: Option<DnfBool>,
    /// Run entirely from cache (no network).
    pub cacheonly: Option<DnfBool>,
    /// Check the age of the configuration file.
    pub check_config_file_age: Option<DnfBool>,
    /// Remove dependencies that are no longer needed.
    pub clean_requirements_on_remove: Option<DnfBool>,
    /// Enable debug output from the dependency solver.
    pub debug_solver: Option<DnfBool>,
    /// Default answer "yes" to all questions.
    pub defaultyes: Option<DnfBool>,
    /// Check available disk space before operations.
    pub diskspacecheck: Option<DnfBool>,
    /// Exclude packages from weak dependency autodetection.
    pub exclude_from_weak_autodetect: Option<DnfBool>,
    /// Exit immediately if a lock cannot be acquired.
    pub exit_on_lock: Option<DnfBool>,
    /// Verify GPG keys via DNS.
    pub gpgkey_dns_verification: Option<DnfBool>,
    /// Ignore architecture mismatches.
    pub ignorearch: Option<DnfBool>,
    /// Install weak dependencies automatically.
    pub install_weak_deps: Option<DnfBool>,
    /// Keep downloaded package files after installation.
    pub keepcache: Option<DnfBool>,
    /// Compress rotated log files.
    pub log_compress: Option<DnfBool>,
    /// Handle module obsoletes.
    pub module_obsoletes: Option<DnfBool>,
    /// Allow module stream switching.
    pub module_stream_switch: Option<DnfBool>,
    /// Handle package obsoletes.
    pub obsoletes: Option<DnfBool>,
    /// Enable DNF plugins.
    pub plugins: Option<DnfBool>,
    /// Protect the running kernel from removal.
    pub protect_running_kernel: Option<DnfBool>,
    /// Fail on any error during dependency resolution.
    pub strict: Option<DnfBool>,
    /// Upgrade group objects as a unit.
    pub upgrade_group_objects_upgrade: Option<DnfBool>,
    /// Enable zchunk metadata compression.
    pub zchunk: Option<DnfBool>,
    /// Packages that should only ever be installed, never upgraded.
    pub installonlypkgs: Vec<String>,
    /// Packages protected from automatic removal.
    pub protected_packages: Vec<String>,
    /// Packages excluded from weak dependency detection.
    pub exclude_from_weak: Vec<String>,
    /// Types of group packages to install.
    pub group_package_types: Vec<String>,
    /// Optional repository metadata types to download.
    pub optional_metadata_types: Vec<String>,
    /// RPM transaction flags (scripts, triggers, docs, etc.).
    pub tsflags: Vec<TsFlag>,
    /// Paths protected from `/usr` drift.
    pub usr_drift_protected_paths: Vec<String>,
    /// Multilib package installation policy.
    pub multilib_policy: Option<MultilibPolicy>,
    /// SQLite persistence mode for repository metadata.
    pub persistence: Option<Persistence>,
    /// RPM transaction verbosity level.
    pub rpmverbosity: Option<RpmVerbosity>,
    /// Module platform identifier for modularity.
    pub module_platform_id: Option<ModulePlatformId>,
    /// Unrecognized key-value pairs preserved for round-trip fidelity.
    pub extras: IndexMap<String, Vec<String>>,
}
