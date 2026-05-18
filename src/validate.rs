//! Validation engine for checking DNF repository configuration consistency.
//!
//! Provides [`ValidationReport`] as the top-level result type, containing
//! separate lists of errors and warnings. Each finding is a [`ValidationIssue`]
//! with a severity level ([`IssueLevel`]), a location ([`IssueLocation`]), an
//! optional field name, and a human-readable message.
//!
//! # Current validation rules
//!
//! - **Error**: repo missing URL source (no `baseurl`, `mirrorlist`, or `metalink`)
//! - **Warning**: both `baseurl` and `mirrorlist`/`metalink` are set
//! - **Warning**: `gpgkey` is set but neither `gpgcheck` nor `repo_gpgcheck` is enabled

use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::types::{DnfBool, RepoId};

/// Report containing validation errors and warnings.
///
/// Use [`is_ok()`](ValidationReport::is_ok) to check if the configuration is
/// valid (no errors). Warnings alone do not indicate invalidity.
///
/// # Examples
///
/// ```
/// use dnf_repofile::{Repo, RepoId, ValidationReport};
///
/// // A repo with no URL source is invalid
/// let repo = Repo::new(RepoId::try_new("test").unwrap());
/// let report = repo.validate();
/// assert!(!report.is_ok());
/// assert_eq!(report.errors.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Issues classified as errors (configuration is invalid).
    pub errors: Vec<ValidationIssue>,
    /// Issues classified as warnings (advisory, non-fatal).
    pub warnings: Vec<ValidationIssue>,
}

/// A single validation finding with severity level and location.
///
/// Each issue identifies where the problem was found, its severity, the
/// specific field (if applicable), and a human-readable message.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Whether this is an error or a warning.
    pub level: IssueLevel,
    /// Where the issue was found (file, repo, or main section).
    pub location: IssueLocation,
    /// The specific option field name (e.g., `"baseurl"`, `"gpgkey"`), if applicable.
    pub field: Option<String>,
    /// A human-readable description of the issue.
    pub message: String,
}

/// Severity level for a validation issue.
///
/// - [`Error`](IssueLevel::Error) — configuration is invalid and should not be used.
/// - [`Warning`](IssueLevel::Warning) — advisory notice; configuration may still work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueLevel {
    /// A hard error: the configuration is invalid.
    Error,
    /// A soft warning: the configuration may still work but is suspicious.
    Warning,
}

/// Identifies where a validation issue was found.
#[derive(Debug, Clone)]
pub enum IssueLocation {
    /// The issue is in a specific `.repo` file on disk.
    File(String),
    /// The issue is in a specific repository section.
    Repo(RepoId),
    /// The issue is in the `[main]` section.
    Main,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    /// Create a new empty validation report.
    pub fn new() -> Self {
        ValidationReport {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns `true` if there are no errors (warnings are ignored).
    ///
    /// A report with only warnings is still considered valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::ValidationReport;
    ///
    /// let report = ValidationReport::new();
    /// assert!(report.is_ok());
    /// ```
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns `true` if there are any issues (errors or warnings).
    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

impl Repo {
    /// Validate a single repository's configuration.
    ///
    /// Checks:
    ///
    /// - At least one URL source is present (`baseurl`, `mirrorlist`, or `metalink`).
    /// - `baseurl` is not set alongside `mirrorlist` or `metalink` (warns of
    ///   potential ambiguity).
    /// - `gpgkey` is set without `gpgcheck` or `repo_gpgcheck` being enabled (warns).
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        let mut r = ValidationReport::new();

        if self.baseurl.is_empty() && self.mirrorlist.is_none() && self.metalink.is_none() {
            let issue = ValidationIssue {
                level: IssueLevel::Error,
                location: IssueLocation::Repo(self.id.clone()),
                field: Some("baseurl".into()),
                message:
                    "repo must have at least one URL source (baseurl, mirrorlist, or metalink)"
                        .into(),
            };
            r.errors.push(issue);
        }

        if !self.baseurl.is_empty() && (self.mirrorlist.is_some() || self.metalink.is_some()) {
            r.warnings.push(ValidationIssue {
                level: IssueLevel::Warning,
                location: IssueLocation::Repo(self.id.clone()),
                field: Some("baseurl".into()),
                message:
                    "baseurl and mirrorlist/metalink both set; DNF may ignore mirrorlist/metalink"
                        .into(),
            });
        }

        if !self.gpgkey.is_empty()
            && self.gpgcheck != Some(DnfBool::True)
            && self.repo_gpgcheck != Some(DnfBool::True)
        {
            r.warnings.push(ValidationIssue {
                level: IssueLevel::Warning,
                location: IssueLocation::Repo(self.id.clone()),
                field: Some("gpgkey".into()),
                message: "gpgkey is set but gpgcheck and repo_gpgcheck are not enabled".into(),
            });
        }

        r
    }
}

impl MainConfig {
    /// Validate the `[main]` configuration.
    ///
    /// Currently returns an empty (valid) report. Future releases will add
    /// validation rules for the `[main]` section.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        ValidationReport::new()
    }
}
