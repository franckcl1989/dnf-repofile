use dnf_repofile::mainconfig::MainConfig;
use dnf_repofile::repo::Repo;
use dnf_repofile::repofile::{RawEntry, RepoFile};
use dnf_repofile::types::*;
use std::str::FromStr;

// ============================================================================
// Helper: a minimal fixture inline
// ============================================================================

const SIMPLE_REPO_INPUT: &str = "[epel]
name=Extra Packages for Enterprise Linux $releasever - $basearch
baseurl=https://download.example.com/pub/epel/$releasever/Everything/$basearch/
enabled=1
gpgcheck=1
gpgkey=https://download.example.com/pub/epel/RPM-GPG-KEY-EPEL-$releasever
";

#[test]
fn test_parse_simple_repo() {
    let rf = RepoFile::parse(SIMPLE_REPO_INPUT).expect("failed to parse simple repo");
    let epel_id = RepoId::try_new("epel").unwrap();
    let block = rf.get(&epel_id).expect("epel section not found");
    let repo = &block.data;

    assert_eq!(repo.name.as_ref().map(|n| n.as_ref()), Some("Extra Packages for Enterprise Linux $releasever - $basearch"));
    assert_eq!(repo.baseurl.len(), 1);
    assert_eq!(
        repo.baseurl[0].as_str(),
        "https://download.example.com/pub/epel/$releasever/Everything/$basearch/"
    );
    assert_eq!(repo.enabled, Some(DnfBool::True));
    assert_eq!(repo.gpgcheck, Some(DnfBool::True));
    assert_eq!(repo.gpgkey.len(), 1);
    assert_eq!(
        repo.gpgkey[0],
        "https://download.example.com/pub/epel/RPM-GPG-KEY-EPEL-$releasever"
    );
}

#[test]
fn test_parse_with_preamble_comments() {
    let input = "# This is a preamble comment
# Another preamble line

[myrepo]
name=My Repo
baseurl=https://example.com/repo/
enabled=1
";
    let rf = RepoFile::parse(input).expect("failed to parse");
    assert!(
        rf.preamble.iter().any(|l| l.contains("preamble comment")),
        "preamble should contain 'preamble comment'"
    );
    assert!(
        rf.preamble.iter().any(|l| l.contains("Another preamble")),
        "preamble should contain 'Another preamble'"
    );
    // The blank line in the preamble is preserved too
    assert_eq!(rf.preamble.len(), 3);
}

#[test]
fn test_parse_multiple_baseurl() {
    let input = "[multirepo]
name=Multi-URL Repo
baseurl=https://mirror1.example.com/repo/
baseurl=https://mirror2.example.com/repo/
baseurl=https://mirror3.example.com/repo/
enabled=1
";
    let rf = RepoFile::parse(input).expect("failed to parse");
    let rid = RepoId::try_new("multirepo").unwrap();
    let repo = &rf.get(&rid).unwrap().data;
    assert_eq!(repo.baseurl.len(), 3);
    assert_eq!(repo.baseurl[0].as_str(), "https://mirror1.example.com/repo/");
    assert_eq!(repo.baseurl[1].as_str(), "https://mirror2.example.com/repo/");
    assert_eq!(repo.baseurl[2].as_str(), "https://mirror3.example.com/repo/");
}

#[test]
fn test_parse_with_main_section() {
    let input = "[main]
best=1
debuglevel=5

[myrepo]
name=Test Repo
baseurl=https://example.com/repo/
enabled=1
";
    let rf = RepoFile::parse(input).expect("failed to parse");
    assert!(rf.main.is_some(), "main section should be present");
    let main_block = rf.main.as_ref().unwrap();
    assert_eq!(main_block.data.best, Some(DnfBool::True));
    assert_eq!(
        main_block.data.debuglevel,
        Some(DebugLevel::try_new(5).unwrap())
    );

    let rid = RepoId::try_new("myrepo").unwrap();
    assert!(rf.contains(&rid), "myrepo should exist");
}

#[test]
fn test_parse_preserves_comments() {
    let input = "# Header comment for the file
[myrepo]
# Leading comment on name
name=My Repo # inline comment
baseurl=https://example.com/repo/
enabled=1
";
    let rf = RepoFile::parse(input).expect("failed to parse");
    let rid = RepoId::try_new("myrepo").unwrap();
    let block = rf.get(&rid).unwrap();

    // Check preamble captures the file-level comment before any section
    let has_header = rf.preamble.iter().any(|l| l.contains("Header comment"));
    assert!(has_header, "file-level comment should be preserved in preamble");

    // Check inline comments preserved in item_comments
    assert_eq!(
        block.item_comments.get("name"),
        Some(&"inline comment".to_string())
    );

    // Check raw_entries contain leading comments
    let name_entry = block.raw_entries.iter().find(|e| e.key == "name").unwrap();
    assert!(
        name_entry
            .leading_comments
            .iter()
            .any(|l| l.contains("Leading comment on name")),
        "leading comments preserved on name entry"
    );

    // Check inline comment on raw entry
    assert_eq!(
        name_entry.inline_comment,
        Some("inline comment".to_string())
    );
}

#[test]
fn test_parse_extras() {
    let input = "[custom]
name=Custom Repo
baseurl=https://example.com/repo/
custom_option=some_value
another_unknown=hello
custom_option=second_value
enabled=1
";
    let rf = RepoFile::parse(input).expect("failed to parse");
    let rid = RepoId::try_new("custom").unwrap();
    let repo = &rf.get(&rid).unwrap().data;

    // extras captures unknown keys
    assert_eq!(
        repo.extras.get("custom_option"),
        Some(&vec!["some_value".to_string(), "second_value".to_string()])
    );
    assert_eq!(
        repo.extras.get("another_unknown"),
        Some(&vec!["hello".to_string()])
    );

    // raw_entries also contain all entries including unknown
    let block = rf.get(&rid).unwrap();
    let custom_opts: Vec<&RawEntry> = block
        .raw_entries
        .iter()
        .filter(|e| e.key == "custom_option")
        .collect();
    assert_eq!(custom_opts.len(), 2);
}

#[test]
fn test_parse_boolean_variants() {
    // Test all 8 boolean forms
    let input = "[booleans]
enabled=1
flag_yes=yes
flag_true=true
flag_on=on
disabled=0
flag_no=no
flag_false=false
flag_off=off
";
    let rf = RepoFile::parse(input).expect("failed to parse");
    let rid = RepoId::try_new("booleans").unwrap();
    let block = rf.get(&rid).unwrap();

    // Check raw values were captured
    let get_val = |key: &str| -> &str {
        block
            .raw_entries
            .iter()
            .find(|e| e.key == key)
            .map(|e| e.value.as_str())
            .unwrap()
    };

    assert_eq!(get_val("enabled"), "1");
    assert_eq!(get_val("flag_yes"), "yes");
    assert_eq!(get_val("flag_true"), "true");
    assert_eq!(get_val("flag_on"), "on");
    assert_eq!(get_val("disabled"), "0");
    assert_eq!(get_val("flag_no"), "no");
    assert_eq!(get_val("flag_false"), "false");
    assert_eq!(get_val("flag_off"), "off");

    let repo = &block.data;

    // Verify the known key "enabled" was parsed into the typed field
    assert_eq!(repo.enabled, Some(DnfBool::True));

    // Unknown keys go to extras
    assert_eq!(
        repo.extras.get("flag_yes"),
        Some(&vec!["yes".to_string()])
    );
}

#[test]
fn test_parse_invalid_section() {
    // Section name with invalid characters (only alphanumeric, dash, underscore, dot, colon allowed)
    let input = "[invalid repo id with spaces]
name=Bad Repo
";
    let result = RepoFile::parse(input);
    assert!(result.is_err(), "expected parse error for invalid repo ID");

    match result {
        Err(e) => {
            let err_str = e.to_string();
            assert!(
                err_str.contains("invalid repo ID"),
                "expected InvalidRepoId error, got: {err_str}"
            );
        }
        Ok(_) => panic!("expected error"),
    }

    // Empty section name
    let input2 = "[]\nname=Empty\n";
    let result2 = RepoFile::parse(input2);
    assert!(result2.is_err(), "expected parse error for empty section name");
}

#[test]
fn test_round_trip() {
    let input = "# Preamble comment
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

[custom-repo]
name=My Custom Packages
baseurl=https://custom.example.com/repo/
enabled=0
gpgcheck=0
module_hotfixes=1
cost=500
# This repo has some custom options
custom_option=some_value
";
    // First parse
    let rf1 = RepoFile::parse(input).expect("first parse failed");

    // Render
    let rendered = rf1.render();

    // Parse again
    let rf2 = RepoFile::parse(&rendered).expect("second parse failed");

    // Compare: same number of repos
    assert_eq!(rf1.len(), rf2.len(), "same repo count");
    assert_eq!(rf1.main.is_some(), rf2.main.is_some(), "both have or lack main");

    // Compare preamble
    assert_eq!(rf1.preamble, rf2.preamble, "preamble should match");

    // Compare main section
    if let (Some(m1), Some(m2)) = (rf1.main(), rf2.main()) {
        assert_eq!(m1.data, m2.data, "main config data should match");
        assert_eq!(m1.raw_entries.len(), m2.raw_entries.len(), "main raw entries count should match");
    }

    // Compare each repo
    for (rid, block1) in rf1.iter() {
        let block2 = rf2.get(rid).expect("repo should exist in second parse");
        assert_eq!(block1.data, block2.data, "repo {rid} data should match");
        assert_eq!(
            block1.raw_entries.len(),
            block2.raw_entries.len(),
            "repo {rid} raw entries count should match"
        );
    }
}

#[test]
fn test_parse_empty_file() {
    let rf = RepoFile::parse("").expect("empty input should parse OK");
    assert_eq!(rf.len(), 0, "no repos expected");
    assert!(rf.main.is_none(), "no main section expected");
    assert!(rf.preamble.is_empty(), "empty preamble");
    assert!(rf.is_empty());
}

#[test]
fn test_parse_comment_only_file() {
    let input = "# Just a comment
; Another comment style
# Third comment
";
    let rf = RepoFile::parse(input).expect("comment-only input should parse OK");
    assert!(rf.is_empty(), "no repos");
    assert!(rf.main.is_none(), "no main");
    // Comments should be in preamble
    assert!(!rf.preamble.is_empty(), "preamble should contain comments");
    assert!(rf.preamble.iter().any(|l| l.contains("Just a comment")));
}

#[test]
fn test_from_str_trait() {
    let input = "[fromstr]\nname=FromStr Test\nbaseurl=https://example.com/repo/\n";
    let rf = RepoFile::from_str(input).expect("FromStr should work");
    let rid = RepoId::try_new("fromstr").unwrap();
    assert!(rf.contains(&rid));
}

#[test]
fn test_add_and_remove() {
    let mut rf = RepoFile::new();
    let rid = RepoId::try_new("testrepo").unwrap();
    let repo = Repo::new(rid.clone());

    assert!(rf.is_empty());
    rf.add(repo).expect("add should succeed");
    assert_eq!(rf.len(), 1);
    assert!(rf.contains(&rid));

    // Adding duplicate should fail
    let dup = Repo::new(rid.clone());
    let err = rf.add(dup);
    assert!(err.is_err(), "adding duplicate should fail");

    // Remove
    let removed = rf.remove(&rid);
    assert!(removed.is_some(), "remove should return the block");
    assert!(rf.is_empty());
}

#[test]
fn test_set_and_get_mut() {
    let mut rf = RepoFile::new();
    let rid = RepoId::try_new("myrepo").unwrap();
    let mut repo = Repo::new(rid.clone());
    repo.name = Some(RepoName::try_new("Original Name").unwrap());
    rf.set(repo);

    // Modify via get_mut
    let block = rf.get_mut(&rid).unwrap();
    block.data.name = Some(RepoName::try_new("Updated Name").unwrap());

    let block = rf.get(&rid).unwrap();
    assert_eq!(
        block.data.name.as_ref().map(|n| n.as_ref()),
        Some("Updated Name")
    );
}

#[test]
fn test_main_methods() {
    let mut rf = RepoFile::new();
    assert!(rf.main().is_none());
    assert!(rf.main_mut().is_none());

    let mc = MainConfig::default();
    rf.set_main(mc);
    assert!(rf.main().is_some());

    if let Some(main) = rf.main_mut() {
        main.data.keepcache = Some(DnfBool::True);
    }

    assert_eq!(rf.main().unwrap().data.keepcache, Some(DnfBool::True));

    rf.remove_main();
    assert!(rf.main().is_none());
}

#[test]
fn test_iter_and_repo_ids() {
    let mut rf = RepoFile::new();
    for name in &["alpha", "beta", "gamma"] {
        let rid = RepoId::try_new(*name).unwrap();
        rf.add(Repo::new(rid)).unwrap();
    }

    let ids: Vec<&RepoId> = rf.repo_ids().collect();
    assert_eq!(ids.len(), 3);

    let names: Vec<&str> = rf.iter().map(|(id, _)| id.as_ref()).collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn test_parse_key_value_before_section() {
    // Key-value before any section goes to preamble as raw lines
    let input = "key_before_section=value1\n[sec1]\nname=Test\n";
    let rf = RepoFile::parse(input).expect("should parse");
    assert!(
        rf.preamble.iter().any(|l| l.contains("key_before_section")),
        "pre-section key=value should be in preamble"
    );
    assert!(rf.contains(&RepoId::try_new("sec1").unwrap()));
}

#[test]
fn test_parse_trailing_newline_handling() {
    // File ending without newline
    let input = "[test]\nname=Test";
    let rf = RepoFile::parse(input).expect("should parse without trailing newline");
    assert!(rf.contains(&RepoId::try_new("test").unwrap()));
}

#[test]
fn test_parse_missing_equals() {
    let input = "[test]\nname\n";
    let result = RepoFile::parse(input);
    assert!(result.is_err(), "missing = should error");

    // Also test key that is only whitespace before =
    let input2 = "[test]\n =value\n";
    let result2 = RepoFile::parse(input2);
    assert!(
        result2.is_err(),
        "empty key before = should error"
    );
}

#[test]
fn test_parse_semicolon_comment() {
    let input = "; semicolon comment line\n[test]\nname=Semicolon\n";
    let rf = RepoFile::parse(input).expect("semicolon comments should work");
    assert!(
        rf.preamble.iter().any(|l| l.contains("semicolon comment")),
        "semicolon comments should be in preamble"
    );
}
