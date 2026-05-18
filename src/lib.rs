//! A pure Rust library for parsing, managing, validating, diffing,
//! and rendering DNF/YUM `.repo` configuration files.
//!
//! # Three-Level API
//!
//! | Level  | Type                       | Purpose                               |
//! |--------|----------------------------|---------------------------------------|
//! | Macro  | [`ReposDir`]               | Manage `/etc/yum.repos.d/` directory  |
//! | Meso   | [`RepoFile`]               | Parse, modify, render a `.repo` file  |
//! | Micro  | [`Repo`], [`MainConfig`]   | Type-safe access to individual fields |
//!
//! # Quick Start
//!
//! ```
//! use dnf_repofile::{RepoFile, RepoId};
//!
//! let input = "[epel]\nname=EPEL\nbaseurl=https://example.com/\n";
//! let rf = RepoFile::parse(input).unwrap();
//! let block = rf.get(&RepoId::try_new("epel").unwrap()).unwrap();
//! println!("{}", block.data.name.as_ref().unwrap());
//! ```
//!
//! # Features
//!
//! - **Parse** `.repo` files into fully-typed Rust structs
//! - **Render** back to text with comment/whitespace preservation
//! - **Validate** repository configurations
//! - **Diff** between repo files or individual repos
//! - **Builder** pattern for programmatic creation
//! - **Variable expansion** (`$releasever`, `${basearch}`, etc.)

pub mod builder;
pub mod diff;
pub mod error;
pub mod mainconfig;
pub mod repo;
pub mod repofile;
pub mod reposdir;
pub mod types;
pub mod validate;
pub mod variables;

// Re-export key types for convenience
pub use builder::RepoBuilder;
pub use diff::{diff_files, diff_main, diff_repos, ConfigDiff, FileDiff, RepoDiff};
pub use error::{Error, Result};
pub use mainconfig::MainConfig;
pub use repo::Repo;
pub use repofile::{RawEntry, RepoFile, SectionBlock};
pub use reposdir::ReposDir;
pub use types::*;
pub use validate::{IssueLevel, IssueLocation, ValidationIssue, ValidationReport};
pub use variables::{detect_variables, expand_variables};
