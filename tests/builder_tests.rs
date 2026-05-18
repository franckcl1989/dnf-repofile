use dnf_repofile::builder::RepoBuilder;
use dnf_repofile::types::*;

#[test]
fn test_builder_basic() {
    let repo = RepoBuilder::new(RepoId::try_new("myrepo").unwrap())
        .name(RepoName::try_new("My Repo").unwrap())
        .enabled(DnfBool::True)
        .gpgcheck(DnfBool::True)
        .baseurl("https://example.com/repo/".parse().unwrap())
        .gpgkey("https://example.com/RPM-GPG-KEY")
        .priority(Priority::try_new(50).unwrap())
        .build();
    assert_eq!(repo.id.as_ref(), "myrepo");
    assert_eq!(repo.name.unwrap().as_ref(), "My Repo");
    assert_eq!(repo.baseurl[0].as_str(), "https://example.com/repo/");
    assert_eq!(repo.gpgkey[0], "https://example.com/RPM-GPG-KEY");
}

#[test]
fn test_builder_from_existing() {
    let existing = RepoBuilder::new(RepoId::try_new("myrepo").unwrap())
        .name(RepoName::try_new("Original").unwrap())
        .enabled(DnfBool::True)
        .baseurl("https://example.com/".parse().unwrap())
        .build();
    let modified = RepoBuilder::from(&existing).enabled(DnfBool::False).build();
    assert_eq!(modified.name.unwrap().as_ref(), "Original");
    assert_eq!(modified.enabled.unwrap(), DnfBool::False);
}
