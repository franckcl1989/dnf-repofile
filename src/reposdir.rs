use crate::error::Result;
use crate::repo::Repo;
use crate::repofile::RepoFile;
use crate::types::RepoId;
use crate::validate::{IssueLevel, IssueLocation, ValidationIssue, ValidationReport};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ReposDir {
    path: PathBuf,
    files: IndexMap<String, RepoFile>,
}

impl ReposDir {
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

    pub fn save(&self, filename: &str) -> std::result::Result<(), std::io::Error> {
        match self.files.get(filename) {
            Some(rf) => fs::write(self.path.join(filename), rf.render()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            )),
        }
    }

    pub fn file_names(&self) -> Vec<&str> {
        self.files.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_file(&self, filename: &str) -> Option<&RepoFile> {
        self.files.get(filename)
    }

    pub fn get_file_mut(&mut self, filename: &str) -> Option<&mut RepoFile> {
        self.files.get_mut(filename)
    }

    pub fn set_file(&mut self, filename: &str, file: RepoFile) {
        self.files.insert(filename.to_string(), file);
    }

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

    pub fn create_file(&mut self, filename: &str) -> &mut RepoFile {
        self.files
            .entry(filename.to_string())
            .or_insert_with(RepoFile::new)
    }

    pub fn find_repo(&self, id: &RepoId) -> Option<(&str, &Repo)> {
        for (name, rf) in &self.files {
            if let Some(block) = rf.get(id) {
                return Some((name.as_str(), &block.data));
            }
        }
        None
    }

    pub fn file_for_repo(&self, id: &RepoId) -> Option<&str> {
        for (name, rf) in &self.files {
            if rf.contains(id) {
                return Some(name.as_str());
            }
        }
        None
    }

    pub fn all_repos(&self) -> Vec<(&str, &Repo)> {
        let mut r = Vec::new();
        for (name, rf) in &self.files {
            for (_, block) in rf.iter() {
                r.push((name.as_str(), &block.data));
            }
        }
        r
    }

    pub fn repo_count(&self) -> usize {
        self.files.values().map(|rf| rf.len()).sum()
    }

    pub fn iter_repos(&self) -> impl Iterator<Item = (&str, &Repo)> {
        self.files
            .iter()
            .flat_map(|(name, rf)| rf.iter().map(move |(_, block)| (name.as_str(), &block.data)))
    }

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
