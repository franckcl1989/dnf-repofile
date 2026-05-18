use dnf_repofile::repo::Repo;
use dnf_repofile::types::*;

#[test]
fn test_repo_new_empty() {
    let repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    assert_eq!(repo.id.as_ref(), "testrepo");
    assert!(repo.name.is_none());
    assert!(repo.baseurl.is_empty());
    assert!(repo.mirrorlist.is_none());
    assert!(repo.metalink.is_none());
}

#[test]
fn test_repo_url_source_none_when_empty() {
    let repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    assert!(repo.url_source().is_none());
}

#[test]
fn test_repo_url_source_baseurl() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl.push("https://example.com/repo/".parse().unwrap());
    match repo.url_source() {
        Some(UrlSource::BaseUrl(urls)) => assert_eq!(urls.len(), 1),
        other => panic!("expected BaseUrl, got {:?}", other),
    }
}

#[test]
fn test_repo_url_source_mirrorlist() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.mirrorlist = Some("https://example.com/mirrorlist".parse().unwrap());
    match repo.url_source() {
        Some(UrlSource::MirrorList(_)) => {},
        other => panic!("expected MirrorList, got {:?}", other),
    }
}

#[test]
fn test_repo_url_source_prefers_baseurl_when_all_set() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.baseurl.push("https://example.com/repo/".parse().unwrap());
    repo.mirrorlist = Some("https://example.com/mirrorlist".parse().unwrap());
    repo.metalink = Some("https://example.com/metalink".parse().unwrap());
    match repo.url_source() {
        Some(UrlSource::BaseUrl(_)) => {},
        other => panic!("expected BaseUrl, got {:?}", other),
    }
}

#[test]
fn test_repo_gpgkey_can_hold_bare_path() {
    let mut repo = Repo::new(RepoId::try_new("testrepo").unwrap());
    repo.gpgkey.push("/etc/pki/rpm-gpg/RPM-GPG-KEY".to_string());
    assert_eq!(repo.gpgkey[0], "/etc/pki/rpm-gpg/RPM-GPG-KEY");
}
