//! The fully typed [`Repo`] struct representing a `[repo-id]` section.
//!
//! This module defines [`Repo`], which models all 48 known DNF repository
//! configuration options as strongly-typed fields, plus an `extras` map for
//! unknown keys. Use [`Repo::new()`] to create an empty repo, or the
//! [`RepoBuilder`](crate::RepoBuilder) for ergonomic construction.
//!
//! Parse a [`Repo`] indirectly through [`RepoFile::parse`](crate::RepoFile::parse),
//! which populates `Repo` values from the INI sections of a `.repo` file.

use crate::types::*;
use camino::Utf8PathBuf;
use indexmap::IndexMap;
use url::Url;

/// A fully typed `[repo-id]` section from a `.repo` file.
///
/// Contains all 48 known DNF repository configuration options as
/// strongly-typed fields. Unknown keys are preserved in
/// [`extras`](Repo::extras) for round-trip fidelity.
///
/// Use [`Repo::new`] to create an empty repo, or
/// [`RepoBuilder`](crate::RepoBuilder) for ergonomic construction.
///
/// # Examples
///
/// ```
/// use dnf_repofile::{Repo, RepoId, RepoName, DnfBool};
///
/// let mut repo = Repo::new(RepoId::try_new("myrepo").unwrap());
/// repo.name = Some(RepoName::try_new("My Repository").unwrap());
/// repo.baseurl.push("https://example.com/repo/".parse().unwrap());
/// repo.enabled = Some(DnfBool::True);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    // ===== repo-only =====
    /// Repository ID (the `[repo-id]` section name).
    pub id: RepoId,
    /// Human-readable repository name.
    pub name: Option<RepoName>,
    /// Base URLs for the repository (multiple values allowed).
    pub baseurl: Vec<Url>,
    /// Mirror list URL for the repository.
    pub mirrorlist: Option<Url>,
    /// Metalink URL for the repository.
    pub metalink: Option<Url>,
    /// GPG key URLs for package signature verification.
    pub gpgkey: Vec<String>,
    /// Whether the repository is enabled.
    pub enabled: Option<DnfBool>,
    /// Repository priority (1–99, lower = higher priority).
    pub priority: Option<Priority>,
    /// Repository cost (higher = less preferred).
    pub cost: Option<Cost>,
    /// Mark as a module hotfix repository.
    pub module_hotfixes: Option<DnfBool>,
    /// Repository metadata type (e.g., `rpm-md`).
    pub metadata_type: Option<RepoMetadataType>,
    /// Media identifier for DVD/media-based repos.
    pub mediaid: Option<String>,
    /// Metadata types to fetch even when the repo is disabled.
    pub enabled_metadata: Vec<String>,

    // ===== shared =====
    /// Package name globs to exclude from this repo.
    pub excludepkgs: Vec<String>,
    /// Package name globs to include (only) from this repo.
    pub includepkgs: Vec<String>,
    /// Enable GPG signature verification on packages.
    pub gpgcheck: Option<DnfBool>,
    /// Enable GPG signature verification on repository metadata.
    pub repo_gpgcheck: Option<DnfBool>,
    /// Enable GPG verification on local packages.
    pub localpkg_gpgcheck: Option<DnfBool>,
    /// Skip the repo if it is unavailable.
    pub skip_if_unavailable: Option<DnfBool>,
    /// Enable delta RPM support.
    pub deltarpm: Option<DnfBool>,
    /// Maximum delta RPM percentage (0–100, default 75).
    pub deltarpm_percentage: Option<DeltaRpmPercentage>,
    /// Enable group metadata support.
    pub enablegroups: Option<DnfBool>,
    /// Enable fastest mirror detection.
    pub fastestmirror: Option<DnfBool>,
    /// Enable counting of repository usage.
    pub countme: Option<DnfBool>,
    /// Maximum bandwidth usage in bytes.
    pub bandwidth: Option<StorageSize>,
    /// Bandwidth throttle (absolute or percentage).
    pub throttle: Option<Throttle>,
    /// Minimum download rate in bytes before timeout.
    pub minrate: Option<StorageSize>,
    /// Number of retries before giving up.
    pub retries: Option<Retries>,
    /// Timeout in seconds for network operations.
    pub timeout: Option<TimeoutSeconds>,
    /// Maximum parallel downloads (0–20, default 3).
    pub max_parallel_downloads: Option<MaxParallelDownloads>,
    /// Metadata expiration time.
    pub metadata_expire: Option<MetadataExpire>,
    /// IP protocol version preference.
    pub ip_resolve: Option<IpResolve>,
    /// Enable SSL certificate verification.
    pub sslverify: Option<DnfBool>,
    /// Enable SSL status verification.
    pub sslverifystatus: Option<DnfBool>,
    /// Path to the SSL CA certificate.
    pub sslcacert: Option<Utf8PathBuf>,
    /// Path to the SSL client certificate.
    pub sslclientcert: Option<Utf8PathBuf>,
    /// Path to the SSL client key.
    pub sslclientkey: Option<Utf8PathBuf>,
    /// Proxy configuration (unset, disabled, or URL).
    pub proxy: ProxySetting,
    /// Proxy username.
    pub proxy_username: Option<ProxyUsername>,
    /// Proxy password.
    pub proxy_password: Option<ProxyPassword>,
    /// Proxy authentication method.
    pub proxy_auth_method: Option<ProxyAuthMethod>,
    /// Enable SSL verification for proxy connections.
    pub proxy_sslverify: Option<DnfBool>,
    /// Path to the proxy SSL CA certificate.
    pub proxy_sslcacert: Option<Utf8PathBuf>,
    /// Path to the proxy SSL client certificate.
    pub proxy_sslclientcert: Option<Utf8PathBuf>,
    /// Path to the proxy SSL client key.
    pub proxy_sslclientkey: Option<Utf8PathBuf>,
    /// Username for repository authentication.
    pub username: Option<Username>,
    /// Password for repository authentication.
    pub password: Option<Password>,
    /// Custom HTTP User-Agent header.
    pub user_agent: Option<UserAgent>,

    // ===== unknown =====
    /// Unrecognized key-value pairs preserved for round-trip fidelity.
    pub extras: IndexMap<String, Vec<String>>,
}

impl Repo {
    /// Create a new `Repo` with only an ID set; all other fields default to
    /// `None` (or empty).
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::{Repo, RepoId};
    ///
    /// let repo = Repo::new(RepoId::try_new("myrepo").unwrap());
    /// assert_eq!(repo.id.as_ref(), "myrepo");
    /// assert!(repo.baseurl.is_empty());
    /// ```
    pub fn new(id: RepoId) -> Self {
        Repo {
            id,
            name: None,
            baseurl: Vec::new(),
            mirrorlist: None,
            metalink: None,
            gpgkey: Vec::new(),
            enabled: None,
            priority: None,
            cost: None,
            module_hotfixes: None,
            metadata_type: None,
            mediaid: None,
            enabled_metadata: Vec::new(),
            excludepkgs: Vec::new(),
            includepkgs: Vec::new(),
            gpgcheck: None,
            repo_gpgcheck: None,
            localpkg_gpgcheck: None,
            skip_if_unavailable: None,
            deltarpm: None,
            deltarpm_percentage: None,
            enablegroups: None,
            fastestmirror: None,
            countme: None,
            bandwidth: None,
            throttle: None,
            minrate: None,
            retries: None,
            timeout: None,
            max_parallel_downloads: None,
            metadata_expire: None,
            ip_resolve: None,
            sslverify: None,
            sslverifystatus: None,
            sslcacert: None,
            sslclientcert: None,
            sslclientkey: None,
            proxy: ProxySetting::Unset,
            proxy_username: None,
            proxy_password: None,
            proxy_auth_method: None,
            proxy_sslverify: None,
            proxy_sslcacert: None,
            proxy_sslclientcert: None,
            proxy_sslclientkey: None,
            username: None,
            password: None,
            user_agent: None,
            extras: IndexMap::new(),
        }
    }

    /// Determine the URL source type for this repository.
    ///
    /// Returns `Some(UrlSource)` if at least one URL source is configured,
    /// checking `baseurl` first, then `mirrorlist`, then `metalink`.
    /// Returns `None` if no URL source is configured.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::{Repo, RepoId, UrlSource};
    ///
    /// let mut repo = Repo::new(RepoId::try_new("myrepo").unwrap());
    /// repo.baseurl.push("https://example.com/".parse().unwrap());
    ///
    /// let source = repo.url_source().unwrap();
    /// match source {
    ///     UrlSource::BaseUrl(urls) => assert_eq!(urls.len(), 1),
    ///     _ => panic!("expected BaseUrl"),
    /// }
    /// ```
    pub fn url_source(&self) -> Option<UrlSource> {
        if !self.baseurl.is_empty() {
            Some(UrlSource::BaseUrl(self.baseurl.clone()))
        } else if let Some(ref url) = self.mirrorlist {
            Some(UrlSource::MirrorList(url.clone()))
        } else {
            self.metalink
                .as_ref()
                .map(|url| UrlSource::Metalink(url.clone()))
        }
    }
}
