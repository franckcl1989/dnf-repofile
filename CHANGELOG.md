# Changelog

## [0.1.1] - 2026-05-18

### Added
- Comprehensive documentation on all public types and methods
- Module-level documentation for all modules
- Intra-doc links between related types
- Crate-level documentation with quick-start example

## [0.1.0] - 2026-05-18

### Added
- Initial release
- INI parser with comment/whitespace preservation
- Fully typed Repo (48 fields) and MainConfig (57 fields) structs
- RepoFile parse/render with round-trip fidelity
- RepoBuilder for programmatic creation
- Validation engine with issue/warning reporting
- Diff engine for comparing repo files and repositories
- Variable expansion ($var, ${var}, ${var:-default}, ${var:+alt})
- ReposDir for directory-level management
- 210 tests, clippy clean, rustfmt clean
