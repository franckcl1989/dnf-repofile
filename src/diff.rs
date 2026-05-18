use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::repofile::RepoFile;
use crate::types::RepoId;
use indexmap::IndexMap;

/// Result of comparing two entire `.repo` files
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub main_changes: Option<ConfigDiff>,
    pub repos_added: Vec<RepoId>,
    pub repos_removed: Vec<RepoId>,
    pub repos_modified: IndexMap<RepoId, RepoDiff>,
    pub repos_unchanged: Vec<RepoId>,
    pub has_changes: bool,
}

/// Per-repository field-level diff between two `Repo` values
#[derive(Debug, Clone)]
pub struct RepoDiff {
    pub changed: Vec<(String, String, String)>,
    pub added: Vec<(String, String)>,
    pub removed: Vec<(String, String)>,
    pub has_changes: bool,
}

/// Field-level diff between two `MainConfig` values
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub changed: Vec<(String, String, String)>,
    pub added: Vec<(String, String)>,
    pub removed: Vec<(String, String)>,
    pub has_changes: bool,
}

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

    diff.has_changes = !diff.changed.is_empty() || !diff.added.is_empty() || !diff.removed.is_empty();
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

    diff.has_changes = !diff.changed.is_empty() || !diff.added.is_empty() || !diff.removed.is_empty();
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
