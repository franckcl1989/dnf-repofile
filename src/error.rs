use thiserror::Error;

/// Top-level error type for the library
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse .repo file: {0}")]
    Parse(#[from] ParseError),

    #[error("failed to parse boolean value '{0}'")]
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
