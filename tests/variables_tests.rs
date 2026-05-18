use dnf_repofile::variables::*;
use std::collections::HashMap;

#[test]
fn test_expand_simple() {
    let mut v = HashMap::new();
    v.insert("releasever".into(), "9".into());
    assert_eq!(
        expand_variables("https://x.com/$releasever/os/", &v).unwrap(),
        "https://x.com/9/os/"
    );
}

#[test]
fn test_expand_braced() {
    let mut v = HashMap::new();
    v.insert("basearch".into(), "x86_64".into());
    assert_eq!(expand_variables("${basearch}", &v).unwrap(), "x86_64");
}

#[test]
fn test_expand_default_set() {
    let mut v = HashMap::new();
    v.insert("releasever".into(), "9".into());
    assert_eq!(expand_variables("${releasever:-8}", &v).unwrap(), "9");
}

#[test]
fn test_expand_default_unset() {
    assert_eq!(
        expand_variables("${releasever:-8}", &HashMap::new()).unwrap(),
        "8"
    );
}

#[test]
fn test_expand_alt_set() {
    let mut v = HashMap::new();
    v.insert("releasever".into(), "9".into());
    assert_eq!(expand_variables("${releasever:+alt}", &v).unwrap(), "alt");
}

#[test]
fn test_expand_alt_unset() {
    assert_eq!(
        expand_variables("${releasever:+alt}", &HashMap::new()).unwrap(),
        ""
    );
}

#[test]
fn test_expand_missing_error() {
    assert!(expand_variables("$nonexistent", &HashMap::new()).is_err());
}

#[test]
fn test_detect_variables() {
    let vars = detect_variables("https://x.com/$releasever/${basearch}/$arch/");
    assert!(vars.contains(&"releasever".to_string()));
    assert!(vars.contains(&"basearch".to_string()));
    assert!(vars.contains(&"arch".to_string()));
}
