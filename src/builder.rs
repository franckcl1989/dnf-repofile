//! Builder-pattern API for constructing [`Repo`] values.
//!
//! [`RepoBuilder`] provides a fluent interface for programmatic creation of
//! repository configurations without manually setting each field on a
//! [`Repo`] struct.
//!
//! # Examples
//!
//! ```
//! use dnf_repofile::{RepoBuilder, RepoId, RepoName, DnfBool, Priority};
//!
//! let repo = RepoBuilder::new(RepoId::try_new("custom").unwrap())
//!     .name(RepoName::try_new("Custom Repository").unwrap())
//!     .baseurl("https://example.com/".parse().unwrap())
//!     .enabled(DnfBool::yes())
//!     .gpgcheck(DnfBool::yes())
//!     .priority(Priority::try_new(50).unwrap())
//!     .build();
//!
//! assert_eq!(repo.name.as_ref().unwrap().as_ref(), "Custom Repository");
//! ```

use crate::repo::Repo;
use crate::types::*;
use url::Url;

/// Builder-pattern API for constructing a [`Repo`] with a fluent interface.
///
/// Create a new builder with [`RepoBuilder::new`] or clone an existing repo
/// with [`RepoBuilder::from`].
///
/// # Examples
///
/// ```
/// use dnf_repofile::{RepoBuilder, RepoId, DnfBool};
///
/// let repo = RepoBuilder::new(RepoId::try_new("myrepo").unwrap())
///     .enabled(DnfBool::yes())
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct RepoBuilder {
    repo: Repo,
}

impl RepoBuilder {
    /// Create a new builder with the given repository ID.
    ///
    /// All other fields are initialized to `None` or empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::{RepoBuilder, RepoId};
    ///
    /// let builder = RepoBuilder::new(RepoId::try_new("test").unwrap());
    /// ```
    pub fn new(id: RepoId) -> Self {
        RepoBuilder {
            repo: Repo::new(id),
        }
    }

    /// Create a builder pre-populated from an existing [`Repo`].
    ///
    /// Useful for making a modified copy of an existing configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::{RepoBuilder, Repo, RepoId};
    ///
    /// let original = Repo::new(RepoId::try_new("test").unwrap());
    /// let builder = RepoBuilder::from(&original);
    /// ```
    pub fn from(existing: &Repo) -> Self {
        RepoBuilder {
            repo: existing.clone(),
        }
    }

    /// Consume the builder and produce the final [`Repo`].
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::{RepoBuilder, RepoId};
    ///
    /// let repo = RepoBuilder::new(RepoId::try_new("test").unwrap()).build();
    /// ```
    #[must_use]
    pub fn build(self) -> Repo {
        self.repo
    }

    /// Set the human-readable repository name.
    pub fn name(mut self, v: RepoName) -> Self {
        self.repo.name = Some(v);
        self
    }

    /// Add a base URL to the repository.
    ///
    /// Multiple URLs can be added by chaining this method.
    pub fn baseurl(mut self, v: Url) -> Self {
        self.repo.baseurl.push(v);
        self
    }

    /// Set all base URLs at once, replacing any existing URLs.
    pub fn baseurls(mut self, v: Vec<Url>) -> Self {
        self.repo.baseurl = v;
        self
    }

    /// Set the mirror list URL.
    pub fn mirrorlist(mut self, v: Url) -> Self {
        self.repo.mirrorlist = Some(v);
        self
    }

    /// Set the metalink URL.
    pub fn metalink(mut self, v: Url) -> Self {
        self.repo.metalink = Some(v);
        self
    }

    /// Add a GPG key URL.
    ///
    /// Multiple keys can be added by chaining this method.
    pub fn gpgkey(mut self, v: &str) -> Self {
        self.repo.gpgkey.push(v.to_string());
        self
    }

    /// Set all GPG key URLs at once, replacing any existing keys.
    pub fn gpgkeys(mut self, v: Vec<String>) -> Self {
        self.repo.gpgkey = v;
        self
    }

    /// Set whether the repository is enabled.
    pub fn enabled(mut self, v: DnfBool) -> Self {
        self.repo.enabled = Some(v);
        self
    }

    /// Set whether to GPG-check packages from this repo.
    pub fn gpgcheck(mut self, v: DnfBool) -> Self {
        self.repo.gpgcheck = Some(v);
        self
    }

    /// Set whether to GPG-check repository metadata.
    pub fn repo_gpgcheck(mut self, v: DnfBool) -> Self {
        self.repo.repo_gpgcheck = Some(v);
        self
    }

    /// Set the repository priority (1–99, lower = higher priority).
    pub fn priority(mut self, v: Priority) -> Self {
        self.repo.priority = Some(v);
        self
    }

    /// Set the repository cost (higher = less preferred).
    pub fn cost(mut self, v: Cost) -> Self {
        self.repo.cost = Some(v);
        self
    }

    /// Mark the repository as a module hotfix repository.
    pub fn module_hotfixes(mut self, v: DnfBool) -> Self {
        self.repo.module_hotfixes = Some(v);
        self
    }

    /// Set the repository metadata type (e.g., `rpm-md`).
    pub fn metadata_type(mut self, v: RepoMetadataType) -> Self {
        self.repo.metadata_type = Some(v);
        self
    }

    /// Set the media ID for DVD/media-based repositories.
    pub fn mediaid(mut self, v: &str) -> Self {
        self.repo.mediaid = Some(v.to_string());
        self
    }

    /// Add a package name glob to the exclude list.
    pub fn excludepkgs(mut self, v: &str) -> Self {
        self.repo.excludepkgs.push(v.to_string());
        self
    }

    /// Add a package name glob to the include list.
    pub fn includepkgs(mut self, v: &str) -> Self {
        self.repo.includepkgs.push(v.to_string());
        self
    }

    /// Set whether to skip the repository if it is unavailable.
    pub fn skip_if_unavailable(mut self, v: DnfBool) -> Self {
        self.repo.skip_if_unavailable = Some(v);
        self
    }

    /// Set the number of retries for network operations.
    pub fn retries(mut self, v: Retries) -> Self {
        self.repo.retries = Some(v);
        self
    }

    /// Set the network timeout in seconds.
    pub fn timeout(mut self, v: TimeoutSeconds) -> Self {
        self.repo.timeout = Some(v);
        self
    }

    /// Set the maximum number of parallel downloads.
    pub fn max_parallel_downloads(mut self, v: MaxParallelDownloads) -> Self {
        self.repo.max_parallel_downloads = Some(v);
        self
    }

    /// Set the proxy configuration.
    pub fn proxy(mut self, v: ProxySetting) -> Self {
        self.repo.proxy = v;
        self
    }

    /// Set the username for repository authentication.
    pub fn username(mut self, v: Username) -> Self {
        self.repo.username = Some(v);
        self
    }

    /// Set the password for repository authentication.
    pub fn password(mut self, v: Password) -> Self {
        self.repo.password = Some(v);
        self
    }

    /// Add an extra (unknown) key-value pair to the repository configuration.
    ///
    /// Unknown keys are preserved for round-trip fidelity when rendering.
    /// If the same key is added multiple times, the values are appended.
    pub fn extra(mut self, key: &str, value: &str) -> Self {
        self.repo
            .extras
            .entry(key.to_string())
            .or_default()
            .push(value.to_string());
        self
    }
}
