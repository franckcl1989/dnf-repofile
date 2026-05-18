use dnf_repofile::builder::RepoBuilder;
use dnf_repofile::diff::*;
use dnf_repofile::repofile::RepoFile;
use dnf_repofile::types::*;

#[test]
fn test_diff_repos_changed_option() {
    let a = RepoBuilder::new(RepoId::try_new("test").unwrap())
        .name(RepoName::try_new("Old").unwrap())
        .baseurl("https://x.com/".parse().unwrap())
        .build();
    let b = RepoBuilder::new(RepoId::try_new("test").unwrap())
        .name(RepoName::try_new("New").unwrap())
        .baseurl("https://x.com/".parse().unwrap())
        .build();
    let d = diff_repos(&a, &b);
    assert!(d.has_changes);
    assert_eq!(d.changed[0].0, "name");
}

#[test]
fn test_diff_repos_no_changes() {
    let a = RepoBuilder::new(RepoId::try_new("test").unwrap())
        .name(RepoName::try_new("T").unwrap())
        .baseurl("https://x.com/".parse().unwrap())
        .build();
    assert!(!diff_repos(&a, &a).has_changes);
}

#[test]
fn test_diff_files_added_repo() {
    let mut b = RepoFile::new();
    let repo = RepoBuilder::new(RepoId::try_new("newrepo").unwrap())
        .name(RepoName::try_new("N").unwrap())
        .baseurl("https://x.com/".parse().unwrap())
        .build();
    b.add(repo).unwrap();
    let d = diff_files(&RepoFile::new(), &b);
    assert!(d.has_changes);
    assert_eq!(d.repos_added.len(), 1);
}
