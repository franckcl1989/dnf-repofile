use dnf_repofile::error::*;
use std::io;

// ---- ParseBoolError ----

#[test]
fn test_parse_bool_error_display() {
    let err = ParseBoolError { input: "maybe".into() };
    assert!(err.to_string().contains("maybe"));
}

// ---- AddRepoError ----

#[test]
fn test_add_repo_error_display() {
    let err = AddRepoError { id: "myrepo".into() };
    assert!(err.to_string().contains("myrepo"));
}

// ---- ParseError variants ----

#[test]
fn test_parse_error_invalid_section_display() {
    let err = ParseError::InvalidSection { line: 5, header: "[bad@id]".into() };
    let msg = err.to_string();
    assert!(msg.contains("5"));
    assert!(msg.contains("bad@id"));
}

#[test]
fn test_parse_error_missing_equals_display() {
    let err = ParseError::MissingEquals { line: 3, line_text: "badline".into() };
    let msg = err.to_string();
    assert!(msg.contains("3"));
    assert!(msg.contains("badline"));
}

#[test]
fn test_parse_error_empty_section_display() {
    let err = ParseError::EmptySectionName;
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_parse_error_invalid_repo_id_display() {
    let err = ParseError::InvalidRepoId { id: "bad@id".into(), reason: "invalid chars".into() };
    let msg = err.to_string();
    assert!(msg.contains("bad@id"));
    assert!(msg.contains("invalid chars"));
}

// ---- Error variants ----

#[test]
fn test_error_duplicate_repo_display() {
    let err = Error::DuplicateRepo("myrepo".into());
    assert!(err.to_string().contains("myrepo"));
}

#[test]
fn test_error_repo_not_found_display() {
    let err = Error::RepoNotFound("missing".into());
    assert!(err.to_string().contains("missing"));
}

#[test]
fn test_error_invalid_value_display() {
    let err = Error::InvalidValue { key: "priority".into(), message: "out of range".into() };
    let msg = err.to_string();
    assert!(msg.contains("priority"));
    assert!(msg.contains("out of range"));
}

#[test]
fn test_error_other_display() {
    let err = Error::Other("something went wrong".into());
    assert!(err.to_string().contains("something went wrong"));
}

// ---- From impls ----

#[test]
fn test_from_io_error_to_parse_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let parse_err: ParseError = io_err.into();
    assert!(matches!(parse_err, ParseError::Io(_)));
}

#[test]
fn test_from_parse_error_to_error() {
    let parse_err = ParseError::EmptySectionName;
    let err: Error = parse_err.into();
    assert!(matches!(err, Error::Parse(_)));
}

#[test]
fn test_from_parse_bool_error_to_error() {
    let bool_err = ParseBoolError { input: "invalid".into() };
    let err: Error = bool_err.into();
    assert!(matches!(err, Error::ParseBool(_)));
}

#[test]
fn test_from_io_error_to_error() {
    let io_err = io::Error::new(io::ErrorKind::Other, "boom");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

// ---- Error::source() ----

#[test]
fn test_error_source_returns_some_on_wrapped_errors() {
    use std::error::Error as StdError;
    let parse_err = ParseError::EmptySectionName;
    let err = Error::Parse(parse_err);
    assert!(err.source().is_some());
}

// ---- ExpandError ----

#[test]
fn test_expand_error_variable_not_found_display() {
    let err = ExpandError::VariableNotFound { name: "releasever".into() };
    assert!(err.to_string().contains("releasever"));
}

#[test]
fn test_expand_error_max_depth_display() {
    let err = ExpandError::MaxDepthExceeded { depth: 32, expr: "$x".into() };
    assert!(err.to_string().contains("32"));
    assert!(err.to_string().contains("$x"));
}

#[test]
fn test_expand_error_malformed_display() {
    let err = ExpandError::MalformedExpression { expr: "${bad".into() };
    assert!(err.to_string().contains("${bad"));
}

// ---- Send + Sync ----

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
    assert_send_sync::<ParseError>();
    assert_send_sync::<ExpandError>();
}
