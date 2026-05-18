use dnf_repofile::repo::Repo;
use dnf_repofile::types::*;

#[test]
fn test_validate_repo_no_url_source() {
    let repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    let report = repo.validate();
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.message.contains("URL")));
}

#[test]
fn test_validate_repo_with_baseurl_passes() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl
        .push("https://example.com/repo/".parse().unwrap());
    assert!(repo.validate().is_ok());
}

#[test]
fn test_validate_gpgkey_without_gpgcheck_warns() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl
        .push("https://example.com/repo/".parse().unwrap());
    repo.gpgkey.push("https://example.com/key".to_string());
    repo.gpgcheck = Some(DnfBool::False);
    assert!(repo
        .validate()
        .warnings
        .iter()
        .any(|w| w.message.contains("gpg")));
}
