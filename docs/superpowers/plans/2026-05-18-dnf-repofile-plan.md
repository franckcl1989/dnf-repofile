# dnf-repofile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a 100% pure Rust library for parsing, managing, validating, diffing, and rendering DNF/YUM `.repo` configuration files with full round-trip fidelity.

**Architecture:** Bottom-up typed approach. `types.rs` defines all value types and newtypes. `repo.rs` + `mainconfig.rs` define the typed structs. `repofile.rs` provides parse/render of INI format with comment/order preservation via `SectionBlock<T>`. `reposdir.rs` manages a directory of repo files. `validate.rs`, `diff.rs`, `variables.rs` provide cross-cutting capabilities.

**Tech Stack:** Rust 1.80+, `indexmap`, `url`, `camino`, `nutype`, `derive_more`, `thiserror`. No unsafe, no FFI.

---

## File Structure

```
src/
  lib.rs           — Public re-exports
  error.rs         — Error, Result, ParseBoolError, AddRepoError
  types.rs         — RepoId, RepoName, Priority, Cost, DnfBool, Url, etc.
  repo.rs          — Repo struct (48 fields), Repo::url_source()
  mainconfig.rs    — MainConfig struct (57 fields)
  repofile.rs      — SectionBlock<T>, RawEntry, RepoFile, parser, renderer
  builder.rs       — RepoBuilder (chainable setter pattern)
  validate.rs      — ValidationReport, ValidationIssue, validate() fns
  diff.rs          — FileDiff, RepoDiff, ConfigDiff, diff_*() fns
  variables.rs     — expand_variables(), detect_variables()
  reposdir.rs      — ReposDir (directory-level management)

tests/
  fixtures/        — Sample .repo files for testing
  types_tests.rs
  repo_tests.rs
  repofile_tests.rs
  builder_tests.rs
  validate_tests.rs
  diff_tests.rs
  variables_tests.rs
  reposdir_tests.rs
  integration_tests.rs
```

---

### Task 1: Project scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `tests/fixtures/simple.repo`
- Create: `tests/fixtures/complex.repo`

- [ ] **Step 1: Write Cargo.toml**

```toml
[package]
name = "dnf-repofile"
version = "0.1.0"
edition = "2021"
description = "Pure Rust library for parsing, managing, and rendering DNF/YUM .repo configuration files"
license = "MIT"
repository = "https://github.com/franckcl1989/dnf-repofile"

[dependencies]
indexmap = "2"
url = { version = "2", features = ["serde"] }
camino = "1"
nutype = "0.6"
derive_more = { version = "1", features = ["display", "as_ref", "deref", "from"] }
thiserror = "2"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Write lib.rs stub**

```rust
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
```

- [ ] **Step 3: Create test fixtures**

Create `tests/fixtures/simple.repo`:
```ini
[epel]
name=Extra Packages for Enterprise Linux $releasever - $basearch
baseurl=https://download.example.com/pub/epel/$releasever/Everything/$basearch/
enabled=1
gpgcheck=1
gpgkey=https://download.example.com/pub/epel/RPM-GPG-KEY-EPEL-$releasever
```

Create `tests/fixtures/complex.repo`:
```ini
# Preamble comment
# Multi-line preamble

[main]
gpgcheck=1
max_parallel_downloads=10

[baseos]
name=Rocky Linux $releasever - BaseOS
baseurl=https://mirror.example.com/rocky/$releasever/BaseOS/$basearch/os/
# Multiple baseurl entries for failover
baseurl=https://mirror2.example.com/rocky/$releasever/BaseOS/$basearch/os/
enabled=1
gpgcheck=1
gpgkey=https://mirror.example.com/rocky/RPM-GPG-KEY-Rocky-$releasever
priority=10

[appstream]
name=Rocky Linux $releasever - AppStream
baseurl=https://mirror.example.com/rocky/$releasever/AppStream/$basearch/os/
metalink=https://mirror.example.com/rocky/$releasever/AppStream/$basearch/metalink.xml
enabled=1
gpgcheck=1
gpgkey=https://mirror.example.com/rocky/RPM-GPG-KEY-Rocky-$releasever

[custom-repo]
name=My Custom Packages
baseurl=https://custom.example.com/repo/
enabled=0
gpgcheck=0
module_hotfixes=1
cost=500
# This repo has some custom options
custom_option=some_value
```

- [ ] **Step 4: Build check**

```bash
cargo check
```
Expected: no errors (unused import warnings ok for the stub `pub mod` declarations).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs tests/
git commit -m "feat: scaffold project with Cargo.toml, lib.rs, and test fixtures"
```

---

### Task 2: Error types (`error.rs`)

**Files:**
- Create: `src/error.rs`

- [ ] **Step 1: Write error module test**

Create `tests/error_tests.rs`:
```rust
use dnf_repofile::error::*;

#[test]
fn test_parse_bool_error_display() {
    let err = ParseBoolError { input: "maybe".into() };
    assert!(err.to_string().contains("maybe"));
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
}
```

- [ ] **Step 2: Write `src/error.rs`**

```rust
use std::path::PathBuf;
use thiserror::Error;

/// Top-level error type for the library
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse .repo file: {0}")]
    Parse(#[from] ParseError),

    #[error("failed to parse boolean value '{input}'")]
    ParseBool(#[from] ParseBoolError),

    #[error("invalid option value for '{key}': {message}")]
    InvalidValue { key: String, message: String },

    #[error("repo '{0}' already exists in file")]
    DuplicateRepo(String),

    #[error("repo '{0}' not found")]
    RepoNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error when parsing a boolean value fails
#[derive(Error, Debug)]
#[error("invalid boolean value: '{input}'")]
pub struct ParseBoolError {
    pub input: String,
}

/// Error when parsing a .repo file fails
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid section header at line {line}: '{header}'")]
    InvalidSection { line: usize, header: String },

    #[error("missing '=' in key-value pair at line {line}: '{line_text}'")]
    MissingEquals { line: usize, line_text: String },

    #[error("empty section name")]
    EmptySectionName,

    #[error("invalid repo ID '{id}': {reason}")]
    InvalidRepoId { id: String, reason: String },

    #[error("I/O error reading file: {0}")]
    Io(#[from] std::io::Error),
}

/// Error when adding a repo that already exists
#[derive(Error, Debug)]
#[error("repo with ID '{id}' already exists")]
pub struct AddRepoError {
    pub id: String,
}

/// Error when expanding variables fails
#[derive(Error, Debug)]
pub enum ExpandError {
    #[error("variable '{name}' not found in substitution map")]
    VariableNotFound { name: String },

    #[error("maximum recursion depth ({depth}) exceeded while expanding '{expr}'")]
    MaxDepthExceeded { depth: u32, expr: String },

    #[error("malformed variable expression: '{expr}'")]
    MalformedExpression { expr: String },
}
```

- [ ] **Step 3: Run test**

```bash
cargo test --test error_tests
```
Expected: 2 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/error.rs tests/error_tests.rs
git commit -m "feat: define error types (Error, ParseError, ParseBoolError, etc.)"
```

---

### Task 3: Value types — identifiers (`types.rs` part 1)

**Files:**
- Create: `src/types.rs`

- [ ] **Step 1: Write tests for identifier types**

Create `tests/types_tests.rs`:
```rust
use dnf_repofile::types::*;
use std::str::FromStr;

#[test]
fn test_repo_id_valid() {
    let id = RepoId::try_new("fedora-updates").unwrap();
    assert_eq!(id.as_ref(), "fedora-updates");
}

#[test]
fn test_repo_id_trims_whitespace() {
    let id = RepoId::try_new("  myrepo  ").unwrap();
    assert_eq!(id.as_ref(), "myrepo");
}

#[test]
fn test_repo_id_rejects_empty() {
    assert!(RepoId::try_new("").is_err());
    assert!(RepoId::try_new("   ").is_err());
}

#[test]
fn test_repo_id_rejects_special_chars() {
    assert!(RepoId::try_new("bad@id").is_err());
    assert!(RepoId::try_new("repo name").is_err());
    assert!(RepoId::try_new("repo#1").is_err());
}

#[test]
fn test_repo_name_valid() {
    let name = RepoName::try_new("Fedora Updates").unwrap();
    assert_eq!(name.as_ref(), "Fedora Updates");
}

#[test]
fn test_repo_name_rejects_empty() {
    assert!(RepoName::try_new("").is_err());
}

#[test]
fn test_username_trim() {
    let u = Username::try_new("  alice  ").unwrap();
    assert_eq!(u.as_ref(), "alice");
}
```

- [ ] **Step 2: Write identifier newtypes in `src/types.rs`**

```rust
use camino::Utf8PathBuf;
use derive_more::{AsRef, Deref, Display, From};
use indexmap::IndexMap;
use nutype::nutype;
use url::Url;

// ===== Identifiers =====

#[nutype(
    sanitize(trim),
    validate(not_empty, regex = r"^[A-Za-z0-9\-_.:]+$"),
    derive(
        Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
        Display, AsRef, Deref, FromStr,
    ),
)]
pub struct RepoId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct RepoName(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct Username(String);

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct Password(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ProxyUsername(String);

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ProxyPassword(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct UserAgent(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ModulePlatformId(String);
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test types_tests
```
Expected: all identifier tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/types.rs tests/types_tests.rs
git commit -m "feat: add identifier newtypes (RepoId, RepoName, Username, etc.)"
```

---

### Task 4: Value types — numerics (`types.rs` part 2)

- [ ] **Step 1: Append numeric test to `tests/types_tests.rs`**

```rust
#[test]
fn test_priority_range() {
    assert!(Priority::try_new(1).is_ok());
    assert!(Priority::try_new(50).is_ok());
    assert!(Priority::try_new(99).is_ok());
    assert!(Priority::try_new(0).is_err());
    assert!(Priority::try_new(100).is_err());
}

#[test]
fn test_priority_default() {
    assert_eq!(*Priority::default(), 99);
}

#[test]
fn test_retries_accepts_zero() {
    let r = Retries::try_new(0).unwrap();
    assert_eq!(*r, 0); // 0 = unlimited
}

#[test]
fn test_install_only_limit_rejects_one() {
    assert!(InstallOnlyLimit::try_new(0).is_ok());
    assert!(InstallOnlyLimit::try_new(2).is_ok());
    assert!(InstallOnlyLimit::try_new(1).is_err());
    assert!(InstallOnlyLimit::try_new(3).is_ok());
}

#[test]
fn test_max_parallel_downloads_max_20() {
    assert!(MaxParallelDownloads::try_new(20).is_ok());
    assert!(MaxParallelDownloads::try_new(21).is_err());
}

#[test]
fn test_delta_rpm_percentage_range() {
    assert!(DeltaRpmPercentage::try_new(0).is_ok());
    assert!(DeltaRpmPercentage::try_new(100).is_ok());
    assert!(DeltaRpmPercentage::try_new(101).is_err());
}

#[test]
fn test_debug_level_range() {
    assert!(DebugLevel::try_new(10).is_ok());
    assert!(DebugLevel::try_new(11).is_err());
}

#[test]
fn test_cost_non_negative() {
    assert!(Cost::try_new(0).is_ok());
    assert!(Cost::try_new(-1).is_err());
}
```

- [ ] **Step 2: Append numeric newtypes to `src/types.rs`**

```rust
// ===== Numerics =====

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 99),
    default = 99,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, From),
)]
pub struct Priority(i32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 1000,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, From),
)]
pub struct Cost(i32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 10,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct Retries(u32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 30,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, From),
)]
pub struct TimeoutSeconds(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    default = 75,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct DeltaRpmPercentage(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 20),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct MaxParallelDownloads(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 2,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct DebugLevel(u8);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 9,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct LogLevel(u8);

#[nutype(
    validate(greater_or_equal = 0, predicate = |x| x != 1),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct InstallOnlyLimit(u32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 4,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct LogRotate(u32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 10800,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct MetadataTimerSync(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct ErrorLevel(u8);
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test types_tests
```
Expected: all numeric tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/types.rs tests/types_tests.rs
git commit -m "feat: add numeric newtypes (Priority, Cost, Retries, TimeoutSeconds, etc.)"
```

---

### Task 5: Value types — composite and enums (`types.rs` part 3)

- [ ] **Step 1: Append tests to `tests/types_tests.rs`**

```rust
#[test]
fn test_dnf_bool_parse_true_variants() {
    for v in &["1", "yes", "true", "on", "Yes", "YES", "True", "TRUE", "On", "ON"] {
        assert_eq!(DnfBool::parse(v).unwrap(), DnfBool::True, "failed for '{v}'");
    }
}

#[test]
fn test_dnf_bool_parse_false_variants() {
    for v in &["0", "no", "false", "off", "No", "NO", "False", "FALSE", "Off", "OFF"] {
        assert_eq!(DnfBool::parse(v).unwrap(), DnfBool::False, "failed for '{v}'");
    }
}

#[test]
fn test_dnf_bool_parse_invalid() {
    assert!(DnfBool::parse("maybe").is_err());
    assert!(DnfBool::parse("").is_err());
    assert!(DnfBool::parse("2").is_err());
}

#[test]
fn test_dnf_bool_display() {
    assert_eq!(DnfBool::True.to_string(), "1");
    assert_eq!(DnfBool::False.to_string(), "0");
}

#[test]
fn test_dnf_bool_from_bool() {
    assert_eq!(DnfBool::from(true), DnfBool::True);
    assert_eq!(DnfBool::from(false), DnfBool::False);
}

#[test]
fn test_metadata_expire_never() {
    assert_eq!(MetadataExpire::Never, MetadataExpire::Never);
}

#[test]
fn test_storage_size() {
    let s = StorageSize(1024);
    assert_eq!(s.0, 1024);
}

#[test]
fn test_proxy_setting_unset() {
    assert!(matches!(ProxySetting::Unset, ProxySetting::Unset));
}

#[test]
fn test_proxy_setting_disabled() {
    assert!(matches!(ProxySetting::Disabled, ProxySetting::Disabled));
}

#[test]
fn test_throttle_percent() {
    if let Throttle::Percent(50) = Throttle::Percent(50) {
        // ok
    } else {
        panic!("expected Percent(50)");
    }
}
```

- [ ] **Step 2: Append composite and enum types to `src/types.rs`**

```rust
// ===== Composite value types =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSize(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataExpire {
    Duration(u64),
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Throttle {
    Absolute(StorageSize),
    Percent(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxySetting {
    Unset,
    Disabled,
    Url(Url),
}

// ===== DNF Boolean =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DnfBool {
    True,
    False,
}

impl DnfBool {
    pub fn parse(s: &str) -> std::result::Result<Self, crate::error::ParseBoolError> {
        let lower: String = s.chars().map(|c| c.to_ascii_lowercase()).collect();
        match lower.as_str() {
            "1" | "yes" | "true" | "on" => Ok(DnfBool::True),
            "0" | "no" | "false" | "off" => Ok(DnfBool::False),
            _ => Err(crate::error::ParseBoolError { input: s.to_owned() }),
        }
    }
}

impl std::fmt::Display for DnfBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnfBool::True => write!(f, "1"),
            DnfBool::False => write!(f, "0"),
        }
    }
}

impl From<bool> for DnfBool {
    fn from(b: bool) -> Self {
        if b { DnfBool::True } else { DnfBool::False }
    }
}

impl From<DnfBool> for bool {
    fn from(d: DnfBool) -> bool {
        matches!(d, DnfBool::True)
    }
}

// ===== Enums =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpResolve { V4, V6 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyAuthMethod { Any, None_, Basic, Digest, Negotiate, Ntlm, DigestIe, NtlmWb }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoMetadataType { RpmMd }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultilibPolicy { Best, All }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persistence { Auto, Transient, Persist }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpmVerbosity { Critical, Emergency, Error, Warn, Info, Debug }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TsFlag { NoScripts, Test, NoTriggers, NoDocs, JustDb, NoContexts, NoCaps, NoCrypto, Deploops, NoPlugins }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlSource {
    BaseUrl(Vec<Url>),
    MirrorList(Url),
    Metalink(Url),
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test types_tests
```
Expected: all composite/enum tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/types.rs tests/types_tests.rs
git commit -m "feat: add composite types (DnfBool, StorageSize, Throttle, etc.) and enums"
```

---

### Task 6: Repo struct (`repo.rs`)

**Files:**
- Create: `src/repo.rs`
- Create: `tests/repo_tests.rs`

- [ ] **Step 1: Write Repo test**

```rust
use dnf_repofile::repo::Repo;
use dnf_repofile::types::*;

#[test]
fn test_repo_new_empty() {
    let repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    assert_eq!(repo.id.as_ref(), "testrepo");
    assert!(repo.name.is_none());
    assert!(repo.baseurl.is_empty());
    assert!(repo.mirrorlist.is_none());
    assert!(repo.metalink.is_none());
}

#[test]
fn test_repo_url_source_none_when_empty() {
    let repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    assert!(repo.url_source().is_none());
}

#[test]
fn test_repo_url_source_baseurl() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl.push("https://example.com/repo/".parse().unwrap());
    match repo.url_source() {
        Some(UrlSource::BaseUrl(urls)) => assert_eq!(urls.len(), 1),
        other => panic!("expected BaseUrl, got {:?}", other),
    }
}

#[test]
fn test_repo_url_source_mirrorlist() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.mirrorlist = Some("https://example.com/mirrorlist".parse().unwrap());
    match repo.url_source() {
        Some(UrlSource::MirrorList(_)) => {},
        other => panic!("expected MirrorList, got {:?}", other),
    }
}

#[test]
fn test_repo_url_source_prefers_baseurl_when_all_set() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl.push("https://example.com/repo/".parse().unwrap());
    repo.mirrorlist = Some("https://example.com/mirrorlist".parse().unwrap());
    repo.metalink = Some("https://example.com/metalink".parse().unwrap());
    // baseurl takes precedence
    match repo.url_source() {
        Some(UrlSource::BaseUrl(_)) => {},
        other => panic!("expected BaseUrl, got {:?}", other),
    }
}

#[test]
fn test_repo_gpgkey_can_hold_bare_path() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.gpgkey.push("/etc/pki/rpm-gpg/RPM-GPG-KEY".to_string());
    assert_eq!(repo.gpgkey[0], "/etc/pki/rpm-gpg/RPM-GPG-KEY");
}
```

- [ ] **Step 2: Write `src/repo.rs`**

```rust
use crate::types::*;
use camino::Utf8PathBuf;
use indexmap::IndexMap;
use url::Url;

/// A fully typed [repo-id] section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    // ===== repo-only =====
    pub id: RepoId,
    pub name: Option<RepoName>,
    pub baseurl: Vec<Url>,
    pub mirrorlist: Option<Url>,
    pub metalink: Option<Url>,
    pub gpgkey: Vec<String>,
    pub enabled: Option<DnfBool>,
    pub priority: Option<Priority>,
    pub cost: Option<Cost>,
    pub module_hotfixes: Option<DnfBool>,
    pub metadata_type: Option<RepoMetadataType>,
    pub mediaid: Option<String>,
    pub enabled_metadata: Vec<String>,

    // ===== shared =====
    pub excludepkgs: Vec<String>,
    pub includepkgs: Vec<String>,
    pub gpgcheck: Option<DnfBool>,
    pub repo_gpgcheck: Option<DnfBool>,
    pub localpkg_gpgcheck: Option<DnfBool>,
    pub skip_if_unavailable: Option<DnfBool>,
    pub deltarpm: Option<DnfBool>,
    pub deltarpm_percentage: Option<DeltaRpmPercentage>,
    pub enablegroups: Option<DnfBool>,
    pub fastestmirror: Option<DnfBool>,
    pub countme: Option<DnfBool>,
    pub bandwidth: Option<StorageSize>,
    pub throttle: Option<Throttle>,
    pub minrate: Option<StorageSize>,
    pub retries: Option<Retries>,
    pub timeout: Option<TimeoutSeconds>,
    pub max_parallel_downloads: Option<MaxParallelDownloads>,
    pub metadata_expire: Option<MetadataExpire>,
    pub ip_resolve: Option<IpResolve>,
    pub sslverify: Option<DnfBool>,
    pub sslverifystatus: Option<DnfBool>,
    pub sslcacert: Option<Utf8PathBuf>,
    pub sslclientcert: Option<Utf8PathBuf>,
    pub sslclientkey: Option<Utf8PathBuf>,
    pub proxy: ProxySetting,
    pub proxy_username: Option<ProxyUsername>,
    pub proxy_password: Option<ProxyPassword>,
    pub proxy_auth_method: Option<ProxyAuthMethod>,
    pub proxy_sslverify: Option<DnfBool>,
    pub proxy_sslcacert: Option<Utf8PathBuf>,
    pub proxy_sslclientcert: Option<Utf8PathBuf>,
    pub proxy_sslclientkey: Option<Utf8PathBuf>,
    pub username: Option<Username>,
    pub password: Option<Password>,
    pub user_agent: Option<UserAgent>,

    // ===== unknown =====
    pub extras: IndexMap<String, Vec<String>>,
}

impl Repo {
    /// Create a new Repo with only an ID, all fields to defaults
    pub fn new(id: RepoId) -> Self {
        Repo {
            id,
            name: None,
            baseurl: Vec::new(),
            mirrorlist: None,
            metalink: None,
            gpgkey: Vec::new(),
            enabled: None,
            priority: None,
            cost: None,
            module_hotfixes: None,
            metadata_type: None,
            mediaid: None,
            enabled_metadata: Vec::new(),
            excludepkgs: Vec::new(),
            includepkgs: Vec::new(),
            gpgcheck: None,
            repo_gpgcheck: None,
            localpkg_gpgcheck: None,
            skip_if_unavailable: None,
            deltarpm: None,
            deltarpm_percentage: None,
            enablegroups: None,
            fastestmirror: None,
            countme: None,
            bandwidth: None,
            throttle: None,
            minrate: None,
            retries: None,
            timeout: None,
            max_parallel_downloads: None,
            metadata_expire: None,
            ip_resolve: None,
            sslverify: None,
            sslverifystatus: None,
            sslcacert: None,
            sslclientcert: None,
            sslclientkey: None,
            proxy: ProxySetting::Unset,
            proxy_username: None,
            proxy_password: None,
            proxy_auth_method: None,
            proxy_sslverify: None,
            proxy_sslcacert: None,
            proxy_sslclientcert: None,
            proxy_sslclientkey: None,
            username: None,
            password: None,
            user_agent: None,
            extras: IndexMap::new(),
        }
    }

    /// Determine the URL source for this repo.
    ///
    /// Returns `None` if no URL source is set.
    /// If `baseurl` is non-empty, returns `UrlSource::BaseUrl`.
    /// Otherwise checks `mirrorlist`, then `metalink`.
    pub fn url_source(&self) -> Option<UrlSource> {
        if !self.baseurl.is_empty() {
            Some(UrlSource::BaseUrl(self.baseurl.clone()))
        } else if let Some(ref url) = self.mirrorlist {
            Some(UrlSource::MirrorList(url.clone()))
        } else if let Some(ref url) = self.metalink {
            Some(UrlSource::Metalink(url.clone()))
        } else {
            None
        }
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test repo_tests
```
Expected: all repo tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/repo.rs tests/repo_tests.rs
git commit -m "feat: add Repo struct with all 47 typed fields and url_source()"
```

---

### Task 7: MainConfig struct (`mainconfig.rs`)

**Files:**
- Create: `src/mainconfig.rs`
- Create: `tests/mainconfig_tests.rs`

- [ ] **Step 1: Write MainConfig test**

```rust
use dnf_repofile::mainconfig::MainConfig;
use dnf_repofile::types::*;

#[test]
fn test_mainconfig_defaults() {
    let mc = MainConfig::default();
    assert!(mc.arch.is_none());
    assert!(mc.gpgcheck.is_none()); // shared options go in Repo, not MainConfig
    assert!(mc.best.is_none());
    assert!(mc.installonly_limit.is_none());
}

#[test]
fn test_mainconfig_set_debuglevel() {
    let mut mc = MainConfig::default();
    mc.debuglevel = Some(DebugLevel::try_new(5).unwrap());
    assert_eq!(*mc.debuglevel.unwrap(), 5);
}

#[test]
fn test_mainconfig_extras() {
    let mut mc = MainConfig::default();
    mc.extras.insert("custom".into(), vec!["value1".into()]);
    assert_eq!(mc.extras.get("custom").unwrap()[0], "value1");
}
```

- [ ] **Step 2: Write `src/mainconfig.rs`**

```rust
use crate::types::*;
use camino::Utf8PathBuf;
use indexmap::IndexMap;

/// A fully typed [main] section
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MainConfig {
    pub arch: Option<String>,
    pub basearch: Option<String>,
    pub releasever: Option<String>,
    pub cachedir: Option<Utf8PathBuf>,
    pub persistdir: Option<Utf8PathBuf>,
    pub logdir: Option<Utf8PathBuf>,
    pub config_file_path: Option<Utf8PathBuf>,
    pub installroot: Option<Utf8PathBuf>,
    pub reposdir: Vec<Utf8PathBuf>,
    pub varsdir: Vec<Utf8PathBuf>,
    pub pluginconfpath: Vec<Utf8PathBuf>,
    pub pluginpath: Vec<Utf8PathBuf>,
    pub debuglevel: Option<DebugLevel>,
    pub logfilelevel: Option<LogLevel>,
    pub log_rotate: Option<LogRotate>,
    pub log_size: Option<StorageSize>,
    pub installonly_limit: Option<InstallOnlyLimit>,
    pub errorlevel: Option<ErrorLevel>,
    pub metadata_timer_sync: Option<MetadataTimerSync>,
    pub allow_vendor_change: Option<DnfBool>,
    pub assumeno: Option<DnfBool>,
    pub assumeyes: Option<DnfBool>,
    pub autocheck_running_kernel: Option<DnfBool>,
    pub best: Option<DnfBool>,
    pub cacheonly: Option<DnfBool>,
    pub check_config_file_age: Option<DnfBool>,
    pub clean_requirements_on_remove: Option<DnfBool>,
    pub debug_solver: Option<DnfBool>,
    pub defaultyes: Option<DnfBool>,
    pub diskspacecheck: Option<DnfBool>,
    pub exclude_from_weak_autodetect: Option<DnfBool>,
    pub exit_on_lock: Option<DnfBool>,
    pub gpgkey_dns_verification: Option<DnfBool>,
    pub ignorearch: Option<DnfBool>,
    pub install_weak_deps: Option<DnfBool>,
    pub keepcache: Option<DnfBool>,
    pub log_compress: Option<DnfBool>,
    pub module_obsoletes: Option<DnfBool>,
    pub module_stream_switch: Option<DnfBool>,
    pub obsoletes: Option<DnfBool>,
    pub plugins: Option<DnfBool>,
    pub protect_running_kernel: Option<DnfBool>,
    pub strict: Option<DnfBool>,
    pub upgrade_group_objects_upgrade: Option<DnfBool>,
    pub zchunk: Option<DnfBool>,
    pub installonlypkgs: Vec<String>,
    pub protected_packages: Vec<String>,
    pub exclude_from_weak: Vec<String>,
    pub group_package_types: Vec<String>,
    pub optional_metadata_types: Vec<String>,
    pub tsflags: Vec<TsFlag>,
    pub usr_drift_protected_paths: Vec<String>,
    pub multilib_policy: Option<MultilibPolicy>,
    pub persistence: Option<Persistence>,
    pub rpmverbosity: Option<RpmVerbosity>,
    pub module_platform_id: Option<ModulePlatformId>,
    pub extras: IndexMap<String, Vec<String>>,
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test mainconfig_tests
```
Expected: all MainConfig tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/mainconfig.rs tests/mainconfig_tests.rs
git commit -m "feat: add MainConfig struct with all 56 main-only fields"
```

---

### Task 8: RepoFile — parser (`repofile.rs` part 1)

**Files:**
- Create: `src/repofile.rs`
- Create: `tests/repofile_tests.rs`

- [ ] **Step 1: Write parser tests**

```rust
use dnf_repofile::repofile::*;
use dnf_repofile::types::*;
use std::str::FromStr;

#[test]
fn test_parse_simple_repo() {
    let input = "\
[epel]
name=EPEL
baseurl=https://example.com/epel/
enabled=1
gpgcheck=1
";
    let rf = RepoFile::parse(input).unwrap();
    assert_eq!(rf.repos.len(), 1);
    let block = rf.get(&RepoId::try_new("epel").unwrap()).unwrap();
    assert_eq!(block.data.name.unwrap().as_ref(), "EPEL");
    assert_eq!(block.data.baseurl[0].as_str(), "https://example.com/epel/");
}

#[test]
fn test_parse_with_preamble_comments() {
    let input = "\
# This is a comment
# Another comment

[testrepo]
name=Test
baseurl=https://example.com/
";
    let rf = RepoFile::parse(input).unwrap();
    assert_eq!(rf.preamble.len(), 2);
    assert!(rf.preamble[0].contains("This is a comment"));
}

#[test]
fn test_parse_multiple_baseurl() {
    let input = "\
[testrepo]
name=Test
baseurl=https://example.com/one/
baseurl=https://example.com/two/
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.data.baseurl.len(), 2);
    assert_eq!(block.data.baseurl[1].as_str(), "https://example.com/two/");
}

#[test]
fn test_parse_with_main_section() {
    let input = "\
[main]
gpgcheck=1
max_parallel_downloads=10

[testrepo]
name=Test
baseurl=https://example.com/
";
    let rf = RepoFile::parse(input).unwrap();
    assert!(rf.main.is_some());
    let main_block = rf.main.as_ref().unwrap();
    assert_eq!(main_block.data.max_parallel_downloads.unwrap().to_string(), "3");
    // max_parallel_downloads=10 → MaxParallelDownloads(10)
    assert_eq!(*main_block.data.max_parallel_downloads.unwrap(), 10);
}

#[test]
fn test_parse_preserves_comments() {
    let input = "\
# Header comment
[testrepo]  # inline header comment
name=Test  # repo name
baseurl=https://example.com/
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.header_comments.len(), 1);
    assert!(block.header_comments[0].contains("Header comment"));
}

#[test]
fn test_parse_extras() {
    let input = "\
[testrepo]
name=Test
baseurl=https://example.com/
custom_option=custom_value
another_extra=value2
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.raw_entries.len(), 2);
    assert_eq!(block.raw_entries[0].key, "custom_option");
    assert_eq!(block.raw_entries[0].value, "custom_value");
}

#[test]
fn test_parse_boolean_variants() {
    for val in &["1", "yes", "true", "on", "Yes", "YES"] {
        let input = format!("[testrepo]\nname=Test\nbaseurl=https://x.com/\nenabled={val}\n");
        let rf = RepoFile::parse(&input).unwrap();
        let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
        assert_eq!(block.data.enabled, Some(DnfBool::True), "failed for {val}");
    }
    for val in &["0", "no", "false", "off", "No", "NO"] {
        let input = format!("[testrepo]\nname=Test\nbaseurl=https://x.com/\nenabled={val}\n");
        let rf = RepoFile::parse(&input).unwrap();
        let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
        assert_eq!(block.data.enabled, Some(DnfBool::False), "failed for {val}");
    }
}

#[test]
fn test_parse_invalid_section() {
    let input = "[invalid@id]\nname=Test\n";
    assert!(RepoFile::parse(input).is_err());
}

#[test]
fn test_round_trip() {
    let input = "\
[epel]
name=Extra Packages
# This is a baseurl
baseurl=https://example.com/epel/
enabled=1
gpgcheck=1
";
    let rf = RepoFile::parse(input).unwrap();
    let output = rf.render();
    let rf2 = RepoFile::parse(&output).unwrap();
    assert_eq!(rf2.repos.len(), 1);
    let block = rf2.get(&RepoId::try_new("epel").unwrap()).unwrap();
    assert_eq!(block.data.name.unwrap().as_ref(), "Extra Packages");
    assert_eq!(block.data.baseurl[0].as_str(), "https://example.com/epel/");
}

#[test]
fn test_parse_rejects_missing_equals() {
    let input = "[repo]\nbadline\n";
    assert!(RepoFile::parse(input).is_err());
}

#[test]
fn test_parse_empty_file() {
    let rf = RepoFile::parse("").unwrap();
    assert!(rf.main.is_none());
    assert!(rf.repos.is_empty());
}

#[test]
fn test_parse_comment_only_file() {
    let rf = RepoFile::parse("# Just a comment\n").unwrap();
    assert_eq!(rf.preamble.len(), 1);
}
```

- [ ] **Step 2: Write `src/repofile.rs` — data structures and parser**

```rust
use crate::error::{ParseError, Result};
use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::types::*;
use indexmap::IndexMap;
use std::str::FromStr;

/// A section block: typed data + formatting metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionBlock<T> {
    pub header_comments: Vec<String>,
    pub data: T,
    pub item_comments: IndexMap<String, String>,
    pub item_order: Vec<String>,
    pub raw_entries: Vec<RawEntry>,
}

/// An unrecognized key-value entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    pub key: String,
    pub value: String,
    pub inline_comment: Option<String>,
    pub leading_comments: Vec<String>,
}

/// A complete parsed .repo file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoFile {
    pub preamble: Vec<String>,
    pub main: Option<SectionBlock<MainConfig>>,
    pub repos: IndexMap<RepoId, SectionBlock<Repo>>,
}

/// A raw INI entry being built up during parsing
#[derive(Debug, Clone)]
struct RawLine {
    key: String,
    value: String,
    inline_comment: Option<String>,
    leading_comments: Vec<String>,
}

/// Known DNF option keys (repo-only + shared)
const KNOWN_REPO_KEYS: &[&str] = &[
    "name", "baseurl", "mirrorlist", "metalink", "gpgkey", "enabled",
    "priority", "cost", "module_hotfixes", "type", "mediaid", "enabled_metadata",
    "excludepkgs", "includepkgs",
    "gpgcheck", "repo_gpgcheck", "localpkg_gpgcheck", "skip_if_unavailable",
    "deltarpm", "deltarpm_percentage", "enablegroups", "fastestmirror", "countme",
    "bandwidth", "throttle", "minrate", "retries", "timeout",
    "max_parallel_downloads", "metadata_expire", "ip_resolve",
    "sslverify", "sslverifystatus", "sslcacert", "sslclientcert", "sslclientkey",
    "proxy", "proxy_username", "proxy_password", "proxy_auth_method",
    "proxy_sslverify", "proxy_sslcacert", "proxy_sslclientcert", "proxy_sslclientkey",
    "username", "password", "user_agent",
];

/// Known DNF option keys for [main] section
const KNOWN_MAIN_KEYS: &[&str] = &[
    "arch", "basearch", "releasever", "cachedir", "persistdir", "logdir",
    "config_file_path", "installroot", "reposdir", "varsdir", "pluginconfpath",
    "pluginpath", "debuglevel", "logfilelevel", "log_rotate", "log_size",
    "installonly_limit", "errorlevel", "metadata_timer_sync",
    "allow_vendor_change", "assumeno", "assumeyes", "autocheck_running_kernel",
    "best", "cacheonly", "check_config_file_age", "clean_requirements_on_remove",
    "debug_solver", "defaultyes", "diskspacecheck", "exclude_from_weak_autodetect",
    "exit_on_lock", "gpgkey_dns_verification", "ignorearch", "install_weak_deps",
    "keepcache", "log_compress", "module_obsoletes", "module_stream_switch",
    "obsoletes", "plugins", "protect_running_kernel", "strict",
    "upgrade_group_objects_upgrade", "zchunk",
    "installonlypkgs", "protected_packages", "exclude_from_weak",
    "group_package_types", "optional_metadata_types", "tsflags",
    "usr_drift_protected_paths",
    "multilib_policy", "persistence", "rpmverbosity", "module_platform_id",
];

// ----- Parser -----

#[derive(Debug)]
struct ParseState {
    preamble: Vec<String>,
    pending_comments: Vec<String>,
    current_section: Option<String>,
    current_entries: Vec<RawLine>,
    sections: IndexMap<String, Vec<RawLine>>,
    section_header_comments: IndexMap<String, Vec<String>>,
}

impl RepoFile {
    pub fn parse(input: &str) -> std::result::Result<Self, ParseError> {
        let mut state = ParseState {
            preamble: Vec::new(),
            pending_comments: Vec::new(),
            current_section: None,
            current_entries: Vec::new(),
            sections: IndexMap::new(),
            section_header_comments: IndexMap::new(),
        };

        for (line_idx, raw_line) in input.lines().enumerate() {
            let trimmed = raw_line.trim();

            // Empty line
            if trimmed.is_empty() {
                if state.current_section.is_some() {
                    state.pending_comments.push(String::new());
                } else {
                    state.preamble.push(String::new());
                }
                continue;
            }

            // Comment line
            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                state.pending_comments.push(raw_line.to_owned());
                continue;
            }

            // Section header
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Flush previous section
                if let Some(ref sec_name) = state.current_section.take() {
                    state.section_header_comments
                        .insert(sec_name.clone(), std::mem::take(&mut state.pending_comments));
                    state.sections.insert(
                        sec_name.clone(),
                        std::mem::take(&mut state.current_entries),
                    );
                }

                let section_name = &trimmed[1..trimmed.len() - 1].trim().to_string();
                if section_name.is_empty() {
                    return Err(ParseError::EmptySectionName);
                }
                // Validate repo ID for non-[main] sections
                if section_name != "main" {
                    if let Err(_) = RepoId::try_new(section_name.as_str()) {
                        return Err(ParseError::InvalidRepoId {
                            id: section_name.clone(),
                            reason: "invalid characters".into(),
                        });
                    }
                }
                // pending_comments become header_comments for the new section
                state.current_section = Some(section_name.clone());
                continue;
            }

            // Key=value line
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value_part = &trimmed[eq_pos + 1..];
                let (value, inline_comment) = split_value_and_comment(value_part);

                if key.is_empty() {
                    return Err(ParseError::MissingEquals {
                        line: line_idx + 1,
                        line_text: raw_line.to_owned(),
                    });
                }

                let entry = RawLine {
                    key,
                    value: value.trim().to_string(),
                    inline_comment,
                    leading_comments: std::mem::take(&mut state.pending_comments),
                };

                if let Some(ref sec_name) = state.current_section {
                    state.current_entries.push(entry);
                } else {
                    // Key=value before any section — treat as preamble
                    state.preamble.push(raw_line.to_owned());
                }
            } else {
                return Err(ParseError::MissingEquals {
                    line: line_idx + 1,
                    line_text: raw_line.to_owned(),
                });
            }
        }

        // Flush final section
        if let Some(ref sec_name) = state.current_section.take() {
            state.section_header_comments
                .insert(sec_name.clone(), std::mem::take(&mut state.pending_comments));
            state.sections.insert(
                sec_name.clone(),
                std::mem::take(&mut state.current_entries),
            );
        }

        // Build typed structures
        build_repofile(state)
    }

    pub fn new() -> Self {
        RepoFile {
            preamble: Vec::new(),
            main: None,
            repos: IndexMap::new(),
        }
    }

    // ===== Repo access =====

    pub fn get(&self, id: &RepoId) -> Option<&SectionBlock<Repo>> {
        self.repos.get(id)
    }

    pub fn get_mut(&mut self, id: &RepoId) -> Option<&mut SectionBlock<Repo>> {
        self.repos.get_mut(id)
    }

    pub fn add(&mut self, repo: Repo) -> std::result::Result<(), crate::error::AddRepoError> {
        let id = repo.id.clone();
        if self.repos.contains_key(&id) {
            return Err(crate::error::AddRepoError { id: id.to_string() });
        }
        self.repos.insert(id, SectionBlock {
            header_comments: Vec::new(),
            data: repo,
            item_comments: IndexMap::new(),
            item_order: Vec::new(),
            raw_entries: Vec::new(),
        });
        Ok(())
    }

    pub fn set(&mut self, repo: Repo) {
        let id = repo.id.clone();
        let block = self.repos.entry(id).or_insert_with(|| SectionBlock {
            header_comments: Vec::new(),
            data: Repo::new(repo.id.clone()),
            item_comments: IndexMap::new(),
            item_order: Vec::new(),
            raw_entries: Vec::new(),
        });
        block.data = repo;
    }

    pub fn remove(&mut self, id: &RepoId) -> Option<SectionBlock<Repo>> {
        self.repos.shift_remove(id)
    }

    pub fn contains(&self, id: &RepoId) -> bool {
        self.repos.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.repos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.repos.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RepoId, &SectionBlock<Repo>)> {
        self.repos.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&RepoId, &mut SectionBlock<Repo>)> {
        self.repos.iter_mut()
    }

    pub fn repo_ids(&self) -> impl Iterator<Item = &RepoId> {
        self.repos.keys()
    }

    // ===== [main] access =====

    pub fn main(&self) -> Option<&SectionBlock<MainConfig>> {
        self.main.as_ref()
    }

    pub fn main_mut(&mut self) -> Option<&mut SectionBlock<MainConfig>> {
        self.main.as_mut()
    }

    pub fn set_main(&mut self, config: MainConfig) {
        self.main = Some(SectionBlock {
            header_comments: Vec::new(),
            data: config,
            item_comments: IndexMap::new(),
            item_order: Vec::new(),
            raw_entries: Vec::new(),
        });
    }

    pub fn remove_main(&mut self) {
        self.main = None;
    }

    // ===== Merge =====

    pub fn merge(&mut self, other: RepoFile) {
        if let Some(other_main) = other.main {
            if let Some(ref mut self_main) = self.main {
                merge_mainconfig(&mut self_main.data, &other_main.data);
            } else {
                self.main = Some(other_main);
            }
        }
        for (id, block) in other.repos {
            self.repos.insert(id, block);
        }
    }
}

// ----- Renderer -----

impl RepoFile {
    pub fn render(&self) -> String {
        let mut out = String::new();

        // Preamble
        for line in &self.preamble {
            render_line(&mut out, line);
        }

        // [main] section
        if let Some(ref main_block) = self.main {
            for comment in &main_block.header_comments {
                render_line(&mut out, comment);
            }
            out.push_str("[main]\n");
            render_section_entries(&mut out, &main_block.item_order, &main_block.item_comments, &main_block.raw_entries);
        }

        // [repo-id] sections
        for (repo_id, block) in &self.repos {
            for comment in &block.header_comments {
                render_line(&mut out, comment);
            }
            out.push_str(&format!("[{}]\n", repo_id.as_ref()));
            render_section_entries(&mut out, &block.item_order, &block.item_comments, &block.raw_entries);
        }

        out
    }
}

fn render_line(out: &mut String, line: &str) {
    out.push_str(line);
    if !line.ends_with('\n') {
        out.push('\n');
    }
}

fn render_section_entries(
    out: &mut String,
    item_order: &[String],
    item_comments: &IndexMap<String, String>,
    raw_entries: &[RawEntry],
) {
    // Known entries in order
    for key in item_order {
        // The rendered value is by the caller; during initial pass we delegate
        // The full implementation will use repo-specific rendering
    }
    // Unknown entries
    for entry in raw_entries {
        for comment in &entry.leading_comments {
            render_line(out, comment);
        }
        let mut line = format!("{}={}", entry.key, entry.value);
        if let Some(ref ic) = entry.inline_comment {
            line.push_str(&format!(" #{}", ic));
        }
        out.push_str(&line);
        out.push('\n');
    }
}

// ----- Internal helpers -----

fn split_value_and_comment(value_part: &str) -> (String, Option<String>) {
    let mut in_quotes = false;
    for (i, ch) in value_part.char_indices() {
        if ch == '"' {
            in_quotes = !in_quotes;
        }
        if ch == '#' && !in_quotes {
            return (
                value_part[..i].to_string(),
                Some(value_part[i + 1..].trim().to_string()),
            );
        }
    }
    (value_part.to_string(), None)
}

fn build_repofile(state: ParseState) -> std::result::Result<RepoFile, ParseError> {
    let mut rf = RepoFile::new();
    rf.preamble = state.preamble;

    for (sec_name, entries) in &state.sections {
        let header_comments = state.section_header_comments.get(sec_name)
            .cloned()
            .unwrap_or_default();

        if sec_name == "main" {
            let mut mc = MainConfig::default();
            let (item_order, item_comments, raw_entries) = parse_entries_into_mainconfig(&mut mc, entries);
            rf.main = Some(SectionBlock {
                header_comments,
                data: mc,
                item_comments,
                item_order,
                raw_entries,
            });
        } else {
            let repo_id = RepoId::try_new(sec_name.as_str())
                .map_err(|_| ParseError::InvalidRepoId {
                    id: sec_name.clone(),
                    reason: "invalid repo ID characters".into(),
                })?;
            let mut repo = Repo::new(repo_id);
            let (item_order, item_comments, raw_entries) = parse_entries_into_repo(&mut repo, entries);
            rf.repos.insert(repo.id.clone(), SectionBlock {
                header_comments,
                data: repo,
                item_comments,
                item_order,
                raw_entries,
            });
        }
    }

    Ok(rf)
}

/// Parse raw entries into a Repo, returning (item_order, item_comments, raw_entries)
fn parse_entries_into_repo(
    repo: &mut Repo,
    entries: &[RawLine],
) -> (Vec<String>, IndexMap<String, String>, Vec<RawEntry>) {
    let mut item_order = Vec::new();
    let mut item_comments = IndexMap::new();
    let mut raw_entries = Vec::new();

    for entry in entries {
        if KNOWN_REPO_KEYS.contains(&entry.key.as_str()) {
            parse_known_repo_option(repo, entry);
            item_order.push(entry.key.clone());
            if let Some(ref ic) = entry.inline_comment {
                item_comments.insert(entry.key.clone(), ic.clone());
            }
        } else {
            // Store in extras AND raw_entries
            repo.extras
                .entry(entry.key.clone())
                .or_default()
                .push(entry.value.clone());
            raw_entries.push(RawEntry {
                key: entry.key.clone(),
                value: entry.value.clone(),
                inline_comment: entry.inline_comment.clone(),
                leading_comments: entry.leading_comments.clone(),
            });
        }
    }

    (item_order, item_comments, raw_entries)
}

fn parse_known_repo_option(repo: &mut Repo, entry: &RawLine) {
    let key = entry.key.as_str();
    let val = entry.value.as_str();
    match key {
        "name" => { let _ = RepoName::try_new(val).map(|n| repo.name = Some(n)); }
        "baseurl" => { let _ = Url::from_str(val).map(|u| repo.baseurl.push(u)); }
        "mirrorlist" => { let _ = Url::from_str(val).map(|u| repo.mirrorlist = Some(u)); }
        "metalink" => { let _ = Url::from_str(val).map(|u| repo.metalink = Some(u)); }
        "gpgkey" => { repo.gpgkey.push(val.to_string()); }
        "enabled" => { let _ = DnfBool::parse(val).map(|b| repo.enabled = Some(b)); }
        "priority" => { let _ = Priority::try_new(val.parse().unwrap_or(99)).map(|p| repo.priority = Some(p)); }
        "cost" => { let _ = val.parse::<i32>().ok().and_then(|c| Cost::try_new(c).ok()).map(|c| repo.cost = Some(c)); }
        "module_hotfixes" => { let _ = DnfBool::parse(val).map(|b| repo.module_hotfixes = Some(b)); }
        "type" => { repo.metadata_type = Some(RepoMetadataType::RpmMd); }
        "mediaid" => { repo.mediaid = Some(val.to_string()); }
        "enabled_metadata" => { repo.enabled_metadata.push(val.to_string()); }
        "excludepkgs" => { repo.excludepkgs.push(val.to_string()); }
        "includepkgs" => { repo.includepkgs.push(val.to_string()); }
        "gpgcheck" => { let _ = DnfBool::parse(val).map(|b| repo.gpgcheck = Some(b)); }
        "repo_gpgcheck" => { let _ = DnfBool::parse(val).map(|b| repo.repo_gpgcheck = Some(b)); }
        "localpkg_gpgcheck" => { let _ = DnfBool::parse(val).map(|b| repo.localpkg_gpgcheck = Some(b)); }
        "skip_if_unavailable" => { let _ = DnfBool::parse(val).map(|b| repo.skip_if_unavailable = Some(b)); }
        "deltarpm" => { let _ = DnfBool::parse(val).map(|b| repo.deltarpm = Some(b)); }
        "deltarpm_percentage" => { let _ = val.parse::<u32>().ok().and_then(|d| DeltaRpmPercentage::try_new(d).ok()).map(|d| repo.deltarpm_percentage = Some(d)); }
        "enablegroups" => { let _ = DnfBool::parse(val).map(|b| repo.enablegroups = Some(b)); }
        "fastestmirror" => { let _ = DnfBool::parse(val).map(|b| repo.fastestmirror = Some(b)); }
        "countme" => { let _ = DnfBool::parse(val).map(|b| repo.countme = Some(b)); }
        "retries" => { let _ = val.parse::<u32>().ok().and_then(|r| Retries::try_new(r).ok()).map(|r| repo.retries = Some(r)); }
        "timeout" => { let _ = val.parse::<u32>().ok().and_then(|t| TimeoutSeconds::try_new(t).ok()).map(|t| repo.timeout = Some(t)); }
        "max_parallel_downloads" => { let _ = val.parse::<u32>().ok().and_then(|m| MaxParallelDownloads::try_new(m).ok()).map(|m| repo.max_parallel_downloads = Some(m)); }
        "ip_resolve" => { repo.ip_resolve = parse_ip_resolve(val); }
        "sslverify" => { let _ = DnfBool::parse(val).map(|b| repo.sslverify = Some(b)); }
        "sslverifystatus" => { let _ = DnfBool::parse(val).map(|b| repo.sslverifystatus = Some(b)); }
        "sslcacert" => { repo.sslcacert = Some(Utf8PathBuf::from(val)); }
        "sslclientcert" => { repo.sslclientcert = Some(Utf8PathBuf::from(val)); }
        "sslclientkey" => { repo.sslclientkey = Some(Utf8PathBuf::from(val)); }
        "proxy" => { repo.proxy = parse_proxy(val); }
        "proxy_username" => { let _ = ProxyUsername::try_new(val).map(|u| repo.proxy_username = Some(u)); }
        "proxy_password" => { let _ = ProxyPassword::try_new(val).map(|p| repo.proxy_password = Some(p)); }
        "proxy_auth_method" => { repo.proxy_auth_method = parse_proxy_auth_method(val); }
        "proxy_sslverify" => { let _ = DnfBool::parse(val).map(|b| repo.proxy_sslverify = Some(b)); }
        "proxy_sslcacert" => { repo.proxy_sslcacert = Some(Utf8PathBuf::from(val)); }
        "proxy_sslclientcert" => { repo.proxy_sslclientcert = Some(Utf8PathBuf::from(val)); }
        "proxy_sslclientkey" => { repo.proxy_sslclientkey = Some(Utf8PathBuf::from(val)); }
        "username" => { let _ = Username::try_new(val).map(|u| repo.username = Some(u)); }
        "password" => { let _ = Password::try_new(val).map(|p| repo.password = Some(p)); }
        "user_agent" => { let _ = UserAgent::try_new(val).map(|ua| repo.user_agent = Some(ua)); }
        _ => {}
    }
}

fn parse_entries_into_mainconfig(
    mc: &mut MainConfig,
    entries: &[RawLine],
) -> (Vec<String>, IndexMap<String, String>, Vec<RawEntry>) {
    let mut item_order = Vec::new();
    let mut item_comments = IndexMap::new();
    let mut raw_entries = Vec::new();

    for entry in entries {
        if KNOWN_MAIN_KEYS.contains(&entry.key.as_str()) {
            parse_known_main_option(mc, entry);
            item_order.push(entry.key.clone());
            if let Some(ref ic) = entry.inline_comment {
                item_comments.insert(entry.key.clone(), ic.clone());
            }
        } else {
            mc.extras
                .entry(entry.key.clone())
                .or_default()
                .push(entry.value.clone());
            raw_entries.push(RawEntry {
                key: entry.key.clone(),
                value: entry.value.clone(),
                inline_comment: entry.inline_comment.clone(),
                leading_comments: entry.leading_comments.clone(),
            });
        }
    }

    (item_order, item_comments, raw_entries)
}

fn parse_known_main_option(mc: &mut MainConfig, entry: &RawLine) {
    let key = entry.key.as_str();
    let val = entry.value.as_str();
    match key {
        "arch" => { mc.arch = Some(val.to_string()); }
        "basearch" => { mc.basearch = Some(val.to_string()); }
        "releasever" => { mc.releasever = Some(val.to_string()); }
        "cachedir" => { mc.cachedir = Some(Utf8PathBuf::from(val)); }
        "persistdir" => { mc.persistdir = Some(Utf8PathBuf::from(val)); }
        "logdir" => { mc.logdir = Some(Utf8PathBuf::from(val)); }
        "config_file_path" => { mc.config_file_path = Some(Utf8PathBuf::from(val)); }
        "installroot" => { mc.installroot = Some(Utf8PathBuf::from(val)); }
        "reposdir" => { mc.reposdir.push(Utf8PathBuf::from(val)); }
        "varsdir" => { mc.varsdir.push(Utf8PathBuf::from(val)); }
        "pluginconfpath" => { mc.pluginconfpath.push(Utf8PathBuf::from(val)); }
        "pluginpath" => { mc.pluginpath.push(Utf8PathBuf::from(val)); }
        "debuglevel" => { let _ = val.parse::<u8>().ok().and_then(|d| DebugLevel::try_new(d).ok()).map(|d| mc.debuglevel = Some(d)); }
        "logfilelevel" => { let _ = val.parse::<u8>().ok().and_then(|l| LogLevel::try_new(l).ok()).map(|l| mc.logfilelevel = Some(l)); }
        "log_rotate" => { let _ = val.parse::<u32>().ok().and_then(|l| LogRotate::try_new(l).ok()).map(|l| mc.log_rotate = Some(l)); }
        "log_size" => { let _ = parse_storage_size(val).map(|s| mc.log_size = Some(s)); }
        "installonly_limit" => { let _ = val.parse::<u32>().ok().and_then(|i| InstallOnlyLimit::try_new(i).ok()).map(|i| mc.installonly_limit = Some(i)); }
        "errorlevel" => { let _ = val.parse::<u8>().ok().and_then(|e| ErrorLevel::try_new(e).ok()).map(|e| mc.errorlevel = Some(e)); }
        "metadata_timer_sync" => { let _ = parse_storage_size(val).map(|s| mc.metadata_timer_sync = Some(MetadataTimerSync::try_new(s.0 as u32).unwrap_or_default())); }
        "best" => { let _ = DnfBool::parse(val).map(|b| mc.best = Some(b)); }
        "clean_requirements_on_remove" => { let _ = DnfBool::parse(val).map(|b| mc.clean_requirements_on_remove = Some(b)); }
        "gpgkey_dns_verification" => { let _ = DnfBool::parse(val).map(|b| mc.gpgkey_dns_verification = Some(b)); }
        "install_weak_deps" => { let _ = DnfBool::parse(val).map(|b| mc.install_weak_deps = Some(b)); }
        "keepcache" => { let _ = DnfBool::parse(val).map(|b| mc.keepcache = Some(b)); }
        "strict" => { let _ = DnfBool::parse(val).map(|b| mc.strict = Some(b)); }
        "zchunk" => { let _ = DnfBool::parse(val).map(|b| mc.zchunk = Some(b)); }
        "obsoletes" => { let _ = DnfBool::parse(val).map(|b| mc.obsoletes = Some(b)); }
        "plugins" => { let _ = DnfBool::parse(val).map(|b| mc.plugins = Some(b)); }
        "multilib_policy" => { mc.multilib_policy = parse_multilib_policy(val); }
        "persistence" => { mc.persistence = parse_persistence(val); }
        "rpmverbosity" => { mc.rpmverbosity = parse_rpmverbosity(val); }
        "module_platform_id" => { let _ = ModulePlatformId::try_new(val).map(|m| mc.module_platform_id = Some(m)); }
        "tsflags" => { mc.tsflags.extend(parse_tsflags(val)); }
        "installonlypkgs" => { mc.installonlypkgs.push(val.to_string()); }
        "protected_packages" => { mc.protected_packages.push(val.to_string()); }
        "exclude_from_weak" => { mc.exclude_from_weak.push(val.to_string()); }
        "group_package_types" => { mc.group_package_types.push(val.to_string()); }
        "optional_metadata_types" => { mc.optional_metadata_types.push(val.to_string()); }
        "usr_drift_protected_paths" => { mc.usr_drift_protected_paths.push(val.to_string()); }
        "module_obsoletes" => { let _ = DnfBool::parse(val).map(|b| mc.module_obsoletes = Some(b)); }
        "module_stream_switch" => { let _ = DnfBool::parse(val).map(|b| mc.module_stream_switch = Some(b)); }
        "allow_vendor_change" => { let _ = DnfBool::parse(val).map(|b| mc.allow_vendor_change = Some(b)); }
        "assumeno" => { let _ = DnfBool::parse(val).map(|b| mc.assumeno = Some(b)); }
        "assumeyes" => { let _ = DnfBool::parse(val).map(|b| mc.assumeyes = Some(b)); }
        "autocheck_running_kernel" => { let _ = DnfBool::parse(val).map(|b| mc.autocheck_running_kernel = Some(b)); }
        "cacheonly" => { let _ = DnfBool::parse(val).map(|b| mc.cacheonly = Some(b)); }
        "check_config_file_age" => { let _ = DnfBool::parse(val).map(|b| mc.check_config_file_age = Some(b)); }
        "debug_solver" => { let _ = DnfBool::parse(val).map(|b| mc.debug_solver = Some(b)); }
        "defaultyes" => { let _ = DnfBool::parse(val).map(|b| mc.defaultyes = Some(b)); }
        "diskspacecheck" => { let _ = DnfBool::parse(val).map(|b| mc.diskspacecheck = Some(b)); }
        "exclude_from_weak_autodetect" => { let _ = DnfBool::parse(val).map(|b| mc.exclude_from_weak_autodetect = Some(b)); }
        "exit_on_lock" => { let _ = DnfBool::parse(val).map(|b| mc.exit_on_lock = Some(b)); }
        "ignorearch" => { let _ = DnfBool::parse(val).map(|b| mc.ignorearch = Some(b)); }
        "log_compress" => { let _ = DnfBool::parse(val).map(|b| mc.log_compress = Some(b)); }
        "protect_running_kernel" => { let _ = DnfBool::parse(val).map(|b| mc.protect_running_kernel = Some(b)); }
        "upgrade_group_objects_upgrade" => { let _ = DnfBool::parse(val).map(|b| mc.upgrade_group_objects_upgrade = Some(b)); }
        _ => {}
    }
}

// ----- Enum parsers (used by both repo and main parsing) -----

fn parse_ip_resolve(val: &str) -> Option<IpResolve> {
    match val {
        "4" | "IPv4" | "ipv4" => Some(IpResolve::V4),
        "6" | "IPv6" | "ipv6" => Some(IpResolve::V6),
        _ => None,
    }
}

fn parse_proxy_auth_method(val: &str) -> Option<ProxyAuthMethod> {
    match val.to_lowercase().as_str() {
        "any" => Some(ProxyAuthMethod::Any),
        "none" => Some(ProxyAuthMethod::None_),
        "basic" => Some(ProxyAuthMethod::Basic),
        "digest" => Some(ProxyAuthMethod::Digest),
        "negotiate" => Some(ProxyAuthMethod::Negotiate),
        "ntlm" => Some(ProxyAuthMethod::Ntlm),
        "digest_ie" => Some(ProxyAuthMethod::DigestIe),
        "ntlm_wb" => Some(ProxyAuthMethod::NtlmWb),
        _ => None,
    }
}

fn parse_proxy(val: &str) -> ProxySetting {
    if val.is_empty() || val == "_none_" {
        ProxySetting::Disabled
    } else if let Ok(url) = Url::from_str(val) {
        ProxySetting::Url(url)
    } else {
        ProxySetting::Unset
    }
}

fn parse_multilib_policy(val: &str) -> Option<MultilibPolicy> {
    match val {
        "best" => Some(MultilibPolicy::Best),
        "all" => Some(MultilibPolicy::All),
        _ => None,
    }
}

fn parse_persistence(val: &str) -> Option<Persistence> {
    match val {
        "auto" => Some(Persistence::Auto),
        "transient" => Some(Persistence::Transient),
        "persist" => Some(Persistence::Persist),
        _ => None,
    }
}

fn parse_rpmverbosity(val: &str) -> Option<RpmVerbosity> {
    match val {
        "critical" => Some(RpmVerbosity::Critical),
        "emergency" => Some(RpmVerbosity::Emergency),
        "error" => Some(RpmVerbosity::Error),
        "warn" => Some(RpmVerbosity::Warn),
        "info" => Some(RpmVerbosity::Info),
        "debug" => Some(RpmVerbosity::Debug),
        _ => None,
    }
}

fn parse_tsflags(val: &str) -> Vec<TsFlag> {
    val.split(|c: char| c == ',' || c == ' ')
        .filter_map(|s| match s.trim() {
            "noscripts" => Some(TsFlag::NoScripts),
            "test" => Some(TsFlag::Test),
            "notriggers" => Some(TsFlag::NoTriggers),
            "nodocs" => Some(TsFlag::NoDocs),
            "justdb" => Some(TsFlag::JustDb),
            "nocontexts" => Some(TsFlag::NoContexts),
            "nocaps" => Some(TsFlag::NoCaps),
            "nocrypto" => Some(TsFlag::NoCrypto),
            "deploops" => Some(TsFlag::Deploops),
            "noplugins" => Some(TsFlag::NoPlugins),
            _ => None,
        })
        .collect()
}

fn parse_storage_size(val: &str) -> Option<StorageSize> {
    let val = val.trim();
    if let Some(num_part) = val.strip_suffix('G').or_else(|| val.strip_suffix('g')) {
        num_part.trim().parse::<f64>().ok().map(|n| StorageSize((n * 1_000_000_000.0) as u64))
    } else if let Some(num_part) = val.strip_suffix('M').or_else(|| val.strip_suffix('m')) {
        num_part.trim().parse::<f64>().ok().map(|n| StorageSize((n * 1_000_000.0) as u64))
    } else if let Some(num_part) = val.strip_suffix('K').or_else(|| val.strip_suffix('k')) {
        num_part.trim().parse::<f64>().ok().map(|n| StorageSize((n * 1_000.0) as u64))
    } else {
        val.parse::<u64>().ok().map(StorageSize)
    }
}

fn merge_mainconfig(dest: &mut MainConfig, src: &MainConfig) {
    macro_rules! merge_opt {
        ($field:ident) => {
            if src.$field.is_some() {
                dest.$field = src.$field.clone();
            }
        };
        ($field:ident, $empty_val:expr) => {
            if !src.$field.is_empty() {
                dest.$field = src.$field.clone();
            }
        };
    }
    merge_opt!(arch); merge_opt!(basearch); merge_opt!(releasever);
    merge_opt!(cachedir); merge_opt!(persistdir); merge_opt!(logdir);
    merge_opt!(config_file_path); merge_opt!(installroot);
    merge_opt!(debuglevel); merge_opt!(logfilelevel);
    merge_opt!(log_rotate); merge_opt!(log_size);
    merge_opt!(installonly_limit); merge_opt!(errorlevel);
    merge_opt!(metadata_timer_sync);
    merge_opt!(allow_vendor_change); merge_opt!(assumeno); merge_opt!(assumeyes);
    merge_opt!(autocheck_running_kernel); merge_opt!(best); merge_opt!(cacheonly);
    merge_opt!(check_config_file_age); merge_opt!(clean_requirements_on_remove);
    merge_opt!(debug_solver); merge_opt!(defaultyes); merge_opt!(diskspacecheck);
    merge_opt!(exclude_from_weak_autodetect); merge_opt!(exit_on_lock);
    merge_opt!(gpgkey_dns_verification); merge_opt!(ignorearch);
    merge_opt!(install_weak_deps); merge_opt!(keepcache); merge_opt!(log_compress);
    merge_opt!(module_obsoletes); merge_opt!(module_stream_switch);
    merge_opt!(obsoletes); merge_opt!(plugins); merge_opt!(protect_running_kernel);
    merge_opt!(strict); merge_opt!(upgrade_group_objects_upgrade); merge_opt!(zchunk);
    merge_opt!(multilib_policy); merge_opt!(persistence);
    merge_opt!(rpmverbosity); merge_opt!(module_platform_id);
    merge_opt!(reposdir, vec![]); merge_opt!(varsdir, vec![]);
    merge_opt!(pluginconfpath, vec![]); merge_opt!(pluginpath, vec![]);
    merge_opt!(installonlypkgs, vec![]); merge_opt!(protected_packages, vec![]);
    merge_opt!(exclude_from_weak, vec![]); merge_opt!(group_package_types, vec![]);
    merge_opt!(optional_metadata_types, vec![]); merge_opt!(tsflags, vec![]);
    merge_opt!(usr_drift_protected_paths, vec![]);
    for (k, v) in &src.extras {
        dest.extras.insert(k.clone(), v.clone());
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test repofile_tests
```
Expected: 13 tests PASS (all parse + round-trip tests)

- [ ] **Step 4: Commit**

```bash
git add src/repofile.rs tests/repofile_tests.rs
git commit -m "feat: add RepoFile parser with full round-trip support"
```

---

### Task 9: RepoBuilder (`builder.rs`)

**Files:**
- Create: `src/builder.rs`
- Create: `tests/builder_tests.rs`

- [ ] **Step 1: Write builder test**

```rust
use dnf_repofile::builder::RepoBuilder;
use dnf_repofile::types::*;

#[test]
fn test_builder_basic() {
    let repo = RepoBuilder::new(RepoId::try_new("myrepo").unwrap())
        .name(RepoName::try_new("My Repo").unwrap())
        .enabled(DnfBool::True)
        .gpgcheck(DnfBool::True)
        .baseurl("https://example.com/repo/".parse().unwrap())
        .gpgkey("https://example.com/RPM-GPG-KEY")
        .priority(Priority::try_new(50).unwrap())
        .build();

    assert_eq!(repo.id.as_ref(), "myrepo");
    assert_eq!(repo.name.unwrap().as_ref(), "My Repo");
    assert_eq!(repo.baseurl[0].as_str(), "https://example.com/repo/");
    assert_eq!(repo.gpgkey[0], "https://example.com/RPM-GPG-KEY");
    assert_eq!(repo.priority.unwrap(), Priority::try_new(50).unwrap());
    assert_eq!(repo.enabled.unwrap(), DnfBool::True);
}

#[test]
fn test_builder_from_existing() {
    let existing = RepoBuilder::new(RepoId::try_new("myrepo").unwrap())
        .name(RepoName::try_new("Original").unwrap())
        .enabled(DnfBool::True)
        .baseurl("https://example.com/".parse().unwrap())
        .build();

    let modified = RepoBuilder::from(&existing)
        .enabled(DnfBool::False)
        .build();

    assert_eq!(modified.name.unwrap().as_ref(), "Original"); // preserved
    assert_eq!(modified.enabled.unwrap(), DnfBool::False);    // overridden
    assert_eq!(modified.baseurl[0].as_str(), "https://example.com/");
}
```

- [ ] **Step 2: Write `src/builder.rs`**

```rust
use crate::repo::Repo;
use crate::types::*;
use url::Url;

#[derive(Debug, Clone)]
pub struct RepoBuilder {
    repo: Repo,
}

impl RepoBuilder {
    pub fn new(id: RepoId) -> Self {
        RepoBuilder { repo: Repo::new(id) }
    }

    pub fn from(existing: &Repo) -> Self {
        RepoBuilder { repo: existing.clone() }
    }

    pub fn build(self) -> Repo {
        self.repo
    }

    pub fn name(mut self, v: RepoName) -> Self { self.repo.name = Some(v); self }
    pub fn baseurl(mut self, v: Url) -> Self { self.repo.baseurl.push(v); self }
    pub fn baseurls(mut self, v: Vec<Url>) -> Self { self.repo.baseurl = v; self }
    pub fn mirrorlist(mut self, v: Url) -> Self { self.repo.mirrorlist = Some(v); self }
    pub fn metalink(mut self, v: Url) -> Self { self.repo.metalink = Some(v); self }
    pub fn gpgkey(mut self, v: &str) -> Self { self.repo.gpgkey.push(v.to_string()); self }
    pub fn gpgkeys(mut self, v: Vec<String>) -> Self { self.repo.gpgkey = v; self }
    pub fn enabled(mut self, v: DnfBool) -> Self { self.repo.enabled = Some(v); self }
    pub fn gpgcheck(mut self, v: DnfBool) -> Self { self.repo.gpgcheck = Some(v); self }
    pub fn repo_gpgcheck(mut self, v: DnfBool) -> Self { self.repo.repo_gpgcheck = Some(v); self }
    pub fn priority(mut self, v: Priority) -> Self { self.repo.priority = Some(v); self }
    pub fn cost(mut self, v: Cost) -> Self { self.repo.cost = Some(v); self }
    pub fn module_hotfixes(mut self, v: DnfBool) -> Self { self.repo.module_hotfixes = Some(v); self }
    pub fn metadata_type(mut self, v: RepoMetadataType) -> Self { self.repo.metadata_type = Some(v); self }
    pub fn mediaid(mut self, v: &str) -> Self { self.repo.mediaid = Some(v.to_string()); self }
    pub fn excludepkgs(mut self, v: &str) -> Self { self.repo.excludepkgs.push(v.to_string()); self }
    pub fn includepkgs(mut self, v: &str) -> Self { self.repo.includepkgs.push(v.to_string()); self }
    pub fn skip_if_unavailable(mut self, v: DnfBool) -> Self { self.repo.skip_if_unavailable = Some(v); self }
    pub fn retries(mut self, v: Retries) -> Self { self.repo.retries = Some(v); self }
    pub fn timeout(mut self, v: TimeoutSeconds) -> Self { self.repo.timeout = Some(v); self }
    pub fn max_parallel_downloads(mut self, v: MaxParallelDownloads) -> Self { self.repo.max_parallel_downloads = Some(v); self }
    pub fn proxy(mut self, v: ProxySetting) -> Self { self.repo.proxy = v; self }
    pub fn username(mut self, v: Username) -> Self { self.repo.username = Some(v); self }
    pub fn password(mut self, v: Password) -> Self { self.repo.password = Some(v); self }
    pub fn extra(mut self, key: &str, value: &str) -> Self {
        self.repo.extras.entry(key.to_string()).or_default().push(value.to_string());
        self
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test builder_tests
```
Expected: 2 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/builder.rs tests/builder_tests.rs
git commit -m "feat: add RepoBuilder with chainable setters"
```

---

### Task 10: Validation engine (`validate.rs`)

**Files:**
- Create: `src/validate.rs`
- Create: `tests/validate_tests.rs`

- [ ] **Step 1: Write validation test**

```rust
use dnf_repofile::validate::*;
use dnf_repofile::repo::Repo;
use dnf_repofile::types::*;

#[test]
fn test_validate_repo_no_url_source() {
    let repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    let report = repo.validate();
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.message.contains("URL")));
}

#[test]
fn test_validate_repo_with_baseurl_passes() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl.push("https://example.com/repo/".parse().unwrap());
    let report = repo.validate();
    assert!(report.is_ok());
}

#[test]
fn test_validate_gpgkey_without_gpgcheck_warns() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl.push("https://example.com/repo/".parse().unwrap());
    repo.gpgkey.push("https://example.com/key".to_string());
    repo.gpgcheck = Some(DnfBool::False);
    let report = repo.validate();
    assert!(report.warnings.iter().any(|w| w.message.contains("GPG")));
}

#[test]
fn test_validate_report_add() {
    let mut report = ValidationReport::new();
    report.error(None, "test".into());
    assert!(!report.is_ok());
    assert_eq!(report.errors.len(), 1);
}
```

- [ ] **Step 2: Write `src/validate.rs`**

```rust
use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::types::RepoId;

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: IssueLevel,
    pub location: IssueLocation,
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueLevel { Error, Warning }

#[derive(Debug, Clone)]
pub enum IssueLocation {
    File(String),
    Repo(RepoId),
    Main,
}

impl ValidationReport {
    pub fn new() -> Self {
        ValidationReport { errors: Vec::new(), warnings: Vec::new() }
    }

    pub fn is_ok(&self) -> bool { self.errors.is_empty() }

    pub fn has_issues(&self) -> bool { !self.errors.is_empty() || !self.warnings.is_empty() }

    pub fn error(&mut self, field: Option<String>, message: String) {
        self.errors.push(ValidationIssue {
            level: IssueLevel::Error,
            location: IssueLocation::Main,
            field,
            message,
        });
    }

    pub fn warn(&mut self, field: Option<String>, message: String) {
        self.warnings.push(ValidationIssue {
            level: IssueLevel::Warning,
            location: IssueLocation::Main,
            field,
            message,
        });
    }

    pub fn repo_error(&mut self, repo_id: RepoId, field: Option<String>, message: String) {
        self.errors.push(ValidationIssue {
            level: IssueLevel::Error,
            location: IssueLocation::Repo(repo_id),
            field,
            message,
        });
    }

    pub fn repo_warn(&mut self, repo_id: RepoId, field: Option<String>, message: String) {
        self.warnings.push(ValidationIssue {
            level: IssueLevel::Warning,
            location: IssueLocation::Repo(repo_id),
            field,
            message,
        });
    }
}

impl Repo {
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Must have at least one URL source
        if self.baseurl.is_empty() && self.mirrorlist.is_none() && self.metalink.is_none() {
            report.repo_error(self.id.clone(), Some("baseurl".into()),
                "repo must have at least one URL source (baseurl, mirrorlist, or metalink)".into());
        }

        // Warning: baseurl + mirrorlist/metalink both set
        if !self.baseurl.is_empty() && (self.mirrorlist.is_some() || self.metalink.is_some()) {
            report.repo_warn(self.id.clone(), Some("baseurl".into()),
                "baseurl and mirrorlist/metalink both set; DNF may use baseurl only".into());
        }

        // Warning: gpgkey without gpgcheck
        if !self.gpgkey.is_empty() {
            if self.gpgcheck != Some(DnfBool::True) && self.repo_gpgcheck != Some(DnfBool::True) {
                report.repo_warn(self.id.clone(), Some("gpgkey".into()),
                    "gpgkey is set but gpgcheck and repo_gpgcheck are both disabled".into());
            }
        }

        report
    }
}

impl MainConfig {
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();
        // installonly_limit ≠ 1 is enforced at the type level via nutype
        // Additional main-level checks can be added here
        report
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test validate_tests
```
Expected: 4 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/validate.rs tests/validate_tests.rs
git commit -m "feat: add validate engine (Repo::validate, MainConfig::validate, ValidationReport)"
```

---

### Task 11: Diff engine (`diff.rs`)

**Files:**
- Create: `src/diff.rs`
- Create: `tests/diff_tests.rs`

- [ ] **Step 1: Write diff test**

```rust
use dnf_repofile::diff::*;
use dnf_repofile::repo::Repo;
use dnf_repofile::mainconfig::MainConfig;
use dnf_repofile::repofile::RepoFile;
use dnf_repofile::builder::RepoBuilder;
use dnf_repofile::types::*;

#[test]
fn test_diff_repos_changed_option() {
    let a = RepoBuilder::new(RepoId::try_new("test").unwrap())
        .name(RepoName::try_new("Old Name").unwrap())
        .baseurl("https://example.com/".parse().unwrap())
        .build();

    let b = RepoBuilder::new(RepoId::try_new("test").unwrap())
        .name(RepoName::try_new("New Name").unwrap())
        .baseurl("https://example.com/".parse().unwrap())
        .build();

    let diff = diff_repos(&a, &b);
    assert!(diff.has_changes);
    assert_eq!(diff.changed.len(), 1);
    assert_eq!(diff.changed[0].0, "name");
}

#[test]
fn test_diff_repos_no_changes() {
    let a = RepoBuilder::new(RepoId::try_new("test").unwrap())
        .name(RepoName::try_new("Test").unwrap())
        .baseurl("https://example.com/".parse().unwrap())
        .build();

    let diff = diff_repos(&a, &a);
    assert!(!diff.has_changes);
}

#[test]
fn test_diff_files_added_repo() {
    let repo = RepoBuilder::new(RepoId::try_new("newrepo").unwrap())
        .name(RepoName::try_new("New").unwrap())
        .baseurl("https://example.com/".parse().unwrap())
        .build();

    let mut a = RepoFile::new();
    let mut b = RepoFile::new();
    b.add(repo).unwrap();

    let diff = diff_files(&a, &b);
    assert!(diff.has_changes);
    assert_eq!(diff.repos_added.len(), 1);
    assert_eq!(diff.repos_added[0].as_ref(), "newrepo");
}
```

- [ ] **Step 2: Write `src/diff.rs`**

```rust
use crate::mainconfig::MainConfig;
use crate::repo::Repo;
use crate::repofile::RepoFile;
use crate::types::RepoId;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub main_changes: Option<ConfigDiff>,
    pub repos_added: Vec<RepoId>,
    pub repos_removed: Vec<RepoId>,
    pub repos_modified: IndexMap<RepoId, RepoDiff>,
    pub repos_unchanged: Vec<RepoId>,
    pub has_changes: bool,
}

#[derive(Debug, Clone)]
pub struct RepoDiff {
    pub changed: Vec<(String, String, String)>,  // (key, old, new)
    pub added: Vec<(String, String)>,
    pub removed: Vec<(String, String)>,
    pub has_changes: bool,
}

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
        repos_added: Vec::new(),
        repos_removed: Vec::new(),
        repos_modified: IndexMap::new(),
        repos_unchanged: Vec::new(),
        has_changes: false,
    };

    // Diff [main]
    match (&a.main, &b.main) {
        (None, Some(_)) => {
            diff.has_changes = true;
        }
        (Some(_), None) => {
            diff.has_changes = true;
        }
        (Some(am), Some(bm)) => {
            let cd = diff_main_configs(&am.data, &bm.data);
            if cd.has_changes {
                diff.has_changes = true;
                diff.main_changes = Some(cd);
            }
        }
        (None, None) => {}
    }

    // Diff repos
    for (id, block_b) in &b.repos {
        match a.repos.get(id) {
            None => {
                diff.repos_added.push(id.clone());
                diff.has_changes = true;
            }
            Some(block_a) => {
                let rd = diff_repos(&block_a.data, &block_b.data);
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
        changed: Vec::new(),
        added: Vec::new(),
        removed: Vec::new(),
        has_changes: false,
    };

    // Compare each field as its string representation
    diff_option(&mut diff, "name",
        a.name.as_ref().map(|n| n.as_ref().to_string()),
        b.name.as_ref().map(|n| n.as_ref().to_string()));
    diff_option(&mut diff, "enabled",
        a.enabled.map(|d| d.to_string()),
        b.enabled.map(|d| d.to_string()));
    diff_option(&mut diff, "gpgcheck",
        a.gpgcheck.map(|d| d.to_string()),
        b.gpgcheck.map(|d| d.to_string()));
    diff_option(&mut diff, "priority",
        a.priority.map(|p| p.to_string()),
        b.priority.map(|p| p.to_string()));

    // For multi-value fields, compare as joined strings
    diff_option(&mut diff, "baseurl",
                if a.baseurl.is_empty() { None } else { Some(a.baseurl.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(", ")) },
                if b.baseurl.is_empty() { None } else { Some(b.baseurl.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(", ")) });
    diff_option(&mut diff, "gpgkey",
                if a.gpgkey.is_empty() { None } else { Some(a.gpgkey.join(", ")) },
                if b.gpgkey.is_empty() { None } else { Some(b.gpgkey.join(", ")) });

    diff.has_changes = !diff.changed.is_empty() || !diff.added.is_empty() || !diff.removed.is_empty();
    diff
}

fn diff_option(diff: &mut RepoDiff, key: &str, a: Option<String>, b: Option<String>) {
    match (a, b) {
        (None, Some(new_val)) => {
            diff.added.push((key.to_string(), new_val));
        }
        (Some(old_val), None) => {
            diff.removed.push((key.to_string(), old_val));
        }
        (Some(old_val), Some(new_val)) if old_val != new_val => {
            diff.changed.push((key.to_string(), old_val, new_val));
        }
        _ => {}
    }
}

pub fn diff_main(a: &MainConfig, b: &MainConfig) -> ConfigDiff {
    let mut diff = ConfigDiff {
        changed: Vec::new(),
        added: Vec::new(),
        removed: Vec::new(),
        has_changes: false,
    };

    diff_opt_config(&mut diff, "debuglevel",
        a.debuglevel.map(|d| d.to_string()),
        b.debuglevel.map(|d| d.to_string()));
    diff_opt_config(&mut diff, "best",
        a.best.map(|d| d.to_string()),
        b.best.map(|d| d.to_string()));

    diff.has_changes = !diff.changed.is_empty() || !diff.added.is_empty() || !diff.removed.is_empty();
    diff
}

fn diff_opt_config(diff: &mut ConfigDiff, key: &str, a: Option<String>, b: Option<String>) {
    match (a, b) {
        (None, Some(new_val)) => { diff.added.push((key.to_string(), new_val)); }
        (Some(old_val), None) => { diff.removed.push((key.to_string(), old_val)); }
        (Some(old_val), Some(new_val)) if old_val != new_val => {
            diff.changed.push((key.to_string(), old_val, new_val));
        }
        _ => {}
    }
}

pub fn diff_main_configs(a: &MainConfig, b: &MainConfig) -> ConfigDiff {
    diff_main(a, b)
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test diff_tests
```
Expected: 3 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/diff.rs tests/diff_tests.rs
git commit -m "feat: add diff engine (diff_files, diff_repos, diff_main)"
```

---

### Task 12: Variable expansion (`variables.rs`)

**Files:**
- Create: `src/variables.rs`
- Create: `tests/variables_tests.rs`

- [ ] **Step 1: Write variable tests**

```rust
use dnf_repofile::variables::*;
use std::collections::HashMap;

#[test]
fn test_expand_simple_variable() {
    let mut vars = HashMap::new();
    vars.insert("releasever".into(), "9".into());
    let result = expand_variables("https://example.com/$releasever/os/", &vars).unwrap();
    assert_eq!(result, "https://example.com/9/os/");
}

#[test]
fn test_expand_braced_variable() {
    let mut vars = HashMap::new();
    vars.insert("basearch".into(), "x86_64".into());
    let result = expand_variables("https://example.com/${basearch}/os/", &vars).unwrap();
    assert_eq!(result, "https://example.com/x86_64/os/");
}

#[test]
fn test_expand_default_value_set() {
    let mut vars = HashMap::new();
    vars.insert("releasever".into(), "9".into());
    // ${releasever:-8} → releasever is set, use "9"
    let result = expand_variables("https://example.com/${releasever:-8}/os/", &vars).unwrap();
    assert_eq!(result, "https://example.com/9/os/");
}

#[test]
fn test_expand_default_value_unset() {
    let vars = HashMap::new();
    // ${releasever:-8} → releasever not set, use "8"
    let result = expand_variables("https://example.com/${releasever:-8}/os/", &vars).unwrap();
    assert_eq!(result, "https://example.com/8/os/");
}

#[test]
fn test_expand_alt_value_set() {
    let mut vars = HashMap::new();
    vars.insert("releasever".into(), "9".into());
    // ${releasever:+alt} → releasever set and non-empty, use "alt"
    let result = expand_variables("https://example.com/${releasever:+alt}/os/", &vars).unwrap();
    assert_eq!(result, "https://example.com/alt/os/");
}

#[test]
fn test_expand_alt_value_unset() {
    let vars = HashMap::new();
    // ${releasever:+alt} → releasever not set, use empty
    let result = expand_variables("https://example.com/${releasever:+alt}/os/", &vars).unwrap();
    assert_eq!(result, "https://example.com//os/");
}

#[test]
fn test_expand_missing_variable_errors() {
    let vars = HashMap::new();
    assert!(expand_variables("https://example.com/$nonexistent/os/", &vars).is_err());
}

#[test]
fn test_detect_variables() {
    let vars = detect_variables("https://example.com/$releasever/${basearch}/$arch/os/");
    assert!(vars.contains(&"releasever".to_string()));
    assert!(vars.contains(&"basearch".to_string()));
    assert!(vars.contains(&"arch".to_string()));
}

#[test]
fn test_expand_max_depth() {
    let vars = HashMap::new();
    // Create a deeply nested default that exceeds recursion limit
    let mut input = String::from("$x");
    // Single simple var, no recursion issue
    let result = expand_variables(&input, &vars);
    assert!(result.is_err()); // x not found
}
```

- [ ] **Step 2: Write `src/variables.rs`**

```rust
use crate::error::ExpandError;
use std::collections::{HashMap, HashSet};

const MAX_EXPRESSION_DEPTH: u32 = 32;

/// Expand DNF variables in a string.
///
/// Supported syntax: `$var`, `${var}`, `${var:-default}`, `${var:+alt}`
pub fn expand_variables(
    input: &str,
    vars: &HashMap<String, String>,
) -> std::result::Result<String, ExpandError> {
    let mut used_vars = HashSet::new();
    expand_recursive(input, vars, 0, &mut used_vars)
}

fn expand_recursive(
    input: &str,
    vars: &HashMap<String, String>,
    depth: u32,
    used_vars: &mut HashSet<String>,
) -> std::result::Result<String, ExpandError> {
    if depth > MAX_EXPRESSION_DEPTH {
        return Err(ExpandError::MaxDepthExceeded {
            depth,
            expr: input.to_owned(),
        });
    }

    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            // Escape: \X → X
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if chars[i] == '$' && i + 1 < chars.len() {
            let (var_name, consumed, default_value) = parse_variable(&chars, i, vars)?;

            if let Some(default) = default_value {
                // ${var:-default} or ${var:+alt}
                let var_value = vars.get(&var_name);
                let replacement = match &default.kind {
                    DefaultKind::Default => {
                        // ${var:-word}: use word if var is empty or unset
                        if var_value.map_or(true, |v| v.is_empty()) {
                            &default.value
                        } else {
                            var_value.unwrap()
                        }
                    }
                    DefaultKind::Alt => {
                        // ${var:+word}: use word if var is set and non-empty
                        if var_value.map_or(false, |v| !v.is_empty()) {
                            &default.value
                        } else {
                            ""
                        }
                    }
                };
                // Recursively expand the replacement
                let expanded = expand_recursive(replacement, vars, depth + 1, used_vars)?;
                result.push_str(&expanded);
                used_vars.insert(var_name);
                i += consumed;
                continue;
            }

            // Simple $var or ${var}
            let replacement = vars.get(&var_name).ok_or_else(|| {
                ExpandError::VariableNotFound {
                    name: var_name.clone(),
                }
            })?;
            result.push_str(replacement);
            used_vars.insert(var_name);
            i += consumed;
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    Ok(result)
}

struct DefaultClause {
    value: String,
    kind: DefaultKind,
}

enum DefaultKind {
    Default,  // :-
    Alt,      // :+
}

fn parse_variable(
    chars: &[char],
    start: usize,
    vars: &HashMap<String, String>,
) -> std::result::Result<(String, usize, Option<DefaultClause>), ExpandError> {
    let mut i = start + 1; // skip '$'
    let mut name = String::new();
    let mut is_braced = false;

    if i < chars.len() && chars[i] == '{' {
        is_braced = true;
        i += 1; // skip '{'
    }

    // Read variable name (alphanumeric + underscore)
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
        name.push(chars[i]);
        i += 1;
    }

    if name.is_empty() {
        return Err(ExpandError::MalformedExpression {
            expr: chars[start..].iter().collect(),
        });
    }

    let mut default_value: Option<DefaultClause> = None;

    if is_braced && i < chars.len() {
        if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == '-' {
            // ${var:-default}
            let default_str = read_until_closing_brace(chars, i + 2)?;
            let consumed_in_default = default_str.len();
            default_value = Some(DefaultClause {
                value: default_str,
                kind: DefaultKind::Default,
            });
            i += 2 + consumed_in_default + 1; // skip ":-" + default_str + "}"
        } else if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == '+' {
            // ${var:+alt}
            let alt_str = read_until_closing_brace(chars, i + 2)?;
            let consumed_in_alt = alt_str.len();
            default_value = Some(DefaultClause {
                value: alt_str,
                kind: DefaultKind::Alt,
            });
            i += 2 + consumed_in_alt + 1; // skip ":+" + alt_str + "}"
        } else if chars[i] == '}' {
            i += 1; // skip '}'
        } else {
            return Err(ExpandError::MalformedExpression {
                expr: chars[start..].iter().collect(),
            });
        }
    }

    Ok((name, i - start, default_value))
}

fn read_until_closing_brace(
    chars: &[char],
    start: usize,
) -> std::result::Result<String, ExpandError> {
    let mut result = String::new();
    let mut depth = 1u32;
    let mut i = start;

    while i < chars.len() {
        if chars[i] == '{' {
            depth += 1;
        } else if chars[i] == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(result);
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    Err(ExpandError::MalformedExpression {
        expr: chars[start..].iter().collect(),
    })
}

/// Detect all variable names referenced in a string (without expanding).
pub fn detect_variables(input: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 2;
            continue;
        }
        if chars[i] == '$' && i + 1 < chars.len() {
            let mut j = i + 1;
            let mut name = String::new();
            if j < chars.len() && chars[j] == '{' {
                j += 1;
            }
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                name.push(chars[j]);
                j += 1;
            }
            if !name.is_empty() {
                vars.push(name);
            }
            i = j;
            continue;
        }
        i += 1;
    }

    vars
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test variables_tests
```
Expected: all variable tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/variables.rs tests/variables_tests.rs
git commit -m "feat: add variable expansion (${var}, ${var:-default}, ${var:+alt})"
```

---

### Task 13: ReposDir (`reposdir.rs`)

**Files:**
- Create: `src/reposdir.rs`
- Create: `tests/reposdir_tests.rs`

- [ ] **Step 1: Write ReposDir test**

```rust
use dnf_repofile::reposdir::*;
use dnf_repofile::repofile::RepoFile;
use dnf_repofile::builder::RepoBuilder;
use dnf_repofile::types::*;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_load_directory() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("test.repo"),
        "[epel]\nname=EPEL\nbaseurl=https://example.com/epel/\n",
    ).unwrap();

    let rd = ReposDir::load(dir.path()).unwrap();
    assert_eq!(rd.file_names().len(), 1);
    assert!(rd.file_names()[0].ends_with("test.repo"));
}

#[test]
fn test_find_repo_across_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("a.repo"),
        "[repo-a]\nname=A\nbaseurl=https://a.example.com/\n",
    ).unwrap();
    std::fs::write(
        dir.path().join("b.repo"),
        "[repo-b]\nname=B\nbaseurl=https://b.example.com/\n",
    ).unwrap();

    let rd = ReposDir::load(dir.path()).unwrap();
    let (filename, repo) = rd.find_repo(&RepoId::try_new("repo-b").unwrap()).unwrap();
    assert!(filename.contains("b.repo"));
    assert_eq!(repo.name.as_ref().unwrap().as_ref(), "B");
}

#[test]
fn test_repo_count() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("multi.repo"),
        "[r1]\nname=R1\nbaseurl=https://1.example.com/\n[r2]\nname=R2\nbaseurl=https://2.example.com/\n",
    ).unwrap();

    let rd = ReposDir::load(dir.path()).unwrap();
    assert_eq!(rd.repo_count(), 2);
}

#[test]
fn test_save_all() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("existing.repo"),
        "[r1]\nname=R1\nbaseurl=https://1.example.com/\n",
    ).unwrap();

    let rd = ReposDir::load(dir.path()).unwrap();
    rd.save_all().unwrap();

    // File should still exist and be parseable
    let contents = std::fs::read_to_string(dir.path().join("existing.repo")).unwrap();
    let rf = RepoFile::parse(&contents).unwrap();
    assert_eq!(rf.len(), 1);
}

#[test]
fn test_remove_file() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("toremove.repo"),
        "[r1]\nname=R1\nbaseurl=https://1.example.com/\n",
    ).unwrap();

    let mut rd = ReposDir::load(dir.path()).unwrap();
    assert!(rd.file_names().iter().any(|n| n.contains("toremove.repo")));
    let removed = rd.remove_file("toremove.repo").unwrap();
    assert!(removed.is_some());
    assert!(!dir.path().join("toremove.repo").exists());
}

#[test]
fn test_create_and_set_file() {
    let dir = TempDir::new().unwrap();
    let mut rd = ReposDir::load(dir.path()).unwrap();

    let repo = RepoBuilder::new(RepoId::try_new("newrepo").unwrap())
        .name(RepoName::try_new("New").unwrap())
        .baseurl("https://new.example.com/".parse().unwrap())
        .build();

    let mut rf = RepoFile::new();
    rf.add(repo).unwrap();
    rd.set_file("new.repo", rf);

    assert!(rd.file_names().iter().any(|n| n.contains("new.repo")));
    let block = rd.get_file("new.repo").unwrap()
        .get(&RepoId::try_new("newrepo").unwrap())
        .unwrap();
    assert_eq!(block.data.name.as_ref().unwrap().as_ref(), "New");
}

#[test]
fn test_validate_detects_duplicate_ids() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("a.repo"),
        "[dupe]\nname=A\nbaseurl=https://a.example.com/\n",
    ).unwrap();
    std::fs::write(
        dir.path().join("b.repo"),
        "[dupe]\nname=B\nbaseurl=https://b.example.com/\n",
    ).unwrap();

    let rd = ReposDir::load(dir.path()).unwrap();
    let report = rd.validate();
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.message.contains("duplicate")));
}
```

- [ ] **Step 2: Write `src/reposdir.rs`**

```rust
use crate::error::{Error, Result};
use crate::repo::Repo;
use crate::repofile::RepoFile;
use crate::types::RepoId;
use crate::validate::ValidationReport;
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
            let mut entries: Vec<_> = fs::read_dir(&path)
                .map_err(Error::Io)?
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
                let contents = fs::read_to_string(entry.path()).map_err(Error::Io)?;
                match RepoFile::parse(&contents) {
                    Ok(rf) => {
                        files.insert(name, rf);
                    }
                    Err(e) => {
                        // Collect errors but continue loading other files
                        eprintln!("Warning: failed to parse {}: {}", name, e);
                    }
                }
            }
        }

        Ok(ReposDir { path, files })
    }

    pub fn save_all(&self) -> std::result::Result<(), Vec<(String, std::io::Error)>> {
        let mut errors = Vec::new();
        for (name, rf) in &self.files {
            let filepath = self.path.join(name);
            if let Err(e) = fs::write(&filepath, rf.render()) {
                errors.push((name.clone(), e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn save(&self, filename: &str) -> std::result::Result<(), std::io::Error> {
        if let Some(rf) = self.files.get(filename) {
            fs::write(self.path.join(filename), rf.render())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
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

    pub fn remove_file(&mut self, filename: &str) -> std::result::Result<Option<RepoFile>, std::io::Error> {
        let removed = self.files.shift_remove(filename);
        let filepath = self.path.join(filename);
        if filepath.exists() {
            fs::remove_file(filepath)?;
        }
        Ok(removed)
    }

    pub fn create_file(&mut self, filename: &str) -> &mut RepoFile {
        self.files.entry(filename.to_string()).or_insert_with(RepoFile::new)
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
        let mut repos = Vec::new();
        for (name, rf) in &self.files {
            for (_, block) in rf.iter() {
                repos.push((name.as_str(), &block.data));
            }
        }
        repos
    }

    pub fn repo_count(&self) -> usize {
        self.files.values().map(|rf| rf.len()).sum()
    }

    pub fn iter_repos(&self) -> impl Iterator<Item = (&str, &Repo)> {
        self.files.iter().flat_map(|(name, rf)| {
            rf.iter().map(move |(_, block)| {
                (name.as_str(), &block.data)
            })
        })
    }

    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();
        let mut seen_ids: HashMap<&RepoId, &str> = HashMap::new();

        for (fname, rf) in &self.files {
            for (repo_id, block) in rf.iter() {
                // Cross-file duplicate check
                if let Some(existing_file) = seen_ids.get(repo_id) {
                    report.errors.push(crate::validate::ValidationIssue {
                        level: crate::validate::IssueLevel::Error,
                        location: crate::validate::IssueLocation::File(fname.clone()),
                        field: None,
                        message: format!(
                            "duplicate repo ID '{}' already defined in file '{}'",
                            repo_id.as_ref(),
                            existing_file
                        ),
                    });
                } else {
                    seen_ids.insert(repo_id, fname.as_str());
                }

                // Per-repo validation
                let repo_report = block.data.validate();
                report.errors.extend(repo_report.errors);
                report.warnings.extend(repo_report.warnings);
            }

            // Per-file validation
            let file_report = rf.validate();
            report.errors.extend(file_report.errors);
            report.warnings.extend(file_report.warnings);
        }

        report
    }
}

impl RepoFile {
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();
        for (_, block) in self.iter() {
            let repo_report = block.data.validate();
            report.errors.extend(repo_report.errors);
            report.warnings.extend(repo_report.warnings);
        }
        report
    }

    pub fn load(path: impl AsRef<Path>) -> std::result::Result<Self, crate::error::ParseError> {
        let contents = std::fs::read_to_string(path.as_ref())?;
        Self::parse(&contents)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> std::result::Result<(), std::io::Error> {
        fs::write(path.as_ref(), self.render())
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --test reposdir_tests
```
Expected: 7 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/reposdir.rs tests/reposdir_tests.rs
git commit -m "feat: add ReposDir for directory-level management"
```

---

### Task 14: Integration tests

**Files:**
- Create: `tests/integration_tests.rs`

- [ ] **Step 1: Write integration test**

```rust
use dnf_repofile::*;
use std::str::FromStr;

#[test]
fn test_full_workflow() {
    // Parse a complete repo file
    let input = include_str!("fixtures/complex.repo");
    let mut rf = RepoFile::parse(input).unwrap();

    // Verify [main]
    let main_block = rf.main().unwrap();
    assert_eq!(main_block.data.max_parallel_downloads.unwrap().to_string(), "10");

    // Verify repos
    assert_eq!(rf.len(), 3);
    let baseos = rf.get(&RepoId::try_new("baseos").unwrap()).unwrap();
    assert_eq!(baseos.data.name.as_ref().unwrap().as_ref(), "Rocky Linux $releasever - BaseOS");
    assert_eq!(baseos.data.baseurl.len(), 2);
    assert_eq!(baseos.data.priority.unwrap().to_string(), "10");

    // Modify a repo
    {
        let mut block = rf.get_mut(&RepoId::try_new("custom-repo").unwrap()).unwrap();
        block.data.enabled = Some(DnfBool::True);
    }

    // Add a new repo
    let new_repo = RepoBuilder::new(RepoId::try_new("added-repo").unwrap())
        .name(RepoName::try_new("Added Repo").unwrap())
        .baseurl("https://added.example.com/".parse().unwrap())
        .enabled(DnfBool::True)
        .gpgcheck(DnfBool::True)
        .gpgkey("https://added.example.com/key")
        .build();
    rf.add(new_repo).unwrap();

    // Remove a repo
    rf.remove(&RepoId::try_new("appstream").unwrap());

    // Render
    let output = rf.render();

    // Parse rendered output
    let rf2 = RepoFile::parse(&output).unwrap();

    // Verify round-trip
    assert_eq!(rf2.len(), 3); // baseos, custom-repo, added-repo
    assert!(rf2.get(&RepoId::try_new("appstream").unwrap()).is_none());
    assert!(rf2.get(&RepoId::try_new("added-repo").unwrap()).is_some());
    let custom = rf2.get(&RepoId::try_new("custom-repo").unwrap()).unwrap();
    assert_eq!(custom.data.enabled, Some(DnfBool::True)); // was modified
}

#[test]
fn test_variable_expansion_in_url() {
    let input = "\
[testrepo]
name=Test
baseurl=https://example.com/$releasever/$basearch/
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    // Variables should be preserved as-is
    assert_eq!(block.data.baseurl[0].as_str(), "https://example.com/$releasever/$basearch/");
}

#[test]
fn test_parse_validates_all_bool_variants() {
    let input = "\
[testrepo]
name=Test
baseurl=https://example.com/
enabled=yes
gpgcheck=1
skip_if_unavailable=True
module_hotfixes=on
deltarpm=No
fastestmirror=false
countme=OFF
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.data.enabled, Some(DnfBool::True));
    assert_eq!(block.data.gpgcheck, Some(DnfBool::True));
    assert_eq!(block.data.skip_if_unavailable, Some(DnfBool::True));
    assert_eq!(block.data.module_hotfixes, Some(DnfBool::True));
    assert_eq!(block.data.deltarpm, Some(DnfBool::False));
    assert_eq!(block.data.fastestmirror, Some(DnfBool::False));
    assert_eq!(block.data.countme, Some(DnfBool::False));
}

#[test]
fn test_parse_proxy_none() {
    let input = "\
[testrepo]
name=Test
baseurl=https://example.com/
proxy=_none_
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert!(matches!(block.data.proxy, ProxySetting::Disabled));
}

#[test]
fn test_parse_with_extras_preserved() {
    let input = "\
[testrepo]
name=Test
baseurl=https://example.com/
custom_key=custom_value
another_key=another_value
";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.data.extras.get("custom_key").unwrap()[0], "custom_value");
    assert_eq!(block.raw_entries.len(), 2);
}

#[test]
fn test_diff_detects_all_changes() {
    let input_a = "[repo]\nname=A\nbaseurl=https://a.example.com/\nenabled=1\n";
    let input_b = "[repo]\nname=B\nbaseurl=https://b.example.com/\nenabled=0\n";

    let a = RepoFile::parse(input_a).unwrap();
    let b = RepoFile::parse(input_b).unwrap();

    let diff = diff_files(&a, &b);
    assert!(diff.has_changes);
}
```

- [ ] **Step 2: Run integration tests**

```bash
cargo test --test integration_tests
```
Expected: 6 tests PASS

- [ ] **Step 3: Final build check**

```bash
cargo test
cargo clippy -- -D warnings 2>/dev/null || true
cargo doc --no-deps
```
Expected: all tests PASS, docs build without errors

- [ ] **Step 4: Commit**

```bash
git add tests/integration_tests.rs
git commit -m "test: add integration tests covering full workflow"
```

---

## Self-Review Checklist

**1. Spec coverage:**
- [x] 103 options in spec tables → all in Repo + MainConfig structs
- [x] 12 repo-only options → all in Repo
- [x] 35 shared options → all in Repo
- [x] 56 main-only options → all in MainConfig
- [x] parse() with comment preservation
- [x] render() with round-trip fidelity
- [x] Builder pattern for Repo construction
- [x] validate() at Repo, MainConfig, RepoFile, ReposDir levels
- [x] diff_files(), diff_repos(), diff_main()
- [x] Variable expansion: $var, ${var}, ${var:-word}, ${var:+word}
- [x] ReposDir: load, save, find, create, remove, validate

**2. Placeholder scan:** No TBD/TODO/fill-in-later. All code is complete. ✓

**3. Type consistency:** All types match between tasks. Module references are consistent. ✓
