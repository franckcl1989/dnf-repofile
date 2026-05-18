use dnf_repofile::*;

#[test]
fn test_full_workflow() {
    let input = include_str!("fixtures/complex.repo");
    let mut rf = RepoFile::parse(input).unwrap();

    // Verify [main]
    let main_block = rf.main().unwrap();
    assert_eq!(main_block.data.extras.get("max_parallel_downloads").unwrap()[0], "10");

    // Verify repos
    assert_eq!(rf.len(), 3);
    let baseos = rf.get(&RepoId::try_new("baseos").unwrap()).unwrap();
    assert_eq!(baseos.data.name.as_ref().unwrap().as_ref(), "Rocky Linux $releasever - BaseOS");
    assert_eq!(baseos.data.baseurl.len(), 2);
    assert_eq!(baseos.data.priority.unwrap().to_string(), "10");

    // Modify a repo -- update both typed data and raw entry for round-trip consistency
    {
        let block = rf.get_mut(&RepoId::try_new("custom-repo").unwrap()).unwrap();
        block.data.enabled = Some(DnfBool::True);
        if let Some(entry) = block.raw_entries.iter_mut().find(|e| e.key == "enabled") {
            entry.value = "1".to_string();
        }
    }

    // Add a new repo
    let new_repo = RepoBuilder::new(RepoId::try_new("added-repo").unwrap())
        .name(RepoName::try_new("Added Repo").unwrap())
        .baseurl("https://added.example.com/".parse().unwrap())
        .enabled(DnfBool::True)
        .gpgcheck(DnfBool::True)
        .gpgkey("https://added.example.com/key")
        .build();
    rf.add(new_repo).unwrap();

    // Remove a repo
    rf.remove(&RepoId::try_new("appstream").unwrap());

    // Render and re-parse
    let output = rf.render();
    let rf2 = RepoFile::parse(&output).unwrap();
    assert_eq!(rf2.len(), 3); // baseos, custom-repo, added-repo
    assert!(rf2.get(&RepoId::try_new("appstream").unwrap()).is_none());
    assert!(rf2.get(&RepoId::try_new("added-repo").unwrap()).is_some());
    let custom = rf2.get(&RepoId::try_new("custom-repo").unwrap()).unwrap();
    assert_eq!(custom.data.enabled, Some(DnfBool::True));
}

#[test]
fn test_variable_expansion_in_url() {
    let input = "[testrepo]\nname=Test\nbaseurl=https://example.com/$releasever/$basearch/\n";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.data.baseurl[0].as_str(), "https://example.com/$releasever/$basearch/");
}

#[test]
fn test_parse_validates_all_bool_variants() {
    let input = "[testrepo]\nname=Test\nbaseurl=https://example.com/\nenabled=yes\ngpgcheck=1\nskip_if_unavailable=True\nmodule_hotfixes=on\ndeltarpm=No\nfastestmirror=false\ncountme=OFF\n";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.data.enabled, Some(DnfBool::True));
    assert_eq!(block.data.gpgcheck, Some(DnfBool::True));
    assert_eq!(block.data.skip_if_unavailable, Some(DnfBool::True));
    assert_eq!(block.data.module_hotfixes, Some(DnfBool::True));
    assert_eq!(block.data.deltarpm, Some(DnfBool::False));
    assert_eq!(block.data.fastestmirror, Some(DnfBool::False));
    assert_eq!(block.data.countme, Some(DnfBool::False));
}

#[test]
fn test_parse_proxy_none() {
    let input = "[testrepo]\nname=Test\nbaseurl=https://example.com/\nproxy=_none_\n";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert!(matches!(block.data.proxy, ProxySetting::Disabled));
}

#[test]
fn test_parse_with_extras_preserved() {
    let input = "[testrepo]\nname=Test\nbaseurl=https://example.com/\ncustom_key=custom_value\nanother_key=another_value\n";
    let rf = RepoFile::parse(input).unwrap();
    let block = rf.get(&RepoId::try_new("testrepo").unwrap()).unwrap();
    assert_eq!(block.data.extras.get("custom_key").unwrap()[0], "custom_value");
    assert_eq!(block.data.extras.len(), 2);
}

#[test]
fn test_diff_detects_all_changes() {
    let a = RepoFile::parse("[repo]\nname=A\nbaseurl=https://a.example.com/\nenabled=1\n").unwrap();
    let b = RepoFile::parse("[repo]\nname=B\nbaseurl=https://b.example.com/\nenabled=0\n").unwrap();
    let diff = diff_files(&a, &b);
    assert!(diff.has_changes);
}
