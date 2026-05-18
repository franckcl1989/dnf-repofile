use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::types::{DnfBool, RepoId};

/// Report containing validation errors and warnings
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

/// A single validation finding with severity level and location
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: IssueLevel,
    pub location: IssueLocation,
    pub field: Option<String>,
    pub message: String,
}

/// Severity level for a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueLevel {
    Error,
    Warning,
}

/// Identifies where a validation issue was found
#[derive(Debug, Clone)]
pub enum IssueLocation {
    File(String),
    Repo(RepoId),
    Main,
}

impl ValidationReport {
    pub fn new() -> Self {
        ValidationReport {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

impl Repo {
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

        if !self.baseurl.is_empty()
            && (self.mirrorlist.is_some() || self.metalink.is_some())
        {
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
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        ValidationReport::new()
    }
}
