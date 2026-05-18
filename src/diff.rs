//! Diff engine for comparing DNF configuration files and values.
//!
//! Provides three diff functions and three result types:
//!
//! - [`diff_files`] — compares two complete [`RepoFile`] values,
//!   reporting added, removed, and modified repos plus any [`MainConfig`] changes.
//! - [`diff_repos`] — compares two individual [`Repo`] values field by field.
//! - [`diff_main`] — compares two [`MainConfig`] values field by field.
//!
//! Each diff reports three categories of changes: **added** (field present in B
//! but not in A), **removed** (field present in A but not in B), and **changed**
//! (field differs between A and B, showing both old and new values).

use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::repofile::RepoFile;
use crate::types::RepoId;
use indexmap::IndexMap;

/// Result of comparing two entire `.repo` files.
///
/// Reports added, removed, modified, and unchanged repos, plus any changes
/// to the `[main]` section.
///
/// # Examples
///
/// ```
/// use dnf_repofile::{RepoFile, diff_files};
///
/// let a = RepoFile::parse("[repo]\nname=Old\nbaseurl=https://a.com/\n").unwrap();
/// let b = RepoFile::parse("[repo]\nname=New\nbaseurl=https://b.com/\n").unwrap();
/// let diff = diff_files(&a, &b);
/// assert!(diff.has_changes);
/// ```
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Changes to the `[main]` section, if present.
    pub main_changes: Option<ConfigDiff>,
    /// Repo IDs present in B but not in A.
    pub repos_added: Vec<RepoId>,
    /// Repo IDs present in A but not in B.
    pub repos_removed: Vec<RepoId>,
    /// Repo IDs present in both files with field-level changes.
    pub repos_modified: IndexMap<RepoId, RepoDiff>,
    /// Repo IDs present in both files with identical values.
    pub repos_unchanged: Vec<RepoId>,
    /// Whether any changes were detected across all categories.
    pub has_changes: bool,
}

/// Per-repository field-level diff between two [`Repo`] values.
///
/// Each tuple in `changed` is `(field_name, old_value, new_value)`.
/// Each tuple in `added` and `removed` is `(field_name, value)`.
#[derive(Debug, Clone)]
pub struct RepoDiff {
    /// Fields whose values differ between A and B.
    pub changed: Vec<(String, String, String)>,
    /// Fields present in B but absent in A.
    pub added: Vec<(String, String)>,
    /// Fields present in A but absent in B.
    pub removed: Vec<(String, String)>,
    /// Whether any field-level changes were detected.
    pub has_changes: bool,
}

/// Field-level diff between two [`MainConfig`] values.
///
/// Each tuple in `changed` is `(field_name, old_value, new_value)`.
/// Each tuple in `added` and `removed` is `(field_name, value)`.
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    /// Fields whose values differ between A and B.
    pub changed: Vec<(String, String, String)>,
    /// Fields present in B but absent in A.
    pub added: Vec<(String, String)>,
    /// Fields present in A but absent in B.
    pub removed: Vec<(String, String)>,
    /// Whether any field-level changes were detected.
    pub has_changes: bool,
}

/// Compare two [`RepoFile`] values and produce a [`FileDiff`].
///
/// Diffs the `[main]` section (if present in both files), then enumerates
/// repos by ID to find added, removed, modified, and unchanged repos.
///
/// # Examples
///
/// ```
/// use dnf_repofile::{RepoFile, diff_files};
///
/// let a = RepoFile::parse("[repo]\nname=Old\nbaseurl=https://a.com/\n").unwrap();
/// let b = RepoFile::parse("[repo]\nname=New\nbaseurl=https://b.com/\n").unwrap();
/// let diff = diff_files(&a, &b);
/// assert!(diff.repos_modified.contains_key(
///     &dnf_repofile::RepoId::try_new("repo").unwrap()
/// ));
/// ```
pub fn diff_files(a: &RepoFile, b: &RepoFile) -> FileDiff {
    let mut diff = FileDiff {
        main_changes: None,
        repos_added: vec![],
        repos_removed: vec![],
        repos_modified: IndexMap::new(),
        repos_unchanged: vec![],
        has_changes: false,
    };

    match (&a.main, &b.main) {
        (Some(am), Some(bm)) => {
            let cd = diff_main(&am.data, &bm.data);
            if cd.has_changes {
                diff.has_changes = true;
                diff.main_changes = Some(cd);
            }
        }
        (None, Some(_)) | (Some(_), None) => {
            diff.has_changes = true;
        }
        (None, None) => {}
    }

    for (id, bb) in &b.repos {
        match a.repos.get(id) {
            None => {
                diff.repos_added.push(id.clone());
                diff.has_changes = true;
            }
            Some(ba) => {
                let rd = diff_repos(&ba.data, &bb.data);
                if rd.has_changes {
                    diff.repos_modified.insert(id.clone(), rd);
                    diff.has_changes = true;
                } else {
                    diff.repos_unchanged.push(id.clone());
                }
            }
        }
    }

    for (id, _) in &a.repos {
        if !b.repos.contains_key(id) {
            diff.repos_removed.push(id.clone());
            diff.has_changes = true;
        }
    }

    diff
}

/// Compare two [`Repo`] values and produce a [`RepoDiff`].
///
/// Compares the following fields: `name`, `baseurl`, `enabled`, `gpgcheck`,
/// `priority`, and `gpgkey`. Fields that are `None` in both directions are
/// considered absent; fields that change from `Some` to `None` or vice versa
/// are reported as removed or added respectively.
///
/// # Examples
///
/// ```
/// use dnf_repofile::{Repo, RepoId, diff_repos};
///
/// let mut a = Repo::new(RepoId::try_new("repo").unwrap());
/// a.name = Some(dnf_repofile::RepoName::try_new("Old Name").unwrap());
///
/// let mut b = Repo::new(RepoId::try_new("repo").unwrap());
/// b.name = Some(dnf_repofile::RepoName::try_new("New Name").unwrap());
///
/// let diff = diff_repos(&a, &b);
/// assert!(diff.has_changes);
/// ```
pub fn diff_repos(a: &Repo, b: &Repo) -> RepoDiff {
    let mut diff = RepoDiff {
        changed: vec![],
        added: vec![],
        removed: vec![],
        has_changes: false,
    };

    diff_opt(
        &mut diff,
        "name",
        a.name.as_ref().map(|n| n.as_ref().to_owned()),
        b.name.as_ref().map(|n| n.as_ref().to_owned()),
    );
    diff_opt(
        &mut diff,
        "baseurl",
        if a.baseurl.is_empty() {
            None
        } else {
            Some(
                a.baseurl
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        },
        if b.baseurl.is_empty() {
            None
        } else {
            Some(
                b.baseurl
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        },
    );
    diff_opt(
        &mut diff,
        "enabled",
        a.enabled.map(|d| d.to_string()),
        b.enabled.map(|d| d.to_string()),
    );
    diff_opt(
        &mut diff,
        "gpgcheck",
        a.gpgcheck.map(|d| d.to_string()),
        b.gpgcheck.map(|d| d.to_string()),
    );
    diff_opt(
        &mut diff,
        "priority",
        a.priority.map(|p| p.to_string()),
        b.priority.map(|p| p.to_string()),
    );
    diff_opt(
        &mut diff,
        "gpgkey",
        if a.gpgkey.is_empty() {
            None
        } else {
            Some(a.gpgkey.join(", "))
        },
        if b.gpgkey.is_empty() {
            None
        } else {
            Some(b.gpgkey.join(", "))
        },
    );

    diff.has_changes =
        !diff.changed.is_empty() || !diff.added.is_empty() || !diff.removed.is_empty();
    diff
}

fn diff_opt(diff: &mut RepoDiff, key: &str, a: Option<String>, b: Option<String>) {
    match (a, b) {
        (None, Some(nv)) => diff.added.push((key.to_string(), nv)),
        (Some(ov), None) => diff.removed.push((key.to_string(), ov)),
        (Some(ov), Some(nv)) if ov != nv => diff.changed.push((key.to_string(), ov, nv)),
        _ => {}
    }
}

/// Compare two [`MainConfig`] values and produce a [`ConfigDiff`].
///
/// Currently compares `debuglevel` and `best`. This field list will expand
/// in future releases.
///
/// # Examples
///
/// ```
/// use dnf_repofile::{MainConfig, diff_main};
///
/// let a = MainConfig::default();
/// let b = MainConfig::default();
/// let diff = diff_main(&a, &b);
/// assert!(!diff.has_changes);
/// ```
pub fn diff_main(a: &MainConfig, b: &MainConfig) -> ConfigDiff {
    let mut diff = ConfigDiff {
        changed: vec![],
        added: vec![],
        removed: vec![],
        has_changes: false,
    };

    diff_opt_cfg(
        &mut diff,
        "debuglevel",
        a.debuglevel.map(|d| d.to_string()),
        b.debuglevel.map(|d| d.to_string()),
    );
    diff_opt_cfg(
        &mut diff,
        "best",
        a.best.map(|d| d.to_string()),
        b.best.map(|d| d.to_string()),
    );

    diff.has_changes =
        !diff.changed.is_empty() || !diff.added.is_empty() || !diff.removed.is_empty();
    diff
}

fn diff_opt_cfg(diff: &mut ConfigDiff, key: &str, a: Option<String>, b: Option<String>) {
    match (a, b) {
        (None, Some(nv)) => diff.added.push((key.to_string(), nv)),
        (Some(ov), None) => diff.removed.push((key.to_string(), ov)),
        (Some(ov), Some(nv)) if ov != nv => diff.changed.push((key.to_string(), ov, nv)),
        _ => {}
    }
}
