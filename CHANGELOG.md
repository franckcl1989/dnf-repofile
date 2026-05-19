# Changelog

## [0.1.2] - 2026-05-19

### Fixed
- `DnfBool::parse`: removed unnecessary heap allocation (zero-alloc on success path)
- `parse_proxy`: preserve unparseable proxy values via new `ProxySetting::Raw(String)` variant instead of silently returning `Unset`
- `merge_mainconfig`: fix Vec fields (reposdir, varsdir, tsflags, etc.) being silently dropped during merge
- `parse_storage_size`: use `checked_mul` to prevent u64 overflow panic on extreme values

### Changed
- `RepoFile::add` now returns `Result<(), Error>` (using `Error::DuplicateRepo`) instead of separate `AddRepoError` type; removed `AddRepoError`
- Added `#[non_exhaustive]` to 11 expandable public enums for forward compatibility
- Removed unused `KNOWN_REPO_KEYS` / `KNOWN_MAIN_KEYS` dead code
- Removed unnecessary `<T: Debug>` bound on internal `render_section_entries` function
- Examples: replaced unwrap patterns with `?` where applicable, safe name access

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
