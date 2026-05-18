//! A pure Rust library for parsing, managing, and rendering
//! DNF/YUM `.repo` configuration files.
//!
//! Provides full CRUD at three levels:
//! - **ReposDir** — manage a directory of `.repo` files
//! - **RepoFile** — parse, modify, render a single `.repo` file
//! - **Repo** / **MainConfig** — type-safe access to individual options

pub mod error;
pub mod types;
pub mod repo;
pub mod mainconfig;
pub mod repofile;
pub mod builder;
pub mod validate;
pub mod diff;
pub mod variables;
pub mod reposdir;

// Re-export key types for convenience
pub use error::{Error, Result};
pub use types::*;
pub use repo::Repo;
pub use mainconfig::MainConfig;
pub use repofile::{RepoFile, SectionBlock, RawEntry};
pub use builder::RepoBuilder;
pub use validate::{ValidationReport, ValidationIssue, IssueLevel, IssueLocation};
pub use diff::{FileDiff, RepoDiff, ConfigDiff, diff_files, diff_repos, diff_main};
pub use variables::{expand_variables, detect_variables};
pub use reposdir::ReposDir;
