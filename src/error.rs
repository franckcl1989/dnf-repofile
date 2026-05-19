//! Error types for DNF `.repo` file parsing, validation, and variable expansion.
//!
//! This module defines the top-level [`enum@Error`] enum (via [`thiserror`]) that
//! aggregates all domain-specific error types in the library:
//!
//! - [`ParseError`] — INI parsing failures (malformed sections, missing `=`, etc.)
//! - [`ParseBoolError`] — invalid boolean values like `"maybe"`
//! - [`ExpandError`] — variable substitution failures (unknown name, depth, syntax)
//!
//! The [`Result`] type alias is a convenience for `Result<T, Error>`.

use thiserror::Error;

/// Top-level error type for the library.
///
/// Aggregates all domain-specific errors into a single enum so callers can
/// use [`Result<T>`](Result) without naming every variant.
///
/// Supports automatic conversion from [`ParseError`], [`ParseBoolError`],
/// and [`std::io::Error`] via [`From`].
///
/// # Examples
///
/// ```
/// use dnf_repofile::Error;
///
/// let err = Error::Other("custom error".into());
/// assert_eq!(err.to_string(), "custom error");
/// ```
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to parse a `.repo` file string.
    #[error("failed to parse .repo file: {0}")]
    Parse(#[from] ParseError),

    /// Failed to parse a boolean value from a `.repo` file option.
    #[error("failed to parse boolean value '{0}'")]
    ParseBool(#[from] ParseBoolError),

    /// An option value is invalid for its expected type.
    #[error("invalid option value for '{key}': {message}")]
    InvalidValue {
        /// The option key (e.g., `"enabled"`, `"priority"`).
        key: String,
        /// A human-readable description of why the value is invalid.
        message: String,
    },

    /// A repository with this ID already exists in the file.
    #[error("repo '{0}' already exists in file")]
    DuplicateRepo(String),

    /// A repository with this ID was not found.
    #[error("repo '{0}' not found")]
    RepoNotFound(String),

    /// An I/O error occurred (file read/write).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A generic error message.
    #[error("{0}")]
    Other(String),
}

/// Result type alias for convenience.
///
/// Equivalent to `std::result::Result<T, [`Error`]>`.
pub type Result<T> = std::result::Result<T, Error>;

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

/// Error when parsing a boolean value from a `.repo` file option fails.
///
/// This occurs when a value expected to be a DNF boolean (`1`/`0`/`yes`/`no`/`true`/`false`/`on`/`off`)
/// contains something else entirely (e.g., `"maybe"`).
#[derive(Error, Debug)]
#[error("invalid boolean value: '{input}'")]
pub struct ParseBoolError {
    /// The raw input string that could not be parsed as a boolean.
    pub input: String,
}

/// Error when parsing a `.repo` file fails.
///
/// Covers all INI-level syntax issues: invalid section headers, missing
/// `=` separators, empty section names, and invalid repository IDs.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ParseError {
    /// A section header (e.g., `[repo-name]`) has invalid syntax.
    #[error("invalid section header at line {line}: '{header}'")]
    InvalidSection {
        /// The 1-based line number where the error occurred.
        line: usize,
        /// The raw header text.
        header: String,
    },

    /// A key-value pair is missing the `=` separator.
    #[error("missing '=' in key-value pair at line {line}: '{line_text}'")]
    MissingEquals {
        /// The 1-based line number where the error occurred.
        line: usize,
        /// The raw line text.
        line_text: String,
    },

    /// A section header with an empty name was encountered (`[]`).
    #[error("empty section name")]
    EmptySectionName,

    /// A repository ID contains invalid characters or is otherwise malformed.
    #[error("invalid repo ID '{id}': {reason}")]
    InvalidRepoId {
        /// The problematic repo ID string.
        id: String,
        /// Why the ID is invalid.
        reason: String,
    },

    /// An I/O error occurred while reading the file.
    #[error("I/O error reading file: {0}")]
    Io(#[from] std::io::Error),
}

/// Error when expanding DNF variables fails.
///
/// Covers three failure modes: a variable name not found in the substitution
/// map, exceeding the maximum recursion depth (default 32), and malformed
/// variable expression syntax.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ExpandError {
    /// A variable referenced in the input string was not found in the
    /// substitution map.
    #[error("variable '{name}' not found in substitution map")]
    VariableNotFound {
        /// The name of the missing variable.
        name: String,
    },

    /// The maximum recursion depth was exceeded while expanding a variable
    /// expression (likely a circular reference).
    #[error("maximum recursion depth ({depth}) exceeded while expanding '{expr}'")]
    MaxDepthExceeded {
        /// The depth at which expansion stopped.
        depth: u32,
        /// The expression being expanded when the limit was hit.
        expr: String,
    },

    /// A variable expression has invalid syntax (e.g., an empty `$`
    /// without a following name).
    #[error("malformed variable expression: '{expr}'")]
    MalformedExpression {
        /// The invalid expression text.
        expr: String,
    },
}
