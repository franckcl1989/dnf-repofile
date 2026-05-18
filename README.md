# dnf-repofile

[![Crates.io](https://img.shields.io/crates/v/dnf-repofile)](https://crates.io/crates/dnf-repofile)
[![docs.rs](https://img.shields.io/docsrs/dnf-repofile)](https://docs.rs/dnf-repofile)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A 100% pure Rust library for parsing, managing, validating, diffing, and rendering
DNF/YUM `.repo` configuration files with full round-trip fidelity.

## Features

- **Parse** `.repo` files into fully typed Rust structs (103 DNF options modeled)
- **Render** back to text preserving comments, blank lines, and ordering
- **Validate** repository configurations (URL sources, GPG consistency, etc.)
- **Diff** between repo files or individual repositories
- **Builder** pattern for programmatic creation
- **ReposDir** for managing a directory of `.repo` files
- **Variable expansion** supporting `$var`, `${var}`, `${var:-default}`, `${var:+alt}`

## Usage

```rust
use dnf_repofile::*;
use std::str::FromStr;

// Parse a repo file
let input = "\
[epel]
name=Extra Packages for Enterprise Linux
baseurl=https://download.example.com/pub/epel/$releasever/$basearch/
enabled=1
gpgcheck=1
gpgkey=https://download.example.com/pub/epel/RPM-GPG-KEY
";

let rf = RepoFile::parse(input)?;

// Read options
let block = rf.get(&RepoId::try_new("epel")?).unwrap();
println!("name: {}", block.data.name.as_ref().unwrap());
println!("baseurl: {}", block.data.baseurl[0]);

// Validate
let report = rf.validate();
assert!(report.is_ok());

// Modify
rf.get_mut(&RepoId::try_new("epel")?).unwrap().data.enabled = Some(DnfBool::False);

// Render back
println!("{}", rf.render());

// Or use Display
println!("{rf}");

// Programmatic creation with Builder
let new_repo = RepoBuilder::new(RepoId::try_new("custom")?)
    .name(RepoName::try_new("Custom Repository")?)
    .baseurl("https://custom.example.com/".parse()?)
    .gpgcheck(DnfBool::yes())
    .enabled(DnfBool::yes())
    .priority(Priority::try_new(50)?)
    .build();
```

## Three-Level API

| Level | Type | Purpose |
|-------|------|---------|
| Macro | `ReposDir` | Manage a directory of `.repo` files |
| Meso | `RepoFile` | Parse, modify, render a single file |
| Micro | `Repo` / `MainConfig` | Type-safe access to individual options |

## License

MIT
