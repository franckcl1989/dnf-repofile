use dnf_repofile::reposdir::*;
use dnf_repofile::types::*;
use tempfile::TempDir;

#[test]
fn test_load_directory() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("test.repo"),
        "[epel]\nname=EPEL\nbaseurl=https://x.com/\n",
    )
    .unwrap();
    let rd = ReposDir::load(dir.path()).unwrap();
    assert_eq!(rd.file_names().len(), 1);
}

#[test]
fn test_find_repo_cross_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("a.repo"),
        "[ra]\nname=A\nbaseurl=https://a.com/\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.repo"),
        "[rb]\nname=B\nbaseurl=https://b.com/\n",
    )
    .unwrap();
    let rd = ReposDir::load(dir.path()).unwrap();
    let (fname, repo) = rd.find_repo(&RepoId::try_new("rb").unwrap()).unwrap();
    assert!(fname.contains("b.repo"));
    assert_eq!(repo.name.as_ref().unwrap().as_ref(), "B");
}

#[test]
fn test_repo_count() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("m.repo"),
        "[r1]\nname=R1\nbaseurl=https://1.com/\n[r2]\nname=R2\nbaseurl=https://2.com/\n",
    )
    .unwrap();
    assert_eq!(ReposDir::load(dir.path()).unwrap().repo_count(), 2);
}

#[test]
fn test_remove_file() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("rm.repo"),
        "[r1]\nname=R1\nbaseurl=https://1.com/\n",
    )
    .unwrap();
    let mut rd = ReposDir::load(dir.path()).unwrap();
    assert!(rd.remove_file("rm.repo").unwrap().is_some());
    assert!(!dir.path().join("rm.repo").exists());
}

#[test]
fn test_validate_detects_duplicate_ids() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("a.repo"),
        "[dupe]\nname=A\nbaseurl=https://a.com/\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.repo"),
        "[dupe]\nname=B\nbaseurl=https://b.com/\n",
    )
    .unwrap();
    assert!(!ReposDir::load(dir.path()).unwrap().validate().is_ok());
}
