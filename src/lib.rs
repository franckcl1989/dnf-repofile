//! A pure Rust library for parsing, managing, and rendering
//! DNF/YUM `.repo` configuration files.
//!
//! Provides full CRUD at three levels:
//! - **ReposDir** — manage a directory of `.repo` files
//! - **RepoFile** — parse, modify, render a single `.repo` file
//! - **Repo** / **MainConfig** — type-safe access to individual options

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
