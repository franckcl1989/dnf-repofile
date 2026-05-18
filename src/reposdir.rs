//! Directory-level management of `.repo` files on disk.
//!
//! [`ReposDir`] represents a directory (typically `/etc/yum.repos.d/`)
//! containing multiple `.repo` files. It loads all files on construction,
//! providing a unified API for querying and modifying repos across files.
//!
//! # Examples
//!
//! ```
//! use dnf_repofile::ReposDir;
//!
//! // Load all .repo files from a directory
//! // (in real usage: ReposDir::load("/etc/yum.repos.d"))
//! // For testing, a non-existent directory produces an empty ReposDir
//! let rd = ReposDir::load("/tmp/nonexistent").unwrap();
//! assert!(rd.repo_count() == 0);
//! ```

use crate::error::Result;
use crate::repo::Repo;
use crate::repofile::RepoFile;
use crate::types::RepoId;
use crate::validate::{IssueLevel, IssueLocation, ValidationIssue, ValidationReport};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages a directory of `.repo` files on disk.
///
/// Loaded via [`ReposDir::load`], which reads all `*.repo` files in the
/// given directory. Provides methods for querying repos across files,
/// saving changes, and cross-file validation (duplicate repo detection).
#[derive(Debug)]
pub struct ReposDir {
    path: PathBuf,
    files: IndexMap<String, RepoFile>,
}

impl ReposDir {
    /// Load all `.repo` files from a directory on disk.
    ///
    /// Reads all files ending in `.repo` from the given path, parses each
    /// one, and indexes them by filename. Files that fail to parse are
    /// silently skipped.
    ///
    /// Returns an empty [`ReposDir`] if the path does not exist or contains
    /// no `.repo` files.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Io`] if the directory cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::ReposDir;
    ///
    /// let rd = ReposDir::load("/etc/yum.repos.d").unwrap_or_else(|_| {
    ///     ReposDir::load("/tmp").unwrap()
    /// });
    /// ```
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut files = IndexMap::new();

        if path.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(&path)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.ends_with(".repo"))
                        .unwrap_or(false)
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(contents) = fs::read_to_string(entry.path()) {
                    if let Ok(rf) = RepoFile::parse(&contents) {
                        files.insert(name, rf);
                    }
                }
            }
        }

        Ok(ReposDir { path, files })
    }

    /// Save all loaded `.repo` files back to disk.
    ///
    /// Writes each file to its original path. If any write fails, returns
    /// a `Vec` of `(filename, io_error)` tuples for the failed files.
    ///
    /// # Errors
    ///
    /// Returns `Err(Vec)` containing all errors encountered during saving.
    /// If the vector is non-empty, some files may have been saved successfully.
    pub fn save_all(&self) -> std::result::Result<(), Vec<(String, std::io::Error)>> {
        let mut errs = Vec::new();
        for (name, rf) in &self.files {
            if let Err(e) = fs::write(self.path.join(name), rf.render()) {
                errs.push((name.clone(), e));
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    /// Save a single `.repo` file by filename.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] with `NotFound` kind if the filename is
    /// not tracked, or the underlying write error if saving fails.
    pub fn save(&self, filename: &str) -> std::result::Result<(), std::io::Error> {
        match self.files.get(filename) {
            Some(rf) => fs::write(self.path.join(filename), rf.render()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            )),
        }
    }

    /// Return the list of tracked `.repo` filenames.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::ReposDir;
    ///
    /// let rd = ReposDir::load("/tmp/nonexistent").unwrap();
    /// assert!(rd.file_names().is_empty());
    /// ```
    pub fn file_names(&self) -> Vec<&str> {
        self.files.keys().map(|s| s.as_str()).collect()
    }

    /// Get a reference to a loaded [`RepoFile`] by filename.
    pub fn get_file(&self, filename: &str) -> Option<&RepoFile> {
        self.files.get(filename)
    }

    /// Get a mutable reference to a loaded [`RepoFile`] by filename.
    pub fn get_file_mut(&mut self, filename: &str) -> Option<&mut RepoFile> {
        self.files.get_mut(filename)
    }

    /// Insert or replace a [`RepoFile`] by filename.
    ///
    /// If a file with this name was previously loaded, it is replaced.
    /// The filename is not validated against the filesystem.
    pub fn set_file(&mut self, filename: &str, file: RepoFile) {
        self.files.insert(filename.to_string(), file);
    }

    /// Remove a tracked file and its on-disk counterpart.
    ///
    /// If the file exists on disk, it is deleted. The [`RepoFile`] is
    /// returned if it was tracked.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the on-disk file exists but cannot be removed.
    pub fn remove_file(
        &mut self,
        filename: &str,
    ) -> std::result::Result<Option<RepoFile>, std::io::Error> {
        let removed = self.files.shift_remove(filename);
        let fp = self.path.join(filename);
        if fp.exists() {
            fs::remove_file(fp)?;
        }
        Ok(removed)
    }

    /// Create a new empty [`RepoFile`] for the given filename, or return
    /// the existing one if already tracked.
    ///
    /// The file is not written to disk until [`save`](ReposDir::save) or
    /// [`save_all`](ReposDir::save_all) is called.
    pub fn create_file(&mut self, filename: &str) -> &mut RepoFile {
        self.files.entry(filename.to_string()).or_default()
    }

    /// Find a repo by ID across all loaded files.
    ///
    /// Returns `Some((filename, repo))` if the repo is found, or `None` if
    /// no file contains a repo with the given ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use dnf_repofile::{ReposDir, RepoId};
    ///
    /// let rd = ReposDir::load("/tmp/nonexistent").unwrap();
    /// assert!(rd.find_repo(&RepoId::try_new("epel").unwrap()).is_none());
    /// ```
    pub fn find_repo(&self, id: &RepoId) -> Option<(&str, &Repo)> {
        for (name, rf) in &self.files {
            if let Some(block) = rf.get(id) {
                return Some((name.as_str(), &block.data));
            }
        }
        None
    }

    /// Find the filename containing a repo with the given ID.
    ///
    /// Returns `None` if no file contains a repo with this ID.
    pub fn file_for_repo(&self, id: &RepoId) -> Option<&str> {
        for (name, rf) in &self.files {
            if rf.contains(id) {
                return Some(name.as_str());
            }
        }
        None
    }

    /// Collect all repos across all files as `(filename, repo)` pairs.
    pub fn all_repos(&self) -> Vec<(&str, &Repo)> {
        let mut r = Vec::new();
        for (name, rf) in &self.files {
            for (_, block) in rf.iter() {
                r.push((name.as_str(), &block.data));
            }
        }
        r
    }

    /// Count the total number of repos across all loaded files.
    pub fn repo_count(&self) -> usize {
        self.files.values().map(|rf| rf.len()).sum()
    }

    /// Iterate over all repos across all files as `(filename, repo)` pairs.
    pub fn iter_repos(&self) -> impl Iterator<Item = (&str, &Repo)> {
        self.files.iter().flat_map(|(name, rf)| {
            rf.iter()
                .map(move |(_, block)| (name.as_str(), &block.data))
        })
    }

    /// Validate all repos across all files.
    ///
    /// In addition to per-repo validation rules (URL source, GPG consistency),
    /// this checks for **duplicate repo IDs** across different files, which
    /// would cause one file's definition to silently shadow another at runtime.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();
        let mut seen: HashMap<&RepoId, &str> = HashMap::new();

        for (fname, rf) in &self.files {
            for (repo_id, block) in rf.iter() {
                if let Some(existing) = seen.get(repo_id) {
                    report.errors.push(ValidationIssue {
                        level: IssueLevel::Error,
                        location: IssueLocation::File(fname.clone()),
                        field: None,
                        message: format!(
                            "duplicate repo ID '{}' already defined in '{}'",
                            repo_id.as_ref(),
                            existing
                        ),
                    });
                } else {
                    seen.insert(repo_id, fname.as_str());
                }
                let rr = block.data.validate();
                report.errors.extend(rr.errors);
                report.warnings.extend(rr.warnings);
            }
        }

        report
    }
}
