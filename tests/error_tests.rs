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
